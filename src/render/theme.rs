use colored::Colorize;

pub fn header(cmd: &str, repo: &str, duration: chrono::Duration) {
    let days = duration.num_days();
    let right = format!("{repo} \u{00b7} {days}d");
    let width: usize = 64;
    let pad = width.saturating_sub(cmd.len() + right.len());
    println!();
    println!("  {}{:>pad$}", cmd.bold().white(), right.bright_black(), pad = pad);
}

pub fn print_section(label: &str) {
    println!("  {}", label.bright_black().bold());
}

pub fn separator() {
    println!("  {}", "\u{2500}".repeat(62).bright_black());
}

pub fn accent(s: &str) -> String {
    s.bright_blue().bold().to_string()
}

pub fn accent_num(n: usize) -> String {
    format!("{:>3}", n).bright_blue().bold().to_string()
}

pub fn accent_num_f(n: f64, decimals: usize) -> String {
    format!("{n:.decimals$}").bright_blue().bold().to_string()
}

pub fn success_num(n: usize) -> String {
    format!("{:>3}", n).green().bold().to_string()
}

pub fn dim(s: &str) -> String {
    s.bright_black().to_string()
}

pub fn dim_num(n: usize) -> String {
    format!("{:>3}", n).bright_black().to_string()
}

pub fn warn(s: &str) -> String {
    s.yellow().to_string()
}

pub fn error_color(s: &str) -> String {
    s.red().to_string()
}
