---
title: "Format examples as heading instead of plain label"
template_version: "0.2.0"
task_started: 2026-06-14
task_completed: 2026-06-14
---

# Context

- Branch: `feature/format-examples-as-headings`
- Focus: Markdown/Org rendering in `src/format.rs`

# Goal

Render examples in Markdown and Org as headings instead of plain `Examples:` labels.

# Result

Markdown and Org now render examples as nested headings instead of plain `Examples:` labels, while human and JSON output stay unchanged.

# Changes

- Adjusted [src/format.rs](/home/hubisan/projects/coding/rust-woerterbuch/src/format.rs) so definition rendering knows the surrounding heading level for context-sensitive example headings.
- Rendered examples as `###/#### Examples` in Markdown and `*** / **** Examples` in Org depending on the current section depth.
- Kept human output on the existing plain `Examples:` label to avoid unrelated formatting changes.
- Added targeted renderer tests for Markdown and Org example-heading output and updated existing expectations.

# Checks

- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

# Out of scope

- Human output changes
- JSON output changes

# Open points

- None.
