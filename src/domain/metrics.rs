use std::collections::HashMap;
use std::time::Duration;

use crate::domain::stats::Distribution;
use crate::domain::types::{Period, PrState, PullRequest, SizeBucket};

#[derive(Debug)]
pub struct PrMetrics {
    pub period: Period,
    pub total: usize,
    pub merged: usize,
    pub closed: usize,
    pub open: usize,
    pub merge_time: Option<Distribution>,
    pub cycle_time: Option<Distribution>,
    pub size_buckets: Vec<BucketCount>,
    pub daily_merged: Vec<u32>,
}

#[derive(Debug)]
pub struct BucketCount {
    pub bucket: SizeBucket,
    pub count: usize,
}

#[derive(Debug)]
pub struct AuthorStats {
    pub author: String,
    pub prs_merged: usize,
    pub prs_opened: usize,
    pub reviews_given: usize,
    pub median_merge_time: Option<Duration>,
    pub median_pr_size: u64,
}

#[derive(Debug)]
pub struct ReviewMetrics {
    pub time_to_first_review: Option<Distribution>,
    pub review_turnaround: Option<Distribution>,
    pub reviews_per_pr: f64,
    pub top_reviewers: Vec<(String, usize)>,
}

pub fn compute_pr_metrics(prs: &[PullRequest], period: &Period) -> PrMetrics {
    let merged_prs: Vec<&PullRequest> = prs
        .iter()
        .filter(|pr| pr.state == PrState::Merged)
        .collect();

    let merge_durations: Vec<chrono::Duration> = merged_prs
        .iter()
        .filter_map(|pr| pr.merge_duration())
        .collect();

    let cycle_durations: Vec<chrono::Duration> = merged_prs
        .iter()
        .filter_map(|pr| pr.cycle_duration())
        .collect();

    let size_buckets = compute_size_buckets(prs);
    let daily_merged = compute_daily_merged(&merged_prs, period);

    PrMetrics {
        period: period.clone(),
        total: prs.len(),
        merged: merged_prs.len(),
        closed: prs.iter().filter(|pr| pr.state == PrState::Closed).count(),
        open: prs.iter().filter(|pr| pr.state == PrState::Open).count(),
        merge_time: Distribution::compute(&merge_durations),
        cycle_time: Distribution::compute(&cycle_durations),
        size_buckets,
        daily_merged,
    }
}

pub fn compute_author_stats(prs: &[PullRequest]) -> Vec<AuthorStats> {
    let mut authors: HashMap<String, AuthorAccumulator> = HashMap::new();

    for pr in prs {
        let acc = authors.entry(pr.author.clone()).or_default();
        acc.prs_opened += 1;
        if pr.state == PrState::Merged {
            acc.prs_merged += 1;
            if let Some(d) = pr.merge_duration() {
                if let Ok(std_d) = d.to_std() {
                    acc.merge_durations.push(std_d);
                }
            }
        }
        acc.total_size += pr.size();
        acc.pr_count += 1;

        for review in &pr.reviews {
            let reviewer = authors.entry(review.author.clone()).or_default();
            reviewer.reviews_given += 1;
        }
    }

    let mut stats: Vec<AuthorStats> = authors
        .into_iter()
        .map(|(author, acc)| {
            let mut sorted = acc.merge_durations.clone();
            sorted.sort();
            let median_merge_time = if sorted.is_empty() {
                None
            } else {
                Some(sorted[sorted.len() / 2])
            };

            AuthorStats {
                author,
                prs_merged: acc.prs_merged,
                prs_opened: acc.prs_opened,
                reviews_given: acc.reviews_given,
                median_merge_time,
                median_pr_size: if acc.pr_count > 0 {
                    acc.total_size / acc.pr_count as u64
                } else {
                    0
                },
            }
        })
        .collect();

    stats.sort_by(|a, b| b.prs_merged.cmp(&a.prs_merged));
    stats
}

pub fn compute_review_metrics(prs: &[PullRequest]) -> ReviewMetrics {
    let first_review_durations: Vec<chrono::Duration> = prs
        .iter()
        .filter_map(|pr| pr.time_to_first_review())
        .collect();

    let total_reviews: usize = prs.iter().map(|pr| pr.reviews.len()).sum();
    let prs_with_reviews = prs.iter().filter(|pr| !pr.reviews.is_empty()).count();

    let mut reviewer_counts: HashMap<String, usize> = HashMap::new();
    for pr in prs {
        for review in &pr.reviews {
            *reviewer_counts.entry(review.author.clone()).or_default() += 1;
        }
    }

    let mut top_reviewers: Vec<(String, usize)> = reviewer_counts.into_iter().collect();
    top_reviewers.sort_by(|a, b| b.1.cmp(&a.1));
    top_reviewers.truncate(10);

    ReviewMetrics {
        time_to_first_review: Distribution::compute(&first_review_durations),
        review_turnaround: None, // GitHub API doesn't expose review request timestamps
        reviews_per_pr: if prs_with_reviews > 0 {
            total_reviews as f64 / prs_with_reviews as f64
        } else {
            0.0
        },
        top_reviewers,
    }
}

fn compute_size_buckets(prs: &[PullRequest]) -> Vec<BucketCount> {
    let mut counts: HashMap<SizeBucket, usize> = HashMap::new();
    for bucket in [SizeBucket::XS, SizeBucket::S, SizeBucket::M, SizeBucket::L, SizeBucket::XL] {
        counts.insert(bucket, 0);
    }
    for pr in prs {
        *counts.entry(pr.size_bucket()).or_default() += 1;
    }

    [SizeBucket::XS, SizeBucket::S, SizeBucket::M, SizeBucket::L, SizeBucket::XL]
        .into_iter()
        .map(|bucket| BucketCount {
            bucket,
            count: counts[&bucket],
        })
        .collect()
}

fn compute_daily_merged(merged_prs: &[&PullRequest], period: &Period) -> Vec<u32> {
    let days = (period.end - period.start).num_days().max(1) as usize;
    let mut daily = vec![0u32; days];

    for pr in merged_prs {
        if let Some(merged_at) = pr.merged_at {
            let day = (merged_at - period.start).num_days();
            if day >= 0 && (day as usize) < days {
                daily[day as usize] += 1;
            }
        }
    }

    daily
}

#[derive(Default)]
struct AuthorAccumulator {
    prs_opened: usize,
    prs_merged: usize,
    reviews_given: usize,
    merge_durations: Vec<Duration>,
    total_size: u64,
    pr_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{PrState, Review, ReviewState};
    use chrono::Utc;

    fn make_pr(number: u64, author: &str, state: PrState, hours_to_merge: Option<i64>) -> PullRequest {
        let created = Utc::now() - chrono::Duration::days(7);
        PullRequest {
            number,
            title: format!("PR #{number}"),
            author: author.to_string(),
            state,
            created_at: created,
            merged_at: hours_to_merge.map(|h| created + chrono::Duration::hours(h)),
            closed_at: None,
            first_commit_at: Some(created - chrono::Duration::hours(1)),
            additions: 50,
            deletions: 20,
            reviews: vec![Review {
                author: "reviewer".to_string(),
                submitted_at: created + chrono::Duration::hours(1),
                state: ReviewState::Approved,
            }],
        }
    }

    #[test]
    fn pr_metrics_counts() {
        let prs = vec![
            make_pr(1, "alice", PrState::Merged, Some(4)),
            make_pr(2, "bob", PrState::Merged, Some(8)),
            make_pr(3, "carol", PrState::Open, None),
            make_pr(4, "dave", PrState::Closed, None),
        ];
        let period = Period {
            start: Utc::now() - chrono::Duration::days(30),
            end: Utc::now(),
        };
        let metrics = compute_pr_metrics(&prs, &period);
        assert_eq!(metrics.total, 4);
        assert_eq!(metrics.merged, 2);
        assert_eq!(metrics.open, 1);
        assert_eq!(metrics.closed, 1);
    }

    #[test]
    fn author_stats_sorted_by_merged() {
        let prs = vec![
            make_pr(1, "alice", PrState::Merged, Some(4)),
            make_pr(2, "alice", PrState::Merged, Some(6)),
            make_pr(3, "bob", PrState::Merged, Some(2)),
        ];
        let stats = compute_author_stats(&prs);
        assert_eq!(stats[0].author, "alice");
        assert_eq!(stats[0].prs_merged, 2);
    }

    #[test]
    fn review_metrics_top_reviewers() {
        let prs = vec![
            make_pr(1, "alice", PrState::Merged, Some(4)),
            make_pr(2, "bob", PrState::Merged, Some(6)),
        ];
        let metrics = compute_review_metrics(&prs);
        assert_eq!(metrics.top_reviewers[0].0, "reviewer");
        assert_eq!(metrics.top_reviewers[0].1, 2);
    }
}
