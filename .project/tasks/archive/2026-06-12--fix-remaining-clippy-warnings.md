---
title: "Fix remaining Clippy warnings"
template_version: "0.2.0"
task_started: 2026-06-12
task_completed: 2026-06-12
---

# Context

- Branch: `fix/clippy-warnings`
- Focus: `src/models.rs`, `src/sources/dwds.rs`

# Goal

Resolve the remaining Clippy warnings so `cargo clippy --all-targets --all-features -- -D warnings` passes again.

# Result

Identified the last two warnings and fixed them with minimal code movement and no behavior change. All preferred local checks now pass again.

# Changes

- Replaced the DWDS manual `iter().any(...)` class exclusion check with `contains(...)`.
- Moved `dedupe(...)` above the `models.rs` test module to satisfy Clippy's item ordering rule.
- Linked the active task from `.project/tasks/todo.md`.

# Checks

- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

# Out of scope

- Functional parser changes beyond the Clippy-driven cleanup.
- CHANGELOG update for this non-user-visible maintenance task.

# Open points

- None.
