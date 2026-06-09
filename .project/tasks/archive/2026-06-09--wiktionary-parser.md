---
title: "Port Wiktionary REST HTML Parser"
template_version: "0.2.0"
task_started: "[2026-06-09 Di]"
task_completed: "[2026-06-09 Di]"
---

# Context

- Branch: `feature/wiktionary-parser`
- Focus: `src/sources/wiktionary.rs`

# Goal

Finish the Rust Wiktionary parser against the German REST HTML API and cover it with deterministic local snapshot tests.

# Result

The Rust Wiktionary source now parses German REST HTML into structured entries, handles multiple homographs, maps labeled sections into senses/synonyms/idioms/etymology, and is covered by deterministic local fixture snapshot tests.

# Changes

- Linked this task file from `.project/tasks/todo.md`.
- Replaced the coarse Wiktionary parser with a REST-HTML-specific section parser.
- Added local REST HTML fixtures for `Bank`, `Haus`, `springen`, and `Wolke`.
- Added snapshot tests plus a 404/not-found test for `Nixdaexistiert`.

# Checks

- `cargo test wiktionary` ✅
- `cargo test` ✅

# Out of scope

- CLI aggregation changes outside the Wiktionary source module.

# Open points

- The parser currently keeps some Wiktionary formatting artifacts in long etymology prose, but the behavior is deterministic and covered by snapshots.

