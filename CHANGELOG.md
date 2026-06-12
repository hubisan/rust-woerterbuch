# Changelog

This file records completed changes.

## Unreleased

- `CHANGED` Updated the README.
- `CHANGED` Improved Duden lookup latency by returning direct Duden entries immediately instead of waiting for the search page; search is now used only as fallback for missing or unparseable direct entries, this is usually the case, when there are multiple homographs like for the word Bank.
- `FIX` Cargo.toml had unexepected string `-`, oups.
- `ADDED` Github CI for automated testing.
- `CHANGED` Renamed binary to `woerterbuch`.
- `CHANGED`: Clarified human-readable source status messages so skipped sections, missing entries, and real errors are shown differently.
- `FIXED`: Fixed `--sections` filtering so requested nested examples, idioms, and synonyms are preserved and unsupported source/section combinations are skipped cleanly.
- `ADDED`: Added structured parsers and offline snapshot tests for Duden, Wiktionary, OpenThesaurus, and DWDS.
- `CHANGED`: Fetch Duden entry and search pages in parallel so homographs can be detected earlier while treating search failures as non-fatal when the direct entry lookup succeeds.

## Instructions

### Changelog Entry Types

- `ADDED`: Use for new features, new functionality, or newly introduced behavior.
- `CHANGED`: Use for modifications to existing behavior.
- `FIXED`: Use for bug fixes or incorrect behavior that now works correctly.
- `REMOVED`: Use for removed functionality, deleted features, or deprecated behavior that no longer exists.

### Rules

- Use one bullet per change.
- Keep entries short and clear.
- Write for users, not developers.
- Do not include unfinished work.
- Do not include internal refactorings unless they affect users.
- Add newest entries at the top of the `Unreleased` section.
- Use uppercase entry types:
  - `ADDED`
  - `CHANGED`
  - `FIXED`
  - `REMOVED`
- Keep wording simple and direct.
- Prefer describing the visible result instead of implementation details.

### Releases

When creating a release:

- move entries from `Unreleased` into a version section
- keep the same bullet format
- use the following heading format:

```markdown
## 0.1.0 - 2026-05-22

- `ADDED`: Added CSV export for Anki notes via AnkiConnect.
- `CHANGED`: Clarified AI workflow and commit rules.
- `FIXED`: Fixed incorrect WAIT status handling.
- `REMOVED`: Removed outdated setup instructions.
```
