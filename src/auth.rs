use anyhow::{Context, Result};
use std::process::Command;

pub fn resolve_token() -> Result<String> {
    // 1. Check GITHUB_TOKEN env var
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            return Ok(token);
        }
    }

    // 2. Try gh CLI
    let output = Command::new("gh")
        .args(["auth", "token"])
        .output()
        .context("failed to run `gh auth token` — install gh CLI or set GITHUB_TOKEN")?;

    if output.status.success() {
        let token = String::from_utf8(output.stdout)
            .context("gh auth token output was not valid UTF-8")?
            .trim()
            .to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    anyhow::bail!(
        "no GitHub token found.\n\
         Set GITHUB_TOKEN env var or run `gh auth login`."
    )
}

pub fn detect_repo() -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("failed to detect git remote — are you in a git repo?")?;

    if !output.status.success() {
        anyhow::bail!("no git remote found. Use --repo owner/name to specify.");
    }

    let url = String::from_utf8(output.stdout)
        .context("git remote URL was not valid UTF-8")?
        .trim()
        .to_string();

    parse_github_repo(&url)
}

fn parse_github_repo(url: &str) -> Result<String> {
    // Handle SSH: git@github.com:owner/repo.git
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        let repo = rest.trim_end_matches(".git");
        return Ok(repo.to_string());
    }

    // Handle HTTPS: https://github.com/owner/repo.git
    if let Some(rest) = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
    {
        let repo = rest.trim_end_matches(".git");
        return Ok(repo.to_string());
    }

    anyhow::bail!("could not parse GitHub repo from remote URL: {url}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ssh_url() {
        let repo = parse_github_repo("git@github.com:acme/backend.git").unwrap();
        assert_eq!(repo, "acme/backend");
    }

    #[test]
    fn parse_https_url() {
        let repo = parse_github_repo("https://github.com/acme/backend.git").unwrap();
        assert_eq!(repo, "acme/backend");
    }

    #[test]
    fn parse_https_no_git_suffix() {
        let repo = parse_github_repo("https://github.com/acme/backend").unwrap();
        assert_eq!(repo, "acme/backend");
    }

    #[test]
    fn parse_unknown_url_fails() {
        let result = parse_github_repo("https://gitlab.com/acme/backend.git");
        assert!(result.is_err());
    }
}
