# gitgutt

```
  ┌─────────────────────────────────────┐
  │  how long does your team actually   │
  │  take to ship things?               │
  │                                     │
  │  gitgutt knows.                     │
  └─────────────────────────────────────┘
```

A fast, lean Rust CLI that pulls your GitHub PR data and tells you the truth about your engineering velocity. No dashboards to configure. No SaaS to sign up for. Just run it.

```
$ gitgutt --repo your-org/your-repo

  gitgutt                                    your-org/your-repo · 30d

  PULL REQUESTS           VELOCITY              MERGE TIME
  47  total               1.6 pr/day            4.2h median
  38  merged              ▁▂▃▅▇▅▃▂▁▃▅▇▆▃       18.6h p90
   6  open
   3  closed

  SIZE DISTRIBUTION                    REVIEW HEALTH
  XS  ████████████████  18             first review   1.2h median
  S   ██████████░░░░░░  12             reviews/pr     2.1 avg
  M   ████████░░░░░░░░   9
  L   ████░░░░░░░░░░░░   5             TOP REVIEWERS
  XL  ██░░░░░░░░░░░░░░   3             alice   ████████████  16
                                        bob     ████████░░░░  11
```

## Install

```bash
cargo install --git https://github.com/yairfalse/gitgutt
```

Or clone and build:

```bash
git clone https://github.com/yairfalse/gitgutt
cd gitgutt
cargo install --path .
```

## Usage

```bash
# run in any git repo with a GitHub remote — auto-detects everything
gitgutt

# or point at a specific repo
gitgutt --repo owner/repo

# deep dives
gitgutt prs                  # merge time distribution, slowest PRs
gitgutt authors              # who's shipping what
gitgutt review               # review health, top reviewers

# change the time window
gitgutt --days 90            # last 90 days instead of 30
```

## Auth

gitgutt needs a GitHub token. It checks (in order):

1. `GITHUB_TOKEN` env var
2. `gh auth token` (if you have [GitHub CLI](https://cli.github.com/) installed)

That's it. No config files.

## What it tells you

| Command | You learn |
|---------|-----------|
| `gitgutt` | The full picture — PR counts, velocity sparkline, merge time, size distribution, review health |
| `gitgutt prs` | Where time goes — merge time histogram, slowest PRs, cycle time trend |
| `gitgutt authors` | Who's doing what — PRs merged, reviews given, merge speed per person |
| `gitgutt review` | Review bottlenecks — time to first review, top reviewers, review load |

## Why

Because every team says "we ship fast" and nobody actually measures it. gitgutt gives you the numbers in 2 seconds flat, right in your terminal, with cute little bar charts.

No accounts. No dashboards. No meetings about dashboards.

Just `gitgutt` and the truth.

## Built with

Rust + [octocrab](https://github.com/XAMPPRocky/octocrab) + [clap](https://github.com/clap-rs/clap) + unicode sparklines + vibes

---

*Made by [False Systems](https://github.com/false-systems)*
