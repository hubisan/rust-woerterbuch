---
title: "Consistent status messages in human output"
template_version: "0.2.0"
task_started: "[2026-06-11 Do]"
task_completed: "[2026-06-12 Fr]"
---

# Context

- Branch: `fix/section-filtering`
- Focus: `src/format.rs`, source status wording in human output

# Goal

Make the human-readable CLI output distinguish clearly between skipped sources, missing entries on a source, and real request/parser errors.

# Result

Human-readable source output now distinguishes between unsupported section/source combinations, source-level misses, and real errors without changing the JSON model.

# Changes

- Replaced the generic `No results.` branch in `src/format.rs` with explicit status messages.
- Normalized `No matches found` errors in human output to `No entry found on source.`
- Render skipped sources as `Skipped: source does not support requested sections.`
- Render empty filtered results as `No content for requested sections.`
- Added formatter unit tests for all status branches.

# Checks

- `cargo test` (passed)

# Out of scope

- JSON schema changes
- Source parser refactors unrelated to display wording

# Open points

- None currently.
