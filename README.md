<p align="center">
  <img src="https://img.shields.io/badge/git--rewind-v0.1.0-dc2626?style=for-the-badge&logo=git&logoColor=white" alt="Version"/>
  <img src="https://img.shields.io/badge/Rust-1.88+-f74c00?style=for-the-badge&logo=rust&logoColor=white" alt="Rust"/>
  <img src="https://img.shields.io/badge/ratatui-0.29-7e57c2?style=for-the-badge&logo=terminal&logoColor=white" alt="ratatui"/>
  <img src="https://img.shields.io/badge/git2-0.21-f05033?style=for-the-badge&logo=git&logoColor=white" alt="git2"/>
  <img src="https://img.shields.io/badge/TUI-Crossterm-1a73e8?style=for-the-badge&logo=windowsterminal&logoColor=white" alt="Crossterm"/>
  <img src="https://img.shields.io/badge/License-MIT-22c55e?style=for-the-badge" alt="License"/>
</p>

<h1 align="center">&#x23EA; git-rewind</h1>
<h3 align="center">Safe, Visual State Time-Travel for Your Git Repository</h3>

<p align="center">
  <i>A terminal-native UI that lets you browse your full Git reflog timeline, inspect commit diffs, and safely rewind your repository to any past state — with dirty-tree warnings and one-key confirmation.</i>
</p>

---

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [System Architecture](#system-architecture)
- [Data Flow — The Redux/Elm Loop](#data-flow)
- [Workspace Structure](#workspace-structure)
- [Crate Responsibilities](#crate-responsibilities)
- [Keybindings Reference](#keybindings-reference)
- [Reset Modes](#reset-modes)
- [Test Suite](#test-suite)
- [Quick Start Guide](#quick-start-guide)
- [Tech Stack](#tech-stack)
- [Roadmap](#roadmap)
- [License](#license)

---

## Overview

`git` has a powerful safety net built in — the **reflog**. Every commit, checkout, reset, and amend is logged. But navigating it requires memorising arcane commands like `git reset --hard HEAD@{3}` and hoping you picked the right entry.

**git-rewind** makes this safe, visual, and interactive.

It is a **full-stack terminal application** built entirely in Rust. Run it in any Git repository and you get an interactive multi-panel UI: your full reflog timeline on the left, commit metadata and changed files on the right, and a confirmation popup that checks for uncommitted changes before letting you travel anywhere.

| Component | Technology | Role |
|-----------|-----------|------|
| **Core** | `git-rewind-core` (pure Rust) | Domain models, reflog types, timeline projection |
| **Git Layer** | `git-rewind-git` + `git2` | Repository discovery, reflog reading, diff inspection, hard/mixed reset |
| **CLI** | `git-rewind-cli` + `clap` | Argument parsing, `version`, `doctor` subcommands, `AppService` orchestration |
| **TUI** | `git-rewind-ui` + `ratatui` + `crossterm` | Rendering, event loop, action dispatch, split-panel UI |

---

## Key Features

- **Live Reflog Timeline** — Loads the real Git reflog from your working repository on startup
- **Split Multi-Panel Layout** — Timeline (left), Commit Details + Changed Files (right), with a persistent footer shortcut guide
- **Commit Metadata Inspection** — Full commit ID, author name & email, timestamp, and commit message on selection
- **Changed Files List** — Color-coded by change type: `[A]` Added (green), `[M]` Modified (yellow), `[D]` Deleted (red), `[R]` Renamed (blue)
- **Dirty Working Tree Detection** — Checks for uncommitted modifications before allowing a rewind; shows a blocking warning popup
- **Hard & Mixed Reset** — Choose your reset strategy at confirmation time; hard discards changes, mixed preserves them in the working tree
- **Redux/Elm Architecture** — Clean separation: `Event → Action → Reducer → State → Render`; no business logic in the renderer
- **RAII Terminal Guard** — Alternate screen and raw mode are always cleaned up, even on panic
- **24 Passing Unit Tests** — Full coverage across all crates

---

## System Architecture

```
git-rewind-ui  (outermost — binary, TUI, I/O)
       |
       v
git-rewind-cli  (orchestration, AppService, CLI parsing)
       |
       v
git-rewind-git  (git2 wrapper, reflog reading, diff, reset)
       |
       v
git-rewind-core (pure domain types — no external I/O)
```

> **Every dependency points inward. No cycles. No inversion.**

### Component Graph

```
                  +-----------------------------+
                  |      git-rewind-ui          |
                  |  main.rs                    |
                  |  runtime/ (events, loop,    |
                  |            terminal RAII)   |
                  |  state/   (AppState,        |
                  |            Dialog, Selection|
                  |            TimelineState)   |
                  |  actions/ (mapper, reducer, |
                  |            Action enum)     |
                  |  render/  (Renderer,        |
                  |            layout, theme)   |
                  +-------------|---------------+
                                |  AppService
                  +-------------|---------------+
                  |    git-rewind-cli           |
                  |  AppService                 |
                  |  load_timeline()            |
                  |  inspect_commit()           |
                  |  inspect_diff()             |
                  |  reset_repository()         |
                  |  is_dirty()                 |
                  |  CLI parsing (clap)         |
                  +-------------|---------------+
                                |  RepositoryHandle
                  +-------------|---------------+
                  |    git-rewind-git           |
                  |  RepositoryHandle           |
                  |  discover() / discover_from |
                  |  ReflogMapper               |
                  |  CommitInspector            |
                  |  DiffInspector              |
                  |  reset(commit_id, hard)     |
                  |  is_dirty()                 |
                  +-------------|---------------+
                                |  domain types
                  +-------------|---------------+
                  |    git-rewind-core          |
                  |  CommitId, ReflogEntry      |
                  |  ReflogAction, ReflogIndex  |
                  |  TimelineItem               |
                  |  project(entries) — pure fn |
                  +-----------------------------+
```

---

## Data Flow

The TUI follows a strict **Redux/Elm architecture**. The renderer only reads state; it never mutates it.

```
Keyboard Input
      |
      v
runtime/events.rs  (crossterm Key --> Key enum)
      |
      v
actions/mapper.rs  map_event_to_action(event, &state)
      |
      +-- Dialog::None               --> Navigation / TriggerReset
      +-- ConfirmReset { dirty:true} --> y (proceed) / n (cancel)
      +-- ConfirmReset { dirty:false}--> h (hard) / m (mixed) / c (cancel)
      |
      v
actions/reducer.rs  reduce(&mut state, action) --> ReduceResult
      |
      +-- Continue              --> re-render frame
      +-- Quit                  --> restore terminal, exit
      +-- ResetRepository{...}  --> AppService::reset_repository()
                                    --> reload timeline
                                    --> re-render
      |
      v
render/renderer.rs  Renderer::render(frame, &state)
      |
      v
Terminal Frame
```

### TriggerReset Safety Flow

When the user presses `r` or `Enter`:

```
TriggerReset
      |
      v
AppService::is_dirty(repo)
      |
      +-- dirty  --> ShowConfirmReset { is_dirty: true }  (warning popup)
      +-- clean  --> ShowConfirmReset { is_dirty: false } (mode picker popup)
```

---

## Workspace Structure

```
git-rewind/
|
+-- Cargo.toml                          # Workspace manifest + shared dependencies
+-- rustfmt.toml                        # Code formatting config
|
+-- crates/
    |
    +-- git-rewind-core/                # Pure domain types (no I/O)
    |   +-- src/
    |       +-- reflog/
    |       |   +-- entry.rs            #   CommitId, ReflogEntry, ReflogTimestamp
    |       |   +-- action.rs           #   ReflogAction enum
    |       +-- timeline/
    |           +-- item.rs             #   TimelineItem (presentation model)
    |           +-- projector.rs        #   project(entries) -> Vec<TimelineItem>
    |
    +-- git-rewind-git/                 # git2 integration layer
    |   +-- src/
    |       +-- repository.rs           #   RepositoryHandle, discover(), reset(), is_dirty()
    |       +-- reflog/mapper.rs        #   git2 reflog --> domain ReflogEntry
    |       +-- commit/inspector.rs     #   CommitInspector: id, author, timestamp, message
    |       +-- commit/model.rs         #   CommitDetails, CommitAuthor
    |       +-- diff/inspector.rs       #   DiffInspector: tree-to-tree diff
    |       +-- diff/model.rs           #   CommitDiff, ChangedFile, FileChangeType
    |       +-- error.rs                #   GitError
    |
    +-- git-rewind-cli/                 # Orchestration & CLI parsing
    |   +-- src/
    |       +-- cli.rs                  #   Cli struct, Commands enum (clap derive)
    |       +-- commands/version.rs     #   `git-rewind version`
    |       +-- commands/doctor.rs      #   `git-rewind doctor`
    |       +-- app/service.rs          #   AppService (all git operations)
    |
    +-- git-rewind-ui/                  # TUI binary
        +-- src/
            +-- main.rs                 #   Entrypoint: CLI args or launch TUI
            +-- state/app.rs            #   AppState, Dialog enum
            +-- state/timeline.rs       #   TimelineState, selected commit details/diff
            +-- state/selection.rs      #   Selection (clamped index)
            +-- actions/action.rs       #   Action enum
            +-- actions/mapper.rs       #   map_event_to_action(event, &state)
            +-- actions/reducer.rs      #   reduce(&mut state, action) -> ReduceResult
            +-- runtime/application.rs  #   run() + run_with_events()
            +-- runtime/terminal.rs     #   TerminalGuard (RAII)
            +-- runtime/events.rs       #   Key enum, poll_event()
            +-- render/renderer.rs      #   Renderer::render(frame, &state)
            +-- render/layout.rs        #   compute(area) -> Layout
            +-- render/timeline.rs      #   Timeline list widget
            +-- render/theme.rs         #   DEFAULT_THEME
```

---

## Crate Responsibilities

### `git-rewind-core` — Pure Domain

Zero external dependencies. Contains only domain types and pure functions.

| Type | Purpose |
|------|---------|
| `CommitId` | Newtype wrapping a Git SHA string |
| `ReflogEntry` | A single reflog record: index, commit, action, message, timestamp |
| `ReflogAction` | `Commit`, `Reset`, `Checkout`, `Amend`, `Merge`, `Rebase`, `Unknown` |
| `ReflogIndex` | Newtype wrapping a reflog position (0 = HEAD) |
| `ReflogTimestamp` | Newtype wrapping `SystemTime` with `Display` |
| `TimelineItem` | Presentation-ready projection of a reflog entry |
| `project(entries)` | Pure function: `&[ReflogEntry] -> Vec<TimelineItem>` |

### `git-rewind-git` — Git Integration

Wraps `git2`. Nothing outside this crate touches `git2` types directly.

| Concern | Implementation |
|---------|---------------|
| Repository discovery | `discover()` (from CWD), `discover_from(path)` |
| Reflog reading | `ReflogMapper` — maps git2 reflog -> domain `ReflogEntry` |
| Commit inspection | `CommitInspector` — id, author name/email, timestamp, message |
| Diff inspection | `DiffInspector` — tree-to-tree diff -> `CommitDiff { files }` |
| Hard reset | `RepositoryHandle::reset(commit_id, hard: true)` |
| Mixed reset | `RepositoryHandle::reset(commit_id, hard: false)` |
| Dirty check | `RepositoryHandle::is_dirty()` — checks index + untracked files |

### `git-rewind-cli` — Orchestration

Bridges git layer to UI. No `git2` imports. No terminal I/O.

| Concern | Implementation |
|---------|---------------|
| CLI argument parsing | `Cli` / `Commands` (clap derive) |
| Timeline loading | `AppService::load_timeline(repo)` |
| Commit inspection | `AppService::inspect_commit(repo, commit_id)` |
| Diff inspection | `AppService::inspect_diff(repo, commit_id)` |
| Repository reset | `AppService::reset_repository(repo, commit_id, hard)` |
| Dirty state query | `AppService::is_dirty(repo)` |

### `git-rewind-ui` — TUI

Owns everything terminal: rendering, event polling, state, and the application loop.

| Concern | Implementation |
|---------|---------------|
| Application loop | `run_with_events(terminal, state, service, repo, next_event)` |
| Terminal lifecycle | `TerminalGuard` (RAII — always restores raw mode + alternate screen) |
| Event translation | `Key` enum; `poll_event(duration)` wraps crossterm |
| State | `AppState { timeline: TimelineState, dialog: Dialog }` |
| Dialog | `Dialog::None` / `Dialog::ConfirmReset { commit_index, is_dirty }` |
| Input mapping | `map_event_to_action(event, &state)` — context-sensitive |
| State reduction | `reduce(&mut state, action) -> ReduceResult` |
| Layout | `compute(area) -> Layout { header, timeline, details, files, footer }` |
| Rendering | `Renderer::render(frame, &state)` — stateless, reads only |

---

## Keybindings Reference

### Normal Mode

| Key | Action |
|-----|--------|
| `j` / `Down` | Select next commit in timeline |
| `k` / `Up` | Select previous commit in timeline |
| `g` / `Home` | Jump to first commit (HEAD) |
| `G` / `End` | Jump to last commit |
| `r` / `Enter` | Initiate rewind to selected commit |
| `q` | Quit |
| `Esc` | Clear error state |

### Dirty Warning Popup

| Key | Action |
|-----|--------|
| `y` | Proceed to reset mode picker |
| `n` / `Esc` / `c` | Cancel — stay where you are |

### Reset Mode Picker

| Key | Action |
|-----|--------|
| `h` | **Hard Reset** — rewind and discard all uncommitted changes |
| `m` | **Mixed Reset** — rewind and preserve changes in working tree |
| `c` / `Esc` | Cancel — return to timeline |

---

## Reset Modes

| Mode | Git Equivalent | What it Does |
|------|---------------|-------------|
| **Hard** | `git reset --hard <commit>` | Moves HEAD to the selected commit. Discards **all** staged and unstaged changes. Working tree matches that commit exactly. |
| **Mixed** | `git reset --mixed <commit>` | Moves HEAD to the selected commit. Preserves your file changes as unstaged modifications. Staged changes return to the working tree. |

> **Safety:** git-rewind always checks `is_dirty()` before allowing a rewind. If uncommitted changes exist, you must explicitly acknowledge the warning before choosing a reset mode.

---

## Test Suite

All 24 tests pass across the workspace:

```bash
cargo test --workspace
```

| Crate | Tests | What Is Covered |
|-------|:-----:|-----------------|
| `git-rewind-core` | 7 | ReflogEntry wrappers, projector (empty, single, multiple, order, summary trimming, unknown actions) |
| `git-rewind-git` | 7 | Repository discovery (non-existent, root, nested), commit inspection, diff (no changes, with changes) |
| `git-rewind-cli` | 4 | AppService: load_timeline, inspect_commit, inspect_diff, error propagation |
| `git-rewind-ui` | 6 | Selection invariants, timeline state, event translation, action mapping (context-sensitive), reducer bounds, dialog flow |
| **Total** | **24** | **All passing** |

### Notable: `test_dialog_reducer_and_mapper`

End-to-end validation of the full dialog confirmation flow:

```rust
// Step 1: Trigger reset -> dirty warning popup appears
reduce(&mut state, Action::ShowConfirmReset { is_dirty: true });
assert_eq!(state.dialog, Dialog::ConfirmReset { commit_index: 0, is_dirty: true });

// Step 2: While dirty popup is open, 'y' maps to "proceed"
assert_eq!(map_event_to_action(Key::Char('y'), &state),
           Some(Action::ShowConfirmReset { is_dirty: false }));

// Step 3: After acknowledging dirty warning, 'h' maps to hard reset
reduce(&mut state, Action::ShowConfirmReset { is_dirty: false });
assert_eq!(map_event_to_action(Key::Char('h'), &state),
           Some(Action::ConfirmResetSelectHard));

// Step 4: Executing hard reset returns ResetRepository result, clears dialog
assert_eq!(reduce(&mut state, Action::ConfirmResetSelectHard),
           ReduceResult::ResetRepository { commit_id: ..., hard: true });
assert_eq!(state.dialog, Dialog::None);
```

---

## Quick Start Guide

### Prerequisites

- **Rust 1.88+** — install via [rustup.rs](https://rustup.rs)
- A Git repository to browse

### 1. Clone

```bash
git clone https://github.com/Gagandeeprai/git-rewind.git
cd git-rewind
```

### 2. Build

```bash
cargo build --release
```

### 3. Run from Any Git Repository

```bash
cd /path/to/your/git/project
cargo run --bin git-rewind-ui
```

### 4. Subcommands

```bash
# Show version
git-rewind-ui version

# Run environment diagnostics
git-rewind-ui doctor

# Launch the interactive TUI (default)
git-rewind-ui
```

### 5. In the TUI

1. Use `j`/`k` or arrow keys to scroll the reflog timeline
2. The right panel automatically loads commit details and changed files for the selected entry
3. Press `r` or `Enter` to rewind to the highlighted commit
4. Follow the confirmation popup (dirty warning if needed, then Hard vs Mixed choice)
5. Press `q` to quit

---

## Tech Stack

| Layer | Technology | Version | Purpose |
|-------|-----------|---------|---------|
| **Language** | Rust | 1.88+ | Entire codebase |
| **TUI Framework** | ratatui | 0.29 | Widget rendering, layout, frame management |
| **Terminal Backend** | crossterm | 0.28 | Raw mode, alternate screen, key events |
| **Git Library** | git2 (libgit2) | 0.21 | Repository discovery, reflog, diff, reset |
| **CLI Parser** | clap | 4.5 (derive) | Subcommand parsing, help generation |
| **Error Handling** | anyhow + thiserror | 1.x | Error propagation and typed domain errors |
| **Workspace** | Cargo workspace | resolver = "2" | Unified dependency management |
| **Test Utilities** | tempfile | 3.x | Isolated temporary git repos in tests |

---

## Roadmap

- [ ] **Scrollable viewport** — `ListState` for large reflogs that exceed terminal height
- [ ] **Reflog search / filter** — Live-filter timeline by commit message or commit ID prefix
- [ ] **Branch awareness** — Show which branch each reflog entry belongs to
- [ ] **`git stash` integration** — Automatically stash dirty changes before a hard reset
- [ ] **Undo rewind** — Record pre-rewind HEAD; undo with one key
- [ ] **Mouse support** — Click to select, scroll wheel navigation
- [ ] **`git rewind` subcommand** — Ship as a proper `git` extension
- [ ] **Config file** — `~/.config/git-rewind/config.toml` for keybinding and theme customisation
- [ ] **Multiple themes** — Dark / light / high-contrast variants

---

## License

This project is licensed under the **MIT License**.

---

<p align="center">
  <b>Built with &#x23EA;&#x1F980; in Rust</b><br/>
  <i>Because <code>git reflog</code> deserves a better interface.</i>
</p>
