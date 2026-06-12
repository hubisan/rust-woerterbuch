---
title: "Clarify and fix section filtering semantics"
template_version: "0.2.0"
task_started: "[2026-06-11 Do]"
task_completed: "[2026-06-11 Do]"
---

# Context

- Branch: `fix/section-filtering`
- Focus: `src/models.rs`, `src/sources.rs`, section filtering semantics

# Goal

Make `--sections` a stable output projection that preserves requested nested content, and skip source requests only when a source cannot provide any requested section.

# Result

Implemented central section pruning that no longer deletes whole senses when only definitions are disabled. Added source capability checks so incompatible sources return empty successful results without performing lookups.

# Changes

- Reworked `DictionaryEntry::retain_sections` and recursive `Sense` pruning to preserve requested nested examples, idioms, and synonyms.
- Added unit tests for section combinations and empty-section behavior in `src/models.rs`.
- Added `source_supports_any_section` and early return behavior in `src/sources.rs`.
- Added source capability and skipped-source tests.

# Checks

- `cargo test` (passed)

# Out of scope

- Parser-internal section-aware scraping for Duden, DWDS, and Wiktionary
- JSON shape changes or removal of `--sections`

# Open points

- None currently.
