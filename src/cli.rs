use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "gitgutt", about = "GitHub PR metrics for engineering teams")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// GitHub repository (owner/repo). Auto-detected from git remote if omitted.
    #[arg(short, long, global = true)]
    pub repo: Option<String>,

    /// Number of days to analyze
    #[arg(short, long, global = true, default_value = "30")]
    pub days: u32,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Overview dashboard with key metrics
    Stats,
    /// PR metrics: merge time distribution, size buckets, slowest PRs
    Prs,
    /// Per-author breakdown: PRs merged, reviews given, merge time
    Authors,
    /// Review health: time to first review, turnaround, top reviewers
    Review,
}

impl Cli {
    pub fn command(&self) -> &Command {
        self.command.as_ref().unwrap_or(&Command::Stats)
    }
}
