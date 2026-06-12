---
title: "Human format single quotes"
template_version: "0.2.0"
task_started: 2026-06-13
task_completed: 2026-06-13
---

# Context

- Branch: `feature/omit-redundant-single-entry-headings`
- Focus: `src/format.rs`, `.project/tasks/`

# Goal

Render labels in human output as `'nr'` instead of `` `nr` ``.

# Result

Human output now renders labels and references with single quotes instead of backticks.

# Changes

- Changed human-format labels from `` `nr` `` to `'nr'`.
- Changed human-format sense references from `` `nr`: ... `` to `'nr': ...`.
- Added tests for human labels and human references while keeping Markdown and Org formatting unchanged.

# Checks

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

# Out of scope

- Markdown and Org label formatting.
- Release checkpoint tasks.

# Open points

- None.
