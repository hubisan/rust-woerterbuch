# TODOs

This file is inspired by Org mode. Task headings may use these TODO keywords:
`TODO`, `NEXT`, `WAIT`, `REVIEW`, `CONTINUE`, `DONE`, `CANCEL`.

This repository uses AI agents to assist with development.

Important files:

- [../agents/AGENTS.md](../agents/AGENTS.md): AI agent instructions and workflow rules
- [./todo.md](./todo.md): Active task index & statuses.
- [../agents/ai-notes.md](../agents/ai-notes.md): Cross-task context, notes, blockers, and decisions.
- [../../CHANGELOG.md](../../CHANGELOG.md): Approved/completed user-visible changes.

# Ziel des Projektes

Das bestehende Emacs-Lisp-Package, welches Wörterbuch-Daten von vier verschiedenen Quellen — Duden, DWDS, Wiktionary und OpenThesaurus — via Scraping aggregiert, wird in ein performantes, asynchrones Rust-CLI-Tool umgewandelt.

## Warum dieser Umstieg?

1. **AI-Ready:** Externe KI-Agenten und LLMs können das Tool nahtlos als CLI-Befehl aufrufen und strukturierte JSON-Daten auslesen.
2. **Performance & Concurrency:** Die vier Quellen werden in Rust dank `tokio` absolut parallel abgefragt und geparst, ohne die Single-Threaded Event-Loop von Emacs zu belasten.
3. **Schlanke Architektur:** Emacs dient am Ende nur noch als UI-Frontend, das die JSON-Ausgabe des Rust-CLI einliest und formatiert.
4. **Neues lernen:** Ein perfektes Einstiegsprojekt, um Rusts modernste Konzepte — Ownership, Pattern Matching und Async — in einem realen Szenario anzuwenden.

## Strategische Design-Entscheidungen für Stabilität & Fairness

- **Kein Lemmatizer zum Start:** Um das Projekt maximal schlank zu halten und eine Überkomplexität beim Einstieg zu vermeiden, wird bewusst auf einen separaten Lemmatizer verzichtet. Die Suchanfragen gehen direkt an die jeweiligen Backends.
- **Unauffälliger User-Agent:** Ein Standard-Browser-Header verhindert Blockierungen und sorgt für stabilen Traffic bei den Live-Abfragen.
- **Optimierte Datenquellen für Duden & Wiktionary:** Für den Duden wird die extrem schlanke AMP-Seite genutzt. Für Wiktionary wird auf die offizielle REST-HTML-API umgestellt.

### Architektur-Learning: Wiktionary REST-API

In der ursprünglichen Emacs-Lisp-Realisation bestand noch keine Kenntnis über die Existenz der offiziellen Wikimedia-REST-HTML-API unter `/api/rest_v1/page/html/`. Damals wurde das gesamte, überladene Web-HTML gescraped.

Der Wechsel auf den REST-Endpunkt liefert nun das nackte Inhalts-HTML komplett ohne Wikipedia-Design-Ballast — keine Sidebars, Navigationsleisten oder Skripte.

**Achtung bei Homonymen:** Da die API den gesamten Artikeltext liefert, sind bei Wörtern mit mehreren Bedeutungen, zum Beispiel `Bank`, auch mehrere Wortart-Blöcke — Substantiv, feminin, Bänke vs. Banken — untereinander auf derselben Seite enthalten. Der Parser muss darauf ausgelegt sein, alle Sektionen via Schleife beziehungsweise Vektor zu erfassen.

## Schritt 2: Die Parser einzeln übersetzen — Der Kern

Jetzt übersetzen wir die Scraping-Logik der Backends nacheinander von Elisp nach Rust. Wir starten mit OpenThesaurus.

# Abgeschlossen

## DONE Wiktionary: REST-HTML-Parser + Snapshot-Tests [[./2026-06-09--wiktionary-parser.md]]

Implementiere den Wiktionary-Parser in Rust fertig und ergänze Snapshot-Tests analog zur bestehenden OpenThesaurus-Implementierung.

Der aktuelle Rust-Code enthält bereits ein Wiktionary-Modul:

* [../../src/sources/wiktionary.rs](../../src/sources/wiktionary.rs)

Dieses Modul nutzt bereits den richtigen REST-Endpunkt:

```text
https://de.wiktionary.org/api/rest_v1/page/html/{wort}
```

Der vorhandene Parser ist aber noch zu grob und soll anhand der alten Emacs-Lisp-Logik sauber fertiggestellt werden.

### Relevante Referenzen

Alte Emacs-Lisp-Implementierung:

* [../../emacs-lisp/lisp/woerterbuch-wiktionary.el](../../emacs-lisp/lisp/woerterbuch-wiktionary.el)

Alte Wiktionary-Testfixtures und erwartete Outputs:

* [../../emacs-lisp/tests/files/wiktionary/](../../emacs-lisp/tests/files/wiktionary/)
* [../../emacs-lisp/tests/test-woerterbuch-wiktionary.el](../../emacs-lisp/tests/test-woerterbuch-wiktionary.el)

Bestehendes Rust-Muster für Snapshot-Tests:

* [../../src/sources/openthesaurus.rs](../../src/sources/openthesaurus.rs)
* [../../tests/snapshots/openthesaurus/](../../tests/snapshots/openthesaurus/)

Gemeinsame Datenstruktur:

* [../../src/models.rs](../../src/models.rs)

### Ziel

Der Wiktionary-Parser soll das REST-HTML der deutschen Wiktionary-API parsen und in die vorhandenen Rust-Modelle überführen:

* `SourceResult`
* `DictionaryEntry`
* `Sense`
* `SynonymGroup`

Die normale Wiktionary-Webseite soll nicht mehr live gescraped werden. Live-Lookups müssen über den REST-HTML-Endpunkt laufen. Die menschenlesbare URL im Ergebnis darf weiterhin die normale Wiktionary-Seite sein, also zum Beispiel:

```text
https://de.wiktionary.org/wiki/Bank
```

### Wichtig: REST-HTML statt altes Webseiten-HTML

Die alten Fixtures unter `emacs-lisp/tests/files/wiktionary/` stammen noch aus der alten Emacs-Lisp-Welt und enthalten die normale Wiktionary-Webseite mit Seitenlayout.

Für die neue Rust-Implementierung sollen Snapshot-Tests dagegen mit gespeicherten REST-HTML-Fixtures arbeiten, also mit HTML, das von diesem Endpunkt stammt:

```text
https://de.wiktionary.org/api/rest_v1/page/html/{wort}
```

Bitte lege dafür neue lokale Fixtures passend zur Rust-Teststruktur an, zum Beispiel unter:

```text
tests/fixtures/wiktionary/
```

oder, falls im Projekt ein anderer Fixture-Ort naheliegender ist, konsistent dort.

Die Tests dürfen nicht gegen die Live-API laufen.

### Inhaltliche Parser-Logik

Orientiere dich fachlich an `woerterbuch-wiktionary.el`, aber schreibe den Rust-Code idiomatisch und passend zur vorhandenen Architektur.

Aus dem Elisp-Code sind insbesondere diese Punkte relevant:

* Wiktionary-Artikel enthalten einen deutschen Sprachbereich.
* Innerhalb davon gibt es mehrere Einträge beziehungsweise Homographen.
* Einträge werden über Überschriften wie `Substantiv, f, Bänke`, `Substantiv, f, Banken`, `Verb`, `Adjektiv` usw. erkannt.
* Labeled Blocks wie `Bedeutungen:`, `Beispiele:`, `Synonyme:`, `Sinnverwandte Wörter:`, `Redewendungen:` und `Herkunft:` müssen erkannt werden.
* Bedeutungen und Beispiele sind über Sense-Labels wie `[1]`, `[2]`, `[1, 2]` oder `[1–3]` miteinander verbunden.
* Synonyme sollen pro Bedeutung gruppiert werden.
* Redewendungen sollen als eigene Liste am Eintrag landen.
* Herkunft soll als `etymology` gespeichert werden.
* Fußnoten, Bearbeiten-Links, Referenzmarker und sonstige Wiktionary-Artefakte sollen aus dem Text entfernt werden.

### Homonyme / mehrere Einträge

Der Parser darf nicht nur den ersten gefundenen Wiktionary-Block auslesen.

Beispiel `Bank`:

* `Bank, Substantiv, f, Bänke`
* `Bank, Substantiv, f, Banken`

Diese müssen als zwei getrennte `DictionaryEntry`-Werte im `entries`-Vektor erscheinen.

Die IDs müssen stabil und deterministisch sein:

```text
entry 1
entry 2
entry 3
...
```

### Mapping auf die Rust-Modelle

Bitte nutze die vorhandenen Felder aus `src/models.rs`:

```rust
DictionaryEntry {
    id,
    headword,
    title,
    part_of_speech,
    grammar,
    etymology,
    idioms,
    synonym_groups,
    url,
    senses,
    ..
}
```

Vorgeschlagenes Mapping:

* Wiktionary-Lemma → `headword`
* komplette Wiktionary-Überschrift → `title`
* Wortart aus der Überschrift, zum Beispiel `Substantiv` → `part_of_speech`
* sinnvoller Grammatiktext aus der Überschrift → `grammar`
* `Herkunft` → `etymology`
* `Redewendungen` → `idioms`
* `Synonyme` und `Sinnverwandte Wörter` → `synonym_groups`
* `Bedeutungen` → `senses[].definition`
* Sense-Label `[1]` → `senses[].label`
* `Beispiele` → `senses[].examples`

Wenn eine Sektion fehlt, soll der Parser kein Fehlerergebnis erzeugen, sondern einfach leere Felder verwenden.

Wenn gar kein sinnvoller Eintrag gefunden wird, soll Wiktionary analog zu OpenThesaurus ein sauberes Fehlerergebnis liefern, zum Beispiel:

```text
ok=false
error=No matches found
```

### Snapshot-Tests

Ergänze Snapshot-Tests nach dem Muster von OpenThesaurus in `src/sources/openthesaurus.rs`.

Bitte erstelle für Wiktionary ebenfalls:

```text
tests/snapshots/wiktionary/
```

und teste lokale Fixtures deterministisch gegen gerenderte Snapshots.

Mindestens diese Wörter sollen abgedeckt werden:

* `Bank` — Homonym-Test mit mehreren Einträgen
* `Haus` — umfangreicher Substantiv-Test
* `springen` — Verb-Test
* `Wolke` oder `Zaun` — einfacher Substantiv-Test
* `Nixdaexistiert` — kein Treffer / Fehlerfall, sofern sinnvoll als Fixture darstellbar

Die Snapshot-Ausgabe soll bewusst textuell und gut diffbar sein, ähnlich wie bei OpenThesaurus:

```text
source=Wiktionary
ok=true
url=https://de.wiktionary.org/wiki/Bank
entry 1 headword=Bank title=Bank, Substantiv, f, Bänke part_of_speech=Substantiv grammar=Substantiv, f, Bänke
sense 1 label=1 definition=Sitz- oder Ablagegelegenheit ...
sense 2 label=2 definition=geologische Formation
synonyms 2 items=[Lage]
idioms=[...]
etymology=...
entry 2 headword=Bank title=Bank, Substantiv, f, Banken part_of_speech=Substantiv grammar=Substantiv, f, Banken
sense 1 label=1 definition=Finanzwesen: Geldinstitut ...
synonyms 1 items=[Geldhaus, Geldinstitut, Finanzinstitut, ...]
```

Die genaue Snapshot-Formatierung darf an die Implementierung angepasst werden, soll aber stabil, lesbar und review-freundlich bleiben.

### Zusätzliche gezielte Unit-Tests

Neben den Snapshot-Tests bitte ein paar kleine Unit-Tests für Parser-Hilfsfunktionen ergänzen, insbesondere für:

* Sense-Label parsing: `[1]`, `[1, 2]`, `[1–3]`
* Textbereinigung von Fußnoten und Wiktionary-Artefakten
* Zuordnung von Beispielen zu Bedeutungen
* Synonymgruppen pro Bedeutung
* mehrere Homographen in einem Artikel

Diese Tests können klein und künstlich sein, ähnlich dem bestehenden Elisp-Test `parses list-based blocks without dl/dd wrappers`.

### Umsetzungshinweise

Bitte halte dich an die vorhandene Architektur:

* `lookup(client, query)` bleibt für HTTP zuständig.
* `parse(query, page_url, html)` bleibt unabhängig vom HTTP-Code testbar.
* Keine Live-Requests in Tests.
* Keine Panics bei fehlenden oder unerwarteten HTML-Sektionen.
* Möglichst keine großen Refactorings außerhalb von `src/sources/wiktionary.rs`.
* Nur dann gemeinsame Modelle ändern, wenn es wirklich nötig ist.
* Ausgabe und Tests sollen deterministisch sein: stabile Reihenfolge, deduplizierte Listen, keine zufälligen Daten.

Falls die aktuelle Parser-Strategie in `src/sources/wiktionary.rs` mit `h2, h3, h4` nicht zuverlässig genug für REST-HTML ist, ersetze sie durch eine robustere Struktur, aber halte den Scope auf Wiktionary begrenzt.

### Akzeptanzkriterien

* Wiktionary-Live-Lookups verwenden `https://de.wiktionary.org/api/rest_v1/page/html/{wort}`.
* Die normale Wiktionary-Webseite wird nicht mehr als Live-HTML geparst.
* Der Parser ist unabhängig vom HTTP-Code testbar.
* Mehrere Wiktionary-Einträge pro Artikel werden unterstützt.
* `Bank` ergibt mehrere `DictionaryEntry`-Einträge.
* Bedeutungen werden als `Sense`-Werte extrahiert.
* Beispiele werden den passenden Bedeutungen zugeordnet.
* Synonyme werden als `SynonymGroup`-Werte extrahiert, möglichst mit Sense-Bezug.
* Herkunft wird als `etymology` extrahiert.
* Redewendungen werden als `idioms` extrahiert.
* Fehlende Sektionen führen nicht zu Fehlern oder Panics.
* Es gibt lokale REST-HTML-Fixtures für Wiktionary.
* Es gibt Snapshot-Tests unter `tests/snapshots/wiktionary/`.
* Die Snapshot-Tests laufen offline.
* Die Tests orientieren sich am bestehenden OpenThesaurus-Muster.
* `cargo test wiktionary` läuft erfolgreich.
* Wenn möglich, läuft auch `cargo test` erfolgreich.

### Out of scope

* Kein Lemmatizer.
* Kein Caching.
* Kein Rate Limiting.
* Keine große Überarbeitung der CLI.
* Keine Umstellung auf ein Snapshot-Framework wie `insta`, solange das Projekt bereits einfache `.snap`-Dateien mit `pretty_assertions` verwendet.
* Keine vollständige 1:1-Kompatibilität mit dem alten Emacs-Lisp-Plist-Output; maßgeblich sind die Rust-Modelle in `src/models.rs`.

### Schritt 3: Asynchrones Zusammenspiel & CLI-Interface

Nun führen wir die Konfigurationen und die parallele Ausführung in der Hauptfunktion zusammen.

#### Prompt für die KI

    Nun führen wir alles zusammen. In meinem Elisp-Code gibt es `woerterbuch-sources` (Reihenfolge der Quellen) und `woerterbuch-default-sections` (welche Inhalte standardmässig gewünscht sind).

    Bitte baue die `main`-Funktion so um, dass:
    1. Über `clap` das Suchwort direkt entgegengenommen wird, zusammen mit optionalen Flags (wie `--json` oder `--sources`).
    2. Die aktivierten Quellen (Duden via AMP, Wiktionary via REST-HTML, etc.) vollkommen parallel mit `tokio::join!` abgefragt und geparst werden.
    3. Die Ergebnisse gemäss der gewünschten Sektionen gefiltert, zusammengeführt und entweder schön formatiert im Terminal oder als JSON auf stdout ausgegeben werden.

## DONE Openthesaurus [archive/2026-06-09--openthesaurus-parser.md](archive/2026-06-09--openthesaurus-parser.md) 

In dem Ordner sind die Emasc Lisp Dateien für das Scrapen/Parsen [emacs-lisp](../../emacs-lisp). Für OpenThesaurus ist relevant:

- [../../emacs-lisp/lisp/woerterbuch-openthesaurus.el](../../emacs-lisp/lisp/woerterbuch-openthesaurus.el) Logik für Scrapen/Parsen
- Testdateien, inbesondere den erwarteten Output je Section:
  [tests/openthesaurus](../../emacs-lisp/tests/files/openthesaurus/)

Bitte übernehme einfach die Logik, aber mache es so wie in Rust normal. Wenn du in der Logik Schwachstellen siehst, dann verbessere dies.
