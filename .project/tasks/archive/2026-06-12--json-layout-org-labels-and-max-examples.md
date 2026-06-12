---
title: "JSON layout, Org labels, and max examples"
template_version: "0.2.0"
task_started: 2026-06-12
task_completed: 2026-06-12
---

# Context

- Branch: `feature/multiple-output-formats`
- Focus: `src/main.rs`, `src/format.rs`, `README.md`, `.project/tasks/`

# Goal

Implement the next output-format follow-ups: use Org-mode code markup for labels, reject `--layout` for JSON output while keeping JSON source-native, and add `--max-examples`.

# Result

The Org label formatting, JSON layout validation, and per-definition example limiting are implemented and approved.

# Changes

- Changed Org rendering so definition labels and sense references use `~label~` instead of backticks.
- Made `--layout` optional in the CLI and reject explicit `--layout` for JSON output with a clear error.
- Removed the grouped JSON presentation variant; JSON now always renders the existing source-native response shape.
- Added `--max-examples <N>` for text-like output and applied the limit per definition or subsense during rendering only.
- Kept JSON unchanged when `--max-examples` is present.
- Updated README usage examples and output-option documentation.
- Added tests for Org label formatting, JSON layout rejection, layout defaults, and example limiting behavior.

# Checks

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

# Out of scope

- Duden Umlaut handling.
- Release checkpoint tasks.

# Open points

- None.
