---
title: "Refresh source fixtures and expected JSON output"
template_version: "0.2.0"
task_started: 2026-06-12
task_completed: 2026-06-12
---

# Context

- Branch: `fixture/source-test-fixtures`
- Focus: source fixtures, parser regression tests, expected JSON outputs

# Goal

Store repo-local raw source fixtures for the existing test words and generate canonical expected JSON outputs from the current Rust implementation so tests no longer depend on live websites or deleted external fixture directories.

# Result

Repo-local raw fixtures now exist for all current source test words across Duden, DWDS, Wiktionary, and OpenThesaurus. Expected JSON outputs are generated from the current Rust parser implementation, and the source tests compare against those local JSON files instead of deleted external assets and ad hoc text snapshots.

# Changes

- Added a small Rust helper binary at `src/bin/refresh_fixtures.rs` to download current source fixtures and render expected JSON from the saved files.
- Added `src/lib.rs` so the refresh binary can reuse the real parser and model modules instead of duplicating parsing logic.
- Stored durable raw fixtures under `tests/fixtures/{source}/{word}/...`, including multi-page Duden `Bank` search and homograph pages.
- Stored canonical expected parser outputs under `tests/expected/{source}/{word}.json`.
- Updated source parser tests to consume the new local fixtures and expected JSON files.

# Checks

- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

# Out of scope

- Parser behavior changes unrelated to fixture portability.
- Broader CLI feature work.

# Open points

- Legacy `tests/snapshots/**` files and old top-level Wiktionary fixture files are no longer used by the active tests, but were left in place for now.
