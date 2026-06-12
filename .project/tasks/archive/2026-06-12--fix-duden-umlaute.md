---
title: "Fix Duden Umlaute"
template_version: "0.2.0"
task_started: 2026-06-12
task_completed: 2026-06-13
---

# Context

- Branch: `fix/duden-umlaute-in-url`
- Focus: `src/sources/duden.rs`, `.project/tasks/`

# Goal

Make Duden lookups work for words whose Duden URL uses `ae/oe/ue/ss` spellings instead of literal umlauts or `ß`.

# Result

Duden lookups now normalize Umlaut and `ß` spellings to the URL form Duden expects, and the search fallback treats `Gerüst` and `Geruest` as equivalent.

# Changes

- Changed the Duden direct-entry URL builder to map `ä/ö/ü/ß` to `ae/oe/ue/ss` before URL encoding.
- Reused the same normalization for Duden search-result matching so expanded spellings and literal umlauts match each other.
- Added tests for `Gerüst`, `verrückt`, `Straße`, and search-result equivalence between `Gerüst` and `Geruest`.

# Checks

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

# Out of scope

- Fixture refresh for live Duden responses unless needed for the parser.
- Release checkpoint tasks.

# Open points

- None.
