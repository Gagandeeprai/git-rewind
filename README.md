# git-rewind

Interactive Git reflog explorer — browse, inspect, and travel through your repository history.

## Crates

| Crate | Description |
|---|---|
| `git-rewind-cli` | CLI entry point & application orchestration |
| `git-rewind-core` | Domain models (reflog entries, timeline) |
| `git-rewind-git` | Git backend layer (git2 bindings) |
| `git-rewind-ui` | TUI frontend (ratatui + crossterm) |

## Usage

```bash
cargo run --bin git-rewind-ui
```

Keys: `j/k` or `Up/Down` to navigate, `r/Enter` to travel, `q/Esc` to quit.
