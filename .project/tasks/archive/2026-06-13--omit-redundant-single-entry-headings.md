---
title: "Omit redundant single-entry headings"
template_version: "0.2.0"
task_started: 2026-06-13
task_completed: 2026-06-13
---

# Context

- Branch: `feature/omit-redundant-single-entry-headings`
- Focus: `src/format.rs`, `.project/tasks/`

# Goal

Omit redundant `Entry 1` headings in text rendering when a source has exactly one entry.

# Result

Text rendering now omits redundant `Entry 1` headings for single-entry sources while preserving entry headings for sources with multiple entries.

# Changes

- Omitted `Entry N` headings in text output when the current source has exactly one entry.
- Kept `Entry N` headings for multi-entry sources in both `by-source` and `by-section` layouts.
- Adjusted by-source section heading levels so single-entry sources render sections directly under the source heading.
- Added coverage for single-entry and multi-entry behavior in both Markdown layouts.

# Checks

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

# Out of scope

- JSON output changes.
- Release checkpoint tasks.

# Open points

- None.
