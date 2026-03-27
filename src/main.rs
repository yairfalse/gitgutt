mod auth;
mod cli;
mod domain;
mod github;
mod render;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;

use crate::cli::{Cli, Command};
use crate::domain::metrics::{compute_author_stats, compute_pr_metrics, compute_review_metrics};
use crate::domain::types::Period;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Resolve auth
    let token = auth::resolve_token()?;

    // Resolve repo
    let repo_str = match &cli.repo {
        Some(r) => r.clone(),
        None => auth::detect_repo()?,
    };

    let (owner, repo) = repo_str
        .split_once('/')
        .context("repo must be in owner/repo format")?;

    // Build period
    let period = Period {
        start: Utc::now() - chrono::Duration::days(cli.days as i64),
        end: Utc::now(),
    };

    // Fetch data
    let client = github::create_client(&token).await?;
    let prs = github::fetch_pull_requests(&client, owner, repo, &period).await?;

    if prs.is_empty() {
        println!("\n  no pull requests found for {repo_str} in the last {} days\n", cli.days);
        return Ok(());
    }

    // Compute & render
    match cli.command() {
        Command::Stats => {
            let metrics = compute_pr_metrics(&prs, &period);
            let review = compute_review_metrics(&prs);
            render::render_dashboard(&metrics, &review, &repo_str);
        }
        Command::Prs => {
            let metrics = compute_pr_metrics(&prs, &period);
            render::render_pr_report(&metrics, &prs, &repo_str);
        }
        Command::Authors => {
            let metrics = compute_pr_metrics(&prs, &period);
            let stats = compute_author_stats(&prs);
            render::render_author_report(&stats, &metrics, &repo_str);
        }
        Command::Review => {
            let metrics = compute_pr_metrics(&prs, &period);
            let review = compute_review_metrics(&prs);
            render::render_review_report(&review, &metrics, &repo_str);
        }
    }

    println!();
    Ok(())
}
