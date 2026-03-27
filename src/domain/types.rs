use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub author: String,
    pub state: PrState,
    pub created_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub first_commit_at: Option<DateTime<Utc>>,
    pub additions: u64,
    pub deletions: u64,
    pub reviews: Vec<Review>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrState {
    Open,
    Merged,
    Closed,
}

#[derive(Debug, Clone)]
pub struct Review {
    pub author: String,
    pub submitted_at: DateTime<Utc>,
    pub state: ReviewState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
}

#[derive(Debug, Clone)]
pub struct Period {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl PullRequest {
    pub fn size(&self) -> u64 {
        self.additions + self.deletions
    }

    pub fn size_bucket(&self) -> SizeBucket {
        match self.size() {
            0..=10 => SizeBucket::XS,
            11..=50 => SizeBucket::S,
            51..=200 => SizeBucket::M,
            201..=500 => SizeBucket::L,
            _ => SizeBucket::XL,
        }
    }

    pub fn merge_duration(&self) -> Option<chrono::Duration> {
        self.merged_at.map(|merged| merged - self.created_at)
    }

    pub fn cycle_duration(&self) -> Option<chrono::Duration> {
        match (self.first_commit_at, self.merged_at) {
            (Some(first), Some(merged)) => Some(merged - first),
            _ => None,
        }
    }

    pub fn time_to_first_review(&self) -> Option<chrono::Duration> {
        self.reviews
            .iter()
            .map(|r| r.submitted_at)
            .min()
            .map(|first_review| first_review - self.created_at)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SizeBucket {
    XS,
    S,
    M,
    L,
    XL,
}

impl std::fmt::Display for SizeBucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SizeBucket::XS => write!(f, "XS"),
            SizeBucket::S => write!(f, " S"),
            SizeBucket::M => write!(f, " M"),
            SizeBucket::L => write!(f, " L"),
            SizeBucket::XL => write!(f, "XL"),
        }
    }
}
