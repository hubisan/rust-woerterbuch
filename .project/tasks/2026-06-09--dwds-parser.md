---
title: "Port DWDS HTML Parser"
template_version: "0.2.0"
task_started: "[2026-06-09 Di]"
task_completed:
---

# Context

- Branch: `feature/dwds-parser`
- Focus: `src/sources/dwds.rs`

# Goal

Replace the current DWDS stub with a structured HTML parser that follows the old Emacs Lisp logic, including homographs, nested senses, etymology, idioms, and deterministic offline snapshot tests.

# Result

Completed and ready for review.

# Changes

- Replaced the DWDS source stub in `src/sources/dwds.rs` with a structured HTML parser.
- Ported the DWDS homograph, sense tree, qualifier, MWA, idiom, etymology, canonical URL, and no-match handling into Rust.
- Added deterministic offline snapshot coverage for `Bank`, `Haus`, `springen`, `verlieben`, `Wolke`, `Zaun`, and `Nixdaexistiert`.

# Checks

- `cargo test dwds`
- `cargo test`

# Out of scope

- CLI changes outside the DWDS source module.

# Open points

- None currently.
