use colored::Colorize;

const SPARK_CHARS: &[char] = &['\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}'];
const BAR_FULL: char = '\u{2588}';
const BAR_EMPTY: char = '\u{2591}';

pub fn sparkline(values: &[u32]) -> String {
    if values.is_empty() {
        return String::new();
    }

    let max = *values.iter().max().unwrap_or(&1) as f64;
    let max = if max == 0.0 { 1.0 } else { max };

    let spark: String = values
        .iter()
        .map(|&v| {
            let idx = ((v as f64 / max) * (SPARK_CHARS.len() - 1) as f64).round() as usize;
            SPARK_CHARS[idx.min(SPARK_CHARS.len() - 1)]
        })
        .collect();

    spark.bright_blue().to_string()
}

pub fn bar(value: usize, max: usize, width: usize) -> String {
    let max = max.max(1);
    let filled = ((value as f64 / max as f64) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    let filled_str: String = std::iter::repeat(BAR_FULL).take(filled).collect();
    let empty_str: String = std::iter::repeat(BAR_EMPTY).take(empty).collect();

    format!(
        "{}{}",
        filled_str.bright_blue(),
        empty_str.bright_black()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparkline_empty() {
        assert_eq!(sparkline(&[]), "");
    }

    #[test]
    fn sparkline_single() {
        let result = sparkline(&[5]);
        // Should contain the highest spark char
        assert!(!result.is_empty());
    }

    #[test]
    fn bar_full() {
        // Can't easily test colored output, but ensure no panic
        let _ = bar(10, 10, 20);
    }

    #[test]
    fn bar_zero() {
        let _ = bar(0, 10, 20);
    }

    #[test]
    fn bar_max_zero() {
        let _ = bar(0, 0, 20);
    }
}
