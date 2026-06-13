---
title: "Add timestamps/headers to dictionary output formats"
template_version: "0.2.0"
task_started: "[2026-06-13 Sat]"
task_completed: "[2026-06-13 Sat]"
---

# Context

- Branch: `improvement/add-header-for-output-non-json`
- Focus: `src/format.rs`

# Goal

Add lookup metadata headers to human, markdown, org, and json output formats while preserving existing layout behavior and example handling rules.

# Result

All four output formats now include lookup metadata headers using one shared local timestamp per render call. JSON keeps all examples independent of `--max-examples`, while text formats still apply the example limit.
The follow-up review point about uppercasing Org header keywords was verified: the renderer already emits `#+TITLE` and `#+DATE` in uppercase.

# Changes

- Added `chrono` and created one shared `Local::now()` timestamp in `render(...)`.
- Prefixed human, markdown, and org output with format-specific title/date headers.
- Inserted a top-level RFC3339 `timestamp` field into JSON output without changing the remaining payload shape.
- Updated renderer tests to assert header structure and JSON timestamp behavior without depending on an exact timestamp value.
- Verified the Org header keywords remain uppercase as `#+TITLE` and `#+DATE`.

# Checks

- `cargo fmt --all --check`
- `cargo test --offline`
- `cargo clippy --offline --all-targets --all-features -- -D warnings`

# Out of scope

- Changelog update before user approval.
- Archiving or marking the task as done.

# Open points

- None.
