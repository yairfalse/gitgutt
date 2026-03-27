use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Distribution {
    pub count: usize,
    pub median: Duration,
    pub p90: Duration,
    pub p99: Duration,
    pub min: Duration,
    pub max: Duration,
}

impl Distribution {
    pub fn compute(durations: &[chrono::Duration]) -> Option<Self> {
        if durations.is_empty() {
            return None;
        }

        let mut std_durations: Vec<Duration> = durations
            .iter()
            .filter_map(|d| d.to_std().ok())
            .collect();

        if std_durations.is_empty() {
            return None;
        }

        std_durations.sort();

        let count = std_durations.len();
        Some(Distribution {
            count,
            median: percentile(&std_durations, 50.0),
            p90: percentile(&std_durations, 90.0),
            p99: percentile(&std_durations, 99.0),
            min: std_durations[0],
            max: std_durations[count - 1],
        })
    }
}

fn percentile(sorted: &[Duration], p: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let minutes = total_secs / 60;
    let hours = total_secs / 3600;
    let days = total_secs / 86400;

    if days > 0 {
        format!("{}.{}d", days, (hours % 24) / 2)
    } else if hours > 0 {
        format!("{}.{}h", hours, (minutes % 60) / 6)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", total_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distribution_single_value() {
        let durations = vec![chrono::Duration::hours(2)];
        let dist = Distribution::compute(&durations).unwrap();
        assert_eq!(dist.count, 1);
        assert_eq!(dist.median, Duration::from_secs(7200));
    }

    #[test]
    fn distribution_multiple_values() {
        let durations: Vec<chrono::Duration> = (1..=10)
            .map(|h| chrono::Duration::hours(h))
            .collect();
        let dist = Distribution::compute(&durations).unwrap();
        assert_eq!(dist.count, 10);
        assert_eq!(dist.min, Duration::from_secs(3600));
        assert_eq!(dist.max, Duration::from_secs(36000));
    }

    #[test]
    fn distribution_empty() {
        let dist = Distribution::compute(&[]);
        assert!(dist.is_none());
    }

    #[test]
    fn format_duration_minutes() {
        assert_eq!(format_duration(Duration::from_secs(1500)), "25m");
    }

    #[test]
    fn format_duration_hours() {
        assert_eq!(format_duration(Duration::from_secs(15120)), "4.2h");
    }

    #[test]
    fn format_duration_days() {
        assert_eq!(format_duration(Duration::from_secs(90000)), "1.0d");
    }
}
