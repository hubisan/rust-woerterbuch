---
title: "Output layout naming and spacing"
template_version: "0.2.0"
task_started: 2026-06-12
task_completed: 2026-06-12
---

# Context

- Branch: `feature/multiple-output-formats`
- Focus: `src/main.rs`, `src/format.rs`, `README.md`, `.project/tasks/`

# Goal

Finish the open output-format follow-up tasks: remove compiler warnings, rename layout variants to `by-source` and `by-section`, keep `by-source` as the default, and align rendered spacing with the Markdown examples.

# Result

The remaining output-format follow-ups are implemented and approved.

# Changes

- Renamed the public layout names from `sources-sections`/`sections-sources` to `by-source`/`by-section` in the CLI and README examples.
- Kept `by-source` as the default CLI layout.
- Reworked text rendering so `by-source` now renders `source -> entry -> section`, matching the Markdown examples more closely.
- Added entry metadata rendering for `Title`, `Part of speech`, and `Grammar` in `by-source` text output when those fields are available.
- Tightened blank-line handling for Markdown, Org, and human-readable output so headings, paragraphs, example blocks, and lists are separated as shown in the example files.
- Removed the unused `print_human` function and added renderer tests for layout names and representative Markdown spacing.
- Added renamed example reference files: `example-output-markdown--by-source.md` and `example-output-markdown--by-section.md`.

# Checks

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

# Out of scope

- JSON layout validation rules from the later `TODO JSON should not accept layout` task.
- Example limiting via `--max-examples`.

# Open points

- None.
