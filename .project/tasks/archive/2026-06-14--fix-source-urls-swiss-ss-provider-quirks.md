---
title: "Fix source URLs, Swiss ss, sharp S candidates, and provider quirks"
template_version: "0.2.0"
task_started: 2026-06-14
task_completed: 2026-06-14
---

# Context

- Branch: `fix/url-quirks-and-sharp-s`
- Focus: source URL normalization, sharp-S fallback candidates, fixture helper reuse

# Goal

Implement source-specific URL handling and verified Swiss `ss` fallback behavior for Duden and Wiktionary without globally rewriting user queries.

# Result

Implemented source-specific URL helpers and verified Swiss `ss` fallback behavior. Duden now builds `sz`-based slugs with accent stripping and candidate URLs, Wiktionary prefers stronger `ß` lemmas over weak Swiss-spelling pages, and DWDS/OpenThesaurus keep user spelling without local `ss -> ß` rewrites.

# Changes

- Added shared [src/orthography.rs](/home/hubisan/projects/coding/rust-woerterbuch/src/orthography.rs) with deterministic sharp-S candidate generation and unit tests.
- Refactored [src/sources/duden.rs](/home/hubisan/projects/coding/rust-woerterbuch/src/sources/duden.rs) to use `ß -> sz`, strip non-German diacritics, try multiple direct-entry URL candidates, and keep search fallback/result matching stable.
- Refactored [src/sources/wiktionary.rs](/home/hubisan/projects/coding/rust-woerterbuch/src/sources/wiktionary.rs) to build MediaWiki titles with `_`, classify weak pages, and prefer verified main-lemma candidates.
- Added shared URL helpers and tests in [src/sources/dwds.rs](/home/hubisan/projects/coding/rust-woerterbuch/src/sources/dwds.rs) and [src/sources/openthesaurus.rs](/home/hubisan/projects/coding/rust-woerterbuch/src/sources/openthesaurus.rs), and reused helpers in [src/bin/refresh_fixtures.rs](/home/hubisan/projects/coding/rust-woerterbuch/src/bin/refresh_fixtures.rs).

# Checks

- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

# Out of scope

- Perfect German orthography conversion
- Full lexical disambiguation for every `ss`/`ß` ambiguity

# Open points

- No fixture refresh/download was run; existing local fixtures and expected JSON stayed compatible with the parser changes.
