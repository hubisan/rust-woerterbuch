---
title: "Port Duden AMP Parser"
template_version: "0.2.0"
task_started: "[2026-06-09 Di]"
task_completed: "[2026-06-09 Di]"
---

# Context

- Branch: `feature/dudenParser`
- Focus: `src/sources/duden.rs`

# Goal

Replace the current Duden stub with a robust AMP HTML parser that follows the old Emacs Lisp logic, including homograph search fallback and deterministic offline snapshot tests.

# Result

The Duden source now follows the old AMP parser logic in Rust: direct entry lookup with `404 -> search` fallback, exact homograph filtering, recursive meaning parsing, page-level origin/synonym extraction, deterministic snapshot coverage for the existing fixture words, and parallel loading of homograph detail pages after the search step.

# Changes

- Linked this task file from `.project/tasks/todo.md`.
- Replaced the Duden stub in `src/sources/duden.rs` with an AMP-specific parser mapped onto the existing Rust models.
- Added exact-match Duden search parsing for homographs like `Bank` and stable `UrlValue::Many` handling.
- Changed Duden homograph detail fetches from serial to parallel while preserving stable entry order.
- Added offline snapshot coverage for `Bank`, `Haus`, `springen`, `verlieben`, `Wolke`, `Zaun`, and `Nixdaexistiert`.
- Added focused parser tests for exact homograph filtering and nested sense/example/idiom handling.

# Checks

- `cargo test duden` ✅
- `cargo test` ✅

# Out of scope

- CLI changes outside the Duden source module.

# Open points

- The Duden HTML fixtures are still referenced from the neighboring `../woerterbuch` test corpus instead of being copied into a repo-local `tests/fixtures/duden/` directory.
