# rust-woerterbuch

A small async Rust CLI for German dictionary lookups across OpenThesaurus, DWDS, Duden, and Wiktionary.

## Goal

`rust-woerterbuch` queries multiple German-language dictionary sources in parallel and returns structured lookup results. Output can be printed in a human-readable terminal format or as JSON for Emacs, scripts, and AI agents.

The tool deliberately sends the user query directly to each backend. There is no lemma normalization step in the CLI, because it adds complexity and can produce surprising source-specific behavior.

Planned sources:

- OpenThesaurus
- DWDS
- Duden, preferably via the lightweight AMP page
- Wiktionary, via the REST HTML API: `https://de.wiktionary.org/api/rest_v1/page/html/{word}`

Default source order:

```text
openthesaurus,dwds,duden,wiktionary
```

## Usage

```bash
cargo run -- Bank
cargo run -- Bank --json
cargo run -- Bank --sources wiktionary,openthesaurus
cargo run -- Bank --sections definitions,synonyms
```

## JSON shape

The JSON output is intentionally designed as a native API-style structure, not as a direct copy of the Emacs Lisp plist format.

```json
{
  "query": "Bank",
  "results": [
    {
      "source": "dwds",
      "ok": true,
      "url": "https://www.dwds.de/wb/Bank",
      "entries": [
        {
          "id": 1,
          "homograph": "1",
          "headword": "Bank",
          "title": "Bank, die",
          "part_of_speech": "Substantiv",
          "grammar": "Substantiv (Femininum)",
          "etymology": "...",
          "idioms": ["durch die Bank"],
          "synonym_groups": [],
          "senses": [
            {
              "id": 1,
              "source_id": "d-1-1",
              "label": "1.",
              "definition": "Sitz fuer mehrere Personen nebeneinander, meist aus Holz",
              "qualifiers": [],
              "examples": [],
              "idioms": [],
              "synonyms": [],
              "subsenses": []
            }
          ]
        }
      ]
    }
  ]
}
```

Definitions are recursive through `subsenses`, so nested meanings from DWDS, Duden, and Wiktionary can be represented without flattening.

## Project structure

```text
src/
  main.rs                 CLI, parallel execution, output handling
  models.rs               JSON-native lookup data structures
  http.rs                 reqwest client, User-Agent setup, HTML helper
  format.rs               Human-readable terminal output
  sources.rs              Source routing, timeouts, and section filtering
  sources/
    duden.rs              Duden fetch and parser scaffold
    dwds.rs               DWDS fetch and parser scaffold
    wiktionary.rs         Wiktionary REST HTML fetch and parser scaffold
    openthesaurus.rs      OpenThesaurus fetch and parser scaffold
```

## Next steps

1. Copy the existing Emacs Lisp backend files into the project directory for reference.
2. Refine the CSS selectors for each source based on the old backend logic and the current HTML output.
3. Add local HTML fixtures and parser tests before optimizing live requests.
4. Port each backend one by one, starting with OpenThesaurus or DWDS.
5. Consider adding rate limiting and caching once the basic parser logic is stable.

## License

This project is licensed under the GNU General Public License v3.0. See [`LICENSE`](LICENSE) for details.

## Notes

The parsers are intentionally defensive and template-oriented. Websites can change their HTML structure, so source-specific selector adjustments should stay isolated in the corresponding modules.
