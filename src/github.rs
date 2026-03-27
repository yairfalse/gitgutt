use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use octocrab::models::pulls::PullRequest as GhPullRequest;
use octocrab::models::pulls::ReviewState as GhReviewState;
use octocrab::Octocrab;

use crate::domain::types::{Period, PrState, PullRequest, Review, ReviewState};

pub async fn create_client(token: &str) -> Result<Octocrab> {
    Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .context("failed to create GitHub client")
}

pub async fn fetch_pull_requests(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    period: &Period,
) -> Result<Vec<PullRequest>> {
    let mut all_prs = Vec::new();
    let mut page: u32 = 1;

    loop {
        let gh_prs = client
            .pulls(owner, repo)
            .list()
            .state(octocrab::params::State::All)
            .sort(octocrab::params::pulls::Sort::Updated)
            .direction(octocrab::params::Direction::Descending)
            .per_page(100)
            .page(page)
            .send()
            .await
            .context("failed to fetch pull requests")?;

        let items = gh_prs.items;
        if items.is_empty() {
            break;
        }

        let mut past_period = false;
        for gh_pr in &items {
            let created: DateTime<Utc> = gh_pr.created_at.unwrap_or_default().into();
            if created < period.start {
                past_period = true;
                continue;
            }
            if created <= period.end {
                let pr = convert_pr(client, owner, repo, gh_pr).await?;
                all_prs.push(pr);
            }
        }

        if past_period || gh_prs.next.is_none() {
            break;
        }
        page += 1;
    }

    Ok(all_prs)
}

async fn convert_pr(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    gh_pr: &GhPullRequest,
) -> Result<PullRequest> {
    let number = gh_pr.number;
    let state = match (gh_pr.merged_at, gh_pr.closed_at) {
        (Some(_), _) => PrState::Merged,
        (None, Some(_)) => PrState::Closed,
        _ => PrState::Open,
    };

    let reviews = fetch_reviews(client, owner, repo, number).await.unwrap_or_default();

    // Get additions/deletions from the detailed PR endpoint
    let (additions, deletions) = match (gh_pr.additions, gh_pr.deletions) {
        (Some(a), Some(d)) => (a, d),
        _ => {
            // Fetch detailed PR for additions/deletions
            match client.pulls(owner, repo).get(number).await {
                Ok(detailed) => (
                    detailed.additions.unwrap_or(0),
                    detailed.deletions.unwrap_or(0),
                ),
                Err(_) => (0, 0),
            }
        }
    };

    Ok(PullRequest {
        number: number as u64,
        title: gh_pr.title.clone().unwrap_or_default(),
        author: gh_pr
            .user
            .as_ref()
            .map(|u| u.login.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        state,
        created_at: gh_pr.created_at.unwrap_or_default().into(),
        merged_at: gh_pr.merged_at.map(Into::into),
        closed_at: gh_pr.closed_at.map(Into::into),
        first_commit_at: None, // Would require commits API — skip for v1
        additions: additions as u64,
        deletions: deletions as u64,
        reviews,
    })
}

async fn fetch_reviews(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> Result<Vec<Review>> {
    let gh_reviews = client
        .pulls(owner, repo)
        .list_reviews(pr_number)
        .send()
        .await
        .context("failed to fetch reviews")?;

    let reviews = gh_reviews
        .items
        .into_iter()
        .filter_map(|r| {
            let state = match r.state? {
                GhReviewState::Approved => ReviewState::Approved,
                GhReviewState::ChangesRequested => ReviewState::ChangesRequested,
                GhReviewState::Commented => ReviewState::Commented,
                _ => return None,
            };
            Some(Review {
                author: r.user.map(|u| u.login).unwrap_or_else(|| "unknown".to_string()),
                submitted_at: r.submitted_at?.into(),
                state,
            })
        })
        .collect();

    Ok(reviews)
}
