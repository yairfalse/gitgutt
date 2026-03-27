use crate::domain::metrics::AuthorStats;
use crate::domain::stats::format_duration;
use crate::domain::types::PullRequest;
use crate::render::theme::*;

pub fn author_table(stats: &[AuthorStats]) {
    println!("  {:12}  {:>6}  {:>6}  {:>7}  {:>10}  {:>4}",
        dim("AUTHOR"),
        dim("MERGED"),
        dim("OPENED"),
        dim("REVIEWS"),
        dim("MERGE TIME"),
        dim("SIZE"),
    );
    println!("  {}", "\u{2500}".repeat(58).bright_black());

    for author in stats {
        let merge_time = author
            .median_merge_time
            .map(|d| format_duration(d))
            .unwrap_or_else(|| "-".to_string());

        let size_label = match author.median_pr_size {
            0..=10 => "XS",
            11..=50 => "S",
            51..=200 => "M",
            201..=500 => "L",
            _ => "XL",
        };

        println!("  {:12}  {:>6}  {:>6}  {:>7}  {:>10}  {:>4}",
            author.author.as_str(),
            accent_num(author.prs_merged),
            dim_num(author.prs_opened),
            dim_num(author.reviews_given),
            accent(&merge_time),
            dim(size_label),
        );
    }

    println!("  {}", "\u{2500}".repeat(58).bright_black());
}

pub fn slowest_prs_table(prs: &[&PullRequest]) {
    for pr in prs {
        let duration = pr
            .merge_duration()
            .and_then(|d| d.to_std().ok())
            .map(format_duration)
            .unwrap_or_else(|| "-".to_string());

        println!("  {}  {:40}  {:8}  {:>5}  {}",
            dim(&format!("#{}", pr.number)),
            truncate(&pr.title, 40),
            dim(&pr.author),
            warn(&duration),
            dim(&format!("+{}/-{}", pr.additions, pr.deletions)),
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}\u{2026}", &s[..max - 1])
    }
}

use colored::Colorize;
