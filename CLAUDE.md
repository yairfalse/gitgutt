# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**gitgutt** — a Rust CLI tool that analyzes GitHub PR metrics for engineering teams. Fetches from GitHub API, computes metrics, renders to terminal. No database, no state, no config files.

## Build & Run

```bash
cargo build
cargo run -- stats                    # dashboard (default)
cargo run -- prs --days 30            # PR deep dive
cargo run -- authors --days 30        # per-author breakdown
cargo run -- review --days 30         # review health
cargo test                            # all tests
cargo test domain::                   # domain-only tests (pure, no network)
```

Auth: `GITHUB_TOKEN` env var or auto-detected from `gh auth token`. Repo: auto-detected from git remote, overridable with `--repo owner/name`.

## Architecture

**Functional Core, Imperative Shell** — domain logic is pure computation with zero I/O.

```
CLI (parse) → Auth (resolve token) → GitHub (fetch) → Domain (compute) → Render (display)
   shell           shell                shell            CORE              shell
```

### Module Map

- `src/cli.rs` — clap argument parsing, command routing
- `src/auth.rs` — token resolution (env var → gh CLI fallback)
- `src/github.rs` — octocrab client, pagination, translates API types → domain types (anticorruption layer)
- `src/domain/types.rs` — PullRequest, Review, Period, PrState, ReviewState (all value objects)
- `src/domain/metrics.rs` — pure functions: `compute_pr_metrics`, `compute_author_stats`, `compute_review_metrics`
- `src/domain/stats.rs` — Distribution, percentile math
- `src/render/charts.rs` — unicode bar charts (█░), sparklines (▁▂▃▅▇)
- `src/render/tables.rs` — tabled formatting
- `src/render/theme.rs` — Nordic color palette, box-drawing chars, spacing

### Key Constraints

- Domain types are decoupled from GitHub API types — `github.rs` does the translation
- Domain functions take `&[PullRequest]` and return metric structs — no async, no Result, no I/O
- All durations use `std::time::Duration`, never raw f64
- PR sizes use semantic buckets: XS/S/M/L/XL
- Single crate, not a workspace — not enough code to justify it

### Terminal Rendering

Nordic minimal aesthetic: one accent color (blue), green/yellow/red only for semantic meaning, ALL CAPS dim labels, no emoji, no outer borders. Information hierarchy: big numbers → bars → details → text.
