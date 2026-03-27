pub mod charts;
pub mod tables;
pub mod theme;

use crate::domain::metrics::{AuthorStats, PrMetrics, ReviewMetrics};
use crate::domain::stats::format_duration;
use crate::domain::types::PullRequest;

use self::charts::{bar, sparkline};
use self::tables::{author_table, slowest_prs_table};
use self::theme::*;

pub fn render_dashboard(metrics: &PrMetrics, review: &ReviewMetrics, repo: &str) {
    header("gitgutt", repo, metrics.period.end - metrics.period.start);
    println!();

    // Row 1: PR counts | Velocity | Merge time
    let days = (metrics.period.end - metrics.period.start).num_days().max(1) as f64;
    let velocity = metrics.merged as f64 / days;

    print_section("PULL REQUESTS");
    println!("  {}  total", accent_num(metrics.total));
    println!("  {}  merged", success_num(metrics.merged));
    println!("  {}  open", dim_num(metrics.open));
    println!("  {}  closed", dim_num(metrics.closed));
    println!();

    print_section("VELOCITY");
    println!("  {} pr/day", accent_num_f(velocity, 1));
    if !metrics.daily_merged.is_empty() {
        println!("  {}", sparkline(&metrics.daily_merged));
    }
    println!();

    print_section("MERGE TIME");
    if let Some(ref dist) = metrics.merge_time {
        println!("  {} median   {} p90   {} p99",
            accent(&format_duration(dist.median)),
            dim(&format_duration(dist.p90)),
            dim(&format_duration(dist.p99)),
        );
    } else {
        println!("  {}", dim("no merged PRs"));
    }
    println!();

    // Row 2: Size distribution
    print_section("SIZE DISTRIBUTION");
    let max_count = metrics.size_buckets.iter().map(|b| b.count).max().unwrap_or(1);
    for bucket in &metrics.size_buckets {
        println!("  {}  {}  {}",
            dim(&format!("{}", bucket.bucket)),
            bar(bucket.count, max_count, 30),
            dim_num(bucket.count),
        );
    }
    println!();

    // Row 3: Review health
    print_section("REVIEW HEALTH");
    if let Some(ref dist) = review.time_to_first_review {
        println!("  first review   {} median", accent(&format_duration(dist.median)));
    }
    if review.reviews_per_pr > 0.0 {
        println!("  reviews/pr     {} avg", accent_num_f(review.reviews_per_pr, 1));
    }
    println!();

    if !review.top_reviewers.is_empty() {
        print_section("TOP REVIEWERS");
        let max_reviews = review.top_reviewers.first().map(|r| r.1).unwrap_or(1);
        for (name, count) in review.top_reviewers.iter().take(5) {
            println!("  {:12}  {}  {}",
                dim(name),
                bar(*count, max_reviews, 20),
                dim_num(*count),
            );
        }
    }
}

pub fn render_pr_report(metrics: &PrMetrics, prs: &[PullRequest], repo: &str) {
    header("gitgutt prs", repo, metrics.period.end - metrics.period.start);
    println!();

    print_section("MERGE TIME DISTRIBUTION");
    if let Some(ref dist) = metrics.merge_time {
        let buckets = [
            (" <1h", 0u64..3600),
            ("1-4h", 3600..14400),
            ("4-12h", 14400..43200),
            ("12-24h", 43200..86400),
            (" 1-3d", 86400..259200),
            ("  >3d", 259200..u64::MAX),
        ];

        let merged_prs: Vec<&PullRequest> = prs
            .iter()
            .filter(|pr| pr.merged_at.is_some())
            .collect();

        let mut bucket_counts: Vec<usize> = Vec::new();
        for (_, range) in &buckets {
            let count = merged_prs
                .iter()
                .filter(|pr| {
                    pr.merge_duration()
                        .and_then(|d| d.to_std().ok())
                        .is_some_and(|d| {
                            let secs = d.as_secs();
                            secs >= range.start && secs < range.end
                        })
                })
                .count();
            bucket_counts.push(count);
        }

        let max_count = *bucket_counts.iter().max().unwrap_or(&1);
        let total = merged_prs.len().max(1);

        for (i, (label, _)) in buckets.iter().enumerate() {
            let count = bucket_counts[i];
            let pct = (count as f64 / total as f64) * 100.0;
            println!("  {}  {}  {}  {}",
                dim(label),
                bar(count, max_count, 40),
                dim_num(count),
                dim(&format!("{pct:4.0}%")),
            );
        }

        println!();
        println!("  median {}    p90 {}    p99 {}    min {}    max {}",
            accent(&format_duration(dist.median)),
            dim(&format_duration(dist.p90)),
            dim(&format_duration(dist.p99)),
            dim(&format_duration(dist.min)),
            dim(&format_duration(dist.max)),
        );
    } else {
        println!("  {}", dim("no merged PRs"));
    }

    println!();
    separator();
    println!();

    print_section("SLOWEST PRS");
    let mut merged: Vec<&PullRequest> = prs
        .iter()
        .filter(|pr| pr.merged_at.is_some())
        .collect();
    merged.sort_by(|a, b| {
        b.merge_duration()
            .unwrap_or_default()
            .cmp(&a.merge_duration().unwrap_or_default())
    });
    slowest_prs_table(&merged[..merged.len().min(5)]);

    if let Some(ref dist) = metrics.cycle_time {
        println!();
        print_section("CYCLE TIME");
        println!("  median {}    p90 {}    p99 {}",
            accent(&format_duration(dist.median)),
            dim(&format_duration(dist.p90)),
            dim(&format_duration(dist.p99)),
        );
        if !metrics.daily_merged.is_empty() {
            println!("  {}", sparkline(&metrics.daily_merged));
        }
    }
}

pub fn render_author_report(stats: &[AuthorStats], metrics: &PrMetrics, repo: &str) {
    header("gitgutt authors", repo, metrics.period.end - metrics.period.start);
    println!();

    print_section("AUTHOR BREAKDOWN");
    author_table(stats);

    println!();
    print_section("CONTRIBUTION BALANCE");
    let total_merged: usize = stats.iter().map(|a| a.prs_merged).sum();
    let max_merged = stats.first().map(|a| a.prs_merged).unwrap_or(1);
    for author in stats.iter().take(10) {
        if author.prs_merged == 0 {
            continue;
        }
        let pct = if total_merged > 0 {
            (author.prs_merged as f64 / total_merged as f64) * 100.0
        } else {
            0.0
        };
        println!("  {:12}  {}  {}",
            dim(&author.author),
            bar(author.prs_merged, max_merged, 40),
            dim(&format!("{pct:4.0}%")),
        );
    }
}

pub fn render_review_report(review: &ReviewMetrics, metrics: &PrMetrics, repo: &str) {
    header("gitgutt review", repo, metrics.period.end - metrics.period.start);
    println!();

    // Side-by-side stats
    print_section("TIME TO FIRST REVIEW");
    if let Some(ref dist) = review.time_to_first_review {
        println!("  median   {}", accent(&format_duration(dist.median)));
        println!("  p90      {}", dim(&format_duration(dist.p90)));
        println!("  p99      {}", dim(&format_duration(dist.p99)));
    } else {
        println!("  {}", dim("no review data"));
    }
    println!();

    if let Some(ref dist) = review.review_turnaround {
        print_section("REVIEW TURNAROUND");
        println!("  median   {}", accent(&format_duration(dist.median)));
        println!("  p90      {}", dim(&format_duration(dist.p90)));
        println!("  p99      {}", dim(&format_duration(dist.p99)));
        println!();
    }

    // First review distribution
    if let Some(ref dist) = review.time_to_first_review {
        print_section("FIRST REVIEW DISTRIBUTION");
        println!("  {} across {} PRs, median {}",
            dim(&format!("{} reviews total", dist.count)),
            dim(&format!("{}", dist.count)),
            accent(&format_duration(dist.median)),
        );
    }
    println!();

    separator();
    println!();

    if !review.top_reviewers.is_empty() {
        print_section("TOP REVIEWERS");
        let max_reviews = review.top_reviewers.first().map(|r| r.1).unwrap_or(1);
        for (name, count) in review.top_reviewers.iter().take(10) {
            println!("  {:12}  {}  {}",
                dim(name),
                bar(*count, max_reviews, 30),
                dim_num(*count),
            );
        }
    }

    println!();
    println!("  reviews/pr   {} avg", accent_num_f(review.reviews_per_pr, 1));
}
