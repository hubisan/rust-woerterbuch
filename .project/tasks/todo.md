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

## TODO DWDS: HTML-Parser + Snapshot-Tests

# Abgeschlossen

## DONE Duden: HTML-Parser + Snapshot-Tests

Task file: [./archive/2026-06-09--duden-parser.md](./archive/2026-06-09--duden-parser.md)

Portiere die alte Duden-Scraping-Logik aus Emacs Lisp nach Rust und ersetze den aktuellen groben Duden-Stub durch einen robusten AMP-HTML-Parser mit deterministischen Offline-Snapshot-Tests.

Diese Aufgabe ist der nächste Parser-Schritt nach OpenThesaurus und Wiktionary. Maßgeblich ist nicht eine freie Neuinterpretation des Duden-HTMLs, sondern die fachliche Logik aus `woerterbuch-duden.el`, idiomatisch in Rust umgesetzt und auf die vorhandenen Rust-Modelle gemappt.

### Wichtigste geprüfte Grundlage

Unbedingt zuerst lesen:

- `../../emacs-lisp/lisp/woerterbuch-duden.el`
- `../../emacs-lisp/tests/test-helper.el`
- `../../emacs-lisp/tests/test-woerterbuch-duden.el`
- `../../emacs-lisp/tests/files/duden/`
- `../../src/sources/duden.rs`
- `../../src/sources/wiktionary.rs`
- `../../src/sources/openthesaurus.rs`
- `../../src/models.rs`

Aus `woerterbuch-duden.el` sind insbesondere diese Funktionen relevant und sollen fachlich portiert werden:

- `woerterbuch-duden--build-url`
- `woerterbuch-duden--build-search-url`
- `woerterbuch-duden--clean-text`
- `woerterbuch-duden--text`
- `woerterbuch-duden--tuple-pairs`
- `woerterbuch-duden--notes`
- `woerterbuch-duden--definition-label`
- `woerterbuch-duden--extract-image-url`
- `woerterbuch-duden--extract-qualifiers`
- `woerterbuch-duden--extract-shortform-definition`
- `woerterbuch-duden--extract-definition-node`
- `woerterbuch-duden--parse-single-definition-section`
- `woerterbuch-duden--parse-definitions`
- `woerterbuch-duden--extract-title-node`
- `woerterbuch-duden--extract-lemma`
- `woerterbuch-duden--extract-title`
- `woerterbuch-duden--field-value`
- `woerterbuch-duden--wortart-from-grammar`
- `woerterbuch-duden--extract-origin`
- `woerterbuch-duden--split-synonym-text`
- `woerterbuch-duden--extract-synonyms`
- `woerterbuch-duden--parse-search-results`
- `woerterbuch-duden--result-from-homographs`
- `woerterbuch-duden--no-match-result`

Der aktuelle Rust-Code in `src/sources/duden.rs` ist nur ein grober Stub. Er verwendet Selektoren wie `.tuple__wortart` und `.tuple__grammatik`, die in den vorhandenen Duden-AMP-Fixtures nicht die eigentliche Struktur abbilden. Die echte Struktur besteht aus `h1.lemma__title`, `span.lemma__main`, `dl.tuple`, `div.division#bedeutungen`, `ol.enumeration`, `li.enumeration__item`, `li.enumeration__sub-item`, `dl.note`, `div#synonyme` und `div#herkunft`.

### Ziel

Der Duden-Parser soll Duden-AMP-HTML in die vorhandenen Rust-Modelle überführen:

- `SourceResult`
- `DictionaryEntry`
- `Sense`
- `SynonymGroup`
- `UrlValue`

Die Ausgabe muss die vorhandene Architektur respektieren und nach dem Muster von OpenThesaurus/Wiktionary über lokale Fixtures und `.snap`-Dateien testbar sein.

### Architektur

Halte den Scope möglichst auf `src/sources/duden.rs` begrenzt.

Erwartete Struktur:

- `lookup(client, query) -> Result<SourceResult>` bleibt für HTTP, Statuscodes, Fallback-Suche und Fehlerbehandlung zuständig.
- Der eigentliche Parser bleibt unabhängig vom HTTP-Code testbar.
- Der Parser soll ein vollständiges `SourceResult` oder testbare Hilfsfunktionen liefern, nicht nur lose Textlisten.
- Tests dürfen keine Live-Requests machen.
- Keine Panics bei fehlenden oder unerwarteten HTML-Sektionen.
- Keine große Umstellung der CLI.
- Keine Änderung an gemeinsamen Modellen, außer es ist wirklich notwendig.
- Keine Umstellung auf ein neues Snapshot-Framework.

Die zentrale Section-Filterung existiert bereits über `SourceResult::retain_sections` in `src/models.rs`. Daher in Duden nicht wieder das alte Elisp-`sections`-Argument nachbauen. Der Rust-Duden-Parser soll grundsätzlich alle gefundenen Daten extrahieren; die gewünschte Auswahl übernimmt danach die bestehende zentrale Logik.

### Live-Lookup und URLs

Die alte Duden-Implementierung und der Projektkontext nutzen die schlanke AMP-Seite.

Portiere die URL-Logik aus `woerterbuch-duden.el`:

- Direkte Duden-Eintrags-URL:
  - Basis: `https://www.duden.de/rechtschreibung/`
  - Leerraum im Lemma zuerst durch `_` ersetzen.
  - Danach URL-encoden.
  - Am Ende `?amp` anhängen.
  - Beispiel: `Bank` -> `https://www.duden.de/rechtschreibung/Bank?amp`

- Duden-Such-URL:
  - Basis: `https://www.duden.de/suchen/dudenonline/`
  - Suchbegriff URL-encoden.
  - Beispiel: `Bank` -> `https://www.duden.de/suchen/dudenonline/Bank`

Für diese Aufgabe ist das alte Duden-Verhalten mit `?amp` maßgeblich. Die alten Expected-Dateien erwarten ebenfalls URLs wie:

- `https://www.duden.de/rechtschreibung/Haus?amp`
- `https://www.duden.de/rechtschreibung/Bank_Sitzgelegenheit?amp`
- `https://www.duden.de/rechtschreibung/Bank_Geldinstitut?amp`

Nicht in dieser Aufgabe auf normale Nicht-AMP-Ergebnis-URLs umstellen.

### HTTP-Verhalten

Der aktuelle `fetch_html`-Helper nutzt `error_for_status()` und ist für Duden allein nicht ausreichend, weil Duden bei Homographen wie `Bank` zuerst auf der direkten URL `404` liefern kann und danach die Suchseite abgefragt werden muss.

Portiere das Verhalten aus dem Elisp-Code:

1. Zuerst direkte AMP-URL abrufen.
2. Wenn die direkte URL erfolgreich ist, diese eine Eintragsseite parsen.
3. Wenn die direkte URL `404` ergibt, Duden-Suche abrufen.
4. Suchergebnisseite parsen und exakt passende Eintragslinks sammeln.
5. Alle exakt passenden Eintragsseiten abrufen und als mehrere `DictionaryEntry`-Werte zurückgeben.
6. Wenn keine passenden Einträge gefunden werden, `SourceResult::error(Source::Duden, "No matches found")` zurückgeben.
7. Bei Nicht-404-HTTP-Fehlern oder Netzwerkfehlern saubere Fehler liefern.

Für `Bank` ist der alte Testablauf in `test-helper.el` wichtig:

- direkte URL `https://www.duden.de/rechtschreibung/Bank?amp` -> simuliert `404`
- Suchseite `https://www.duden.de/suchen/dudenonline/Bank`
- daraus exakt:
  - `https://www.duden.de/rechtschreibung/Bank_Sitzgelegenheit?amp`
  - `https://www.duden.de/rechtschreibung/Bank_Geldinstitut?amp`
- nicht aufnehmen:
  - `Merchant_Bank`
  - `Bad_Bank`
  - `Near_Bank`
  - `Bankster`
  - `Banker`
  - `_bank`
  - sonstige Teilsuche-Treffer

### Suchseite und Homographen

Portiere die Suchlogik aus `woerterbuch-duden--parse-search-results`.

Duden-Suchseiten enthalten einen Segmentblock:

- `div.segment`
- darin `h2.segment__title` mit Text `Wörterbuch`
- darin mehrere `section.vignette`
- relevante Links über `a.vignette__label[href]`

Exakt passende Treffer werden so erkannt:

- Link muss mit `/rechtschreibung/` beginnen.
- Sichtbares Lemma stammt bevorzugt aus `strong` innerhalb von `a.vignette__label`.
- Falls kein `strong` existiert, verwende den Text des Labels.
- Nur wenn der bereinigte sichtbare Text exakt dem bereinigten Suchwort entspricht, wird der Link übernommen.
- Reihenfolge der Treffer bleibt DOM-Reihenfolge.
- URLs werden zu absoluten Duden-URLs gemacht und mit `?amp` versehen.

Für mehrere Treffer:

- `SourceResult.url` soll `UrlValue::Many` enthalten.
- Jeder Treffer wird als eigener `DictionaryEntry` in `entries` abgelegt.
- Entry-IDs sind stabil: `1`, `2`, `3`, ...
- Die Reihenfolge entspricht der Suchergebnis-Reihenfolge.

### Mapping auf Rust-Modelle

Nutze die vorhandenen Felder aus `src/models.rs`.

Mapping:

- Duden-Lemma aus `span.lemma__main` -> `DictionaryEntry.headword`
- vollständiger Titel aus `h1.lemma__title` -> `DictionaryEntry.title`
- Duden-Wortart/Grammatik aus `dl.tuple` mit Key `Wortart` -> `DictionaryEntry.grammar`
- grobe Wortart aus `grammar` vor dem ersten Komma -> `DictionaryEntry.part_of_speech`
  - Beispiel `Substantiv, Neutrum` -> `Substantiv`
  - Beispiel `starkes Verb` -> `starkes Verb`
  - Beispiel `schwaches Verb` -> `schwaches Verb`
- Abschnitt `#herkunft` -> `DictionaryEntry.etymology`
- Abschnitt `#synonyme` -> `DictionaryEntry.synonym_groups`
- Abschnitt `#bedeutungen` oder `#bedeutung` -> `DictionaryEntry.senses`
- `li`-ID wie `Bedeutung-1a` -> `Sense.source_id`
- sichtbares Bedeutungslabel wie `1`, `1a`, `2b` -> `Sense.label`
- `div.enumeration__text` -> `Sense.definition`
- `dl.tuple` innerhalb einer Bedeutung -> `Sense.qualifiers`
- `dl.note` mit Titel `Beispiele` oder `Beispiel` -> `Sense.examples`
- `dl.note` mit Titel `Wendungen, Redensarten, Sprichwörter` -> `Sense.idioms`
- `figure.depiction a[href]` -> `Sense.image_url`
- Unterbedeutungen -> `Sense.subsenses`

Wichtig: Duden-Redewendungen stehen in den alten Fixtures meistens als `dl.note` direkt an einer Bedeutung oder Unterbedeutung. Sie sollen deshalb primär an `Sense.idioms` hängen, nicht pauschal an `DictionaryEntry.idioms`.

### Bedeutungen und Unterbedeutungen

Portiere die rekursive Logik aus `woerterbuch-duden--extract-definition-node`.

Die Duden-AMP-Struktur sieht typischerweise so aus:

- `div#bedeutungen`
- `ol.enumeration`
- direkte Kinder `li.enumeration__item`
- optional darin `ol.enumeration__sub`
- direkte Kinder davon `li.enumeration__sub-item`

Regeln:

- Top-Level-Bedeutungen erhalten Labels `1`, `2`, `3`, ...
- Unterbedeutungen erhalten Labels aus der Duden-ID, zum Beispiel `Bedeutung-1a` -> `1a`.
- Falls keine Duden-ID vorhanden ist, deterministisch fallbacken, zum Beispiel `1a`, `1b`.
- Wenn eine Top-Level-Bedeutung nur Unterbedeutungen hat, bleibt ihre eigene `definition` leer und die Unterbedeutungen stehen in `subsenses`.
- Wenn eine Top-Level-Bedeutung selbst `div.enumeration__text` hat, wird diese als eigene Definition übernommen.
- Für einfache Einträge ohne `ol.enumeration` soll `woerterbuch-duden--parse-single-definition-section` fachlich portiert werden.
- Kurze Formen mit führendem Tuple `Kurzform für` müssen als Definition erzeugt werden:
  - `Kurzform für: Sandbank`
  - `Kurzform für: verschiedene Handwerkstische wie Drehbank, Hobelbank, Werkbank u. a.`

Sehr wichtig für Rust mit `scraper`:

- Bei Bedeutungen möglichst direkte Kinder traversieren, nicht wahllos alle Descendants sammeln.
- Sonst landen Beispiele/Idiome aus Unterbedeutungen fälschlich auf dem Parent-Sense.
- Die Elisp-Funktionen `woerterbuch-duden--children-with-class`, `woerterbuch-duden--direct-child-by-tag-and-class` und `woerterbuch-duden--direct-children-by-tag-and-class` sind dafür die Vorlage.

### Beispiele, Redewendungen, Qualifier, Bilder

Portiere das Verhalten aus:

- `woerterbuch-duden--notes`
- `woerterbuch-duden--note-values`
- `woerterbuch-duden--extract-qualifiers`
- `woerterbuch-duden--extract-image-url`

Regeln:

- `dl.note` wird nur als direkter Note-Block der jeweiligen Bedeutung ausgewertet.
- Note-Titel werden normalisiert.
- `Beispiele` und `Beispiel` werden beide unterstützt.
- Alle `li` innerhalb der jeweiligen Note-Liste werden als Beispiele übernommen.
- `Wendungen, Redensarten, Sprichwörter` wird als Idiom-Liste der jeweiligen Bedeutung übernommen.
- `dl.tuple` innerhalb einer Bedeutung wird als Qualifier-Liste übernommen.
- Qualifier-Format: `Key: Value`, zum Beispiel `Gebrauch: Sport`.
- Tuple `Kurzform für` wird nicht als Qualifier übernommen, sondern als Definition-Fallback verwendet.
- Bild-URL kommt aus `figure.depiction a[href]`.

### Synonyme

Portiere die Logik aus:

- `woerterbuch-duden--split-synonym-text`
- `woerterbuch-duden--extract-synonyms`

Duden-Synonyme stehen in den Fixtures im Bereich:

- `div#synonyme`
- direkter `ul`
- darin `li`
- darin oft mehrere Links, getrennt durch Komma oder Semikolon

Regeln:

- Nur den eigentlichen Synonym-Block parsen, nicht den `nav.more`-Link `Zur Übersicht der Synonyme ...`.
- Synonyme auf Komma und Semikolon splitten, aber nicht innerhalb von Klammern.
- Beispiele:
  - `(österreichisch) sich verschauen` bleibt ein Synonym.
  - `(gehoben) in Liebe entbrennen/erglühen` bleibt ein Synonym.
- Reihenfolge bleibt stabil.
- Deduplizieren ohne Sortierung.
- Leere Einträge ignorieren.
- In Rust als `SynonymGroup` speichern.
- Wenn kein Sense-Bezug erkennbar ist, eine Gruppe mit `sense = None` verwenden.

### Herkunft

Portiere `woerterbuch-duden--extract-origin`.

Regeln:

- Bereich mit `id="herkunft"` suchen.
- Header, `small`, Navigation und Info-Icons ignorieren.
- Relevante direkte Kindtexte zusammenführen.
- Whitespace und HTML-Entities normalisieren.
- Wenn kein Herkunftstext vorhanden ist, `None`.

Beispiele aus den alten Expected-Dateien:

- `Haus`:
  - `mittelhochdeutsch, althochdeutsch hūs, eigentlich = das Bedeckende, Umhüllende`
- `Bank`, Sitzgelegenheit:
  - `mittelhochdeutsch, althochdeutsch banc = Bank, Tisch, ursprünglich = Erhöhung`
- `Bank`, Geldinstitut:
  - `italienisch banco, banca, eigentlich = Tisch des Geldwechslers, aus dem Germanischen`
- `verlieben`:
  - kein Herkunftstext, also `None`

### Textbereinigung

Portiere `woerterbuch-duden--clean-text` und ergänze nur vorsichtig, wo Rust/HTML es nötig macht.

Mindestregeln:

- Whitespace inklusive Non-Breaking-Spaces normalisieren.
- Mehrfachspaces zu einem Space.
- `〈` zu `⟨`.
- `〉` zu `⟩`.
- Leerzeichen vor `,` und `.` entfernen.
- Leerzeichen direkt nach `(` und direkt vor `)` entfernen.
- HTML-Entities sauber dekodieren, soweit `scraper`/Textnodes dies liefern.
- Duden-Layout-Texte wie `Anzeige`, `Weitere Beispiele anzeigen`, `Zur Übersicht der Synonyme ...`, Share-Buttons und Navigation nicht in Ergebnistext übernehmen.
- Sichtbare Duden-Schreibweise mit Soft Hyphen in Headword/Title nicht unnötig zerstören. Beispiele: `sprin­gen`, `Wol­ke`, `ver­lie­ben`.

Nicht die fachlich relevanten Duden-Typografie-Marker entfernen:

- `[ver]mieten`
- `[jemandem]`
- `⟨in übertragener Bedeutung:⟩`
- `(umgangssprachlich)`
- `(gehoben)`

### Fixtures

Lege lokale Rust-Fixtures an, vorzugsweise unter:

- `tests/fixtures/duden/`

Nutze die vorhandenen alten Duden-AMP-Fixtures als Ausgangspunkt. Diese liegen bereits im Repository unter:

- `../../emacs-lisp/tests/files/duden/Haus/duden-Haus.html`
- `../../emacs-lisp/tests/files/duden/Bank/duden-Bank-search.html`
- `../../emacs-lisp/tests/files/duden/Bank/duden-Bank-1.html`
- `../../emacs-lisp/tests/files/duden/Bank/duden-Bank-2.html`
- `../../emacs-lisp/tests/files/duden/springen/duden-springen.html`
- `../../emacs-lisp/tests/files/duden/verlieben/duden-verlieben.html`
- `../../emacs-lisp/tests/files/duden/Wolke/duden-Wolke.html`
- `../../emacs-lisp/tests/files/duden/Zaun/duden-Zaun.html`

Empfohlene Rust-Fixture-Namen:

- `tests/fixtures/duden/Haus.html`
- `tests/fixtures/duden/Bank-search.html`
- `tests/fixtures/duden/Bank-Sitzgelegenheit.html`
- `tests/fixtures/duden/Bank-Geldinstitut.html`
- `tests/fixtures/duden/springen.html`
- `tests/fixtures/duden/verlieben.html`
- `tests/fixtures/duden/Wolke.html`
- `tests/fixtures/duden/Zaun.html`

Für `Nixdaexistiert` gibt es in den alten Duden-Dateien nur Expected-Output, aber keine echte HTML-Fixture. Der Rust-Test für diesen Fall kann deshalb über einen `not_found_result`-Helper oder eine kleine synthetische leere Suchseite laufen.

Keine Live-Downloads in Tests.

### Snapshot-Tests

Ergänze Duden-Snapshots analog zu den bestehenden Quellen:

- `tests/snapshots/duden/Bank.snap`
- `tests/snapshots/duden/Haus.snap`
- `tests/snapshots/duden/springen.snap`
- `tests/snapshots/duden/verlieben.snap`
- `tests/snapshots/duden/Wolke.snap`
- `tests/snapshots/duden/Zaun.snap`
- `tests/snapshots/duden/Nixdaexistiert.snap`

Die Snapshot-Ausgabe soll textuell, deterministisch und gut diffbar sein, ähnlich wie bei Wiktionary/OpenThesaurus.

Der Duden-Snapshot-Renderer muss rekursiv mit `Sense.subsenses` umgehen. Sonst fehlen bei `Haus` und `Bank` die wichtigsten Bedeutungen wie `1a`, `1b`, `2a`.

Beispielhafte Form:

    source=Duden
    ok=true
    url=https://www.duden.de/rechtschreibung/Haus?amp
    entry 1 headword=Haus title=Haus, das part_of_speech=Substantiv grammar=Substantiv, Neutrum
    etymology=mittelhochdeutsch, althochdeutsch hūs, eigentlich = das Bedeckende, Umhüllende
    synonyms sense=- items=[Anwesen, Bau, Bauwerk, Gebäude]
    sense 1 label=1 source_id=- definition=-
    subsense 1.1 label=1a source_id=Bedeutung-1a definition=Gebäude, das Menschen zum Wohnen dient
    image 1.1=https://cdn.duden.de/_media_/full/H/Haus-201020510799.jpg
    examples 1.1=[ein großes, kleines, altes, mehrstöckiges, verwinkeltes Haus | armselige, einfache, verkommene, baufällige, moderne Häuser | ...]
    idioms 1.1=[Haus und Hof ... | [jemandem] ins Haus stehen ...]
    subsense 1.2 label=1b source_id=Bedeutung-1b definition=Gebäude, das zu einem bestimmten Zweck errichtet wurde

Für `Bank` muss der Snapshot zwei Entries enthalten:

    source=Duden
    ok=true
    url=[https://www.duden.de/rechtschreibung/Bank_Sitzgelegenheit?amp | https://www.duden.de/rechtschreibung/Bank_Geldinstitut?amp]
    entry 1 headword=Bank title=Bank, die part_of_speech=Substantiv grammar=Substantiv, feminin
    ...
    entry 2 headword=Bank title=Bank, die part_of_speech=Substantiv grammar=Substantiv, feminin
    synonyms sense=- items=[Bankhaus, Geldinstitut, Kreditanstalt, Kreditinstitut]
    ...

Für `Nixdaexistiert`:

    source=Duden
    ok=false
    url=-
    error=No matches found

Die genaue Formatierung darf an die vorhandenen Render-Helfer angepasst werden, muss aber stabil bleiben.

### Zusätzliche Unit-Tests

Neben Snapshot-Tests bitte gezielte Unit-Tests für Parser-Hilfsfunktionen ergänzen.

Mindestens testen:

- URL-Bildung:
  - `Bank` -> `https://www.duden.de/rechtschreibung/Bank?amp`
  - Leerzeichen im Lemma werden für Eintrags-URLs zu `_`.
  - Such-URL nutzt `/suchen/dudenonline/`.
- Textbereinigung:
  - Non-Breaking-Spaces.
  - `〈...〉` -> `⟨...⟩`.
  - Spaces vor Satzzeichen.
  - Kein Verlust relevanter Klammern oder eckiger Klammern.
- Titel/Lemma:
  - `h1.lemma__title` und `span.lemma__main`.
  - `Haus, das` vs. `Haus`.
  - Soft-Hyphen-Schreibungen wie `ver­lie­ben`.
- Tuple-Parsing:
  - `Wortart: ⓘ` -> Key `Wortart`.
  - `Substantiv, Neutrum` -> `grammar`.
  - `part_of_speech` aus erstem Teil vor Komma.
- Suchergebnisse:
  - `Bank-search.html` liefert exakt zwei URLs.
  - `Merchant_Bank`, `Bad_Bank`, `Near_Bank`, `Bankster`, `Banker` werden nicht übernommen.
- Bedeutungslabels:
  - `Bedeutung-1a` -> `1a`.
  - `Bedeutung-2` -> `2`.
  - Fallback für fehlende IDs.
- Rekursive Bedeutungen:
  - Parent-Sense mit `subsenses`.
  - Flache Bedeutung ohne Unterbedeutungen.
  - Single-definition-Fallback.
- Beispiele:
  - `Beispiele` und `Beispiel`.
  - Beispiele bleiben an der passenden Bedeutung.
  - Parent-Sense sammelt nicht versehentlich Beispiele aus allen Sub-Senses.
- Redewendungen:
  - `Wendungen, Redensarten, Sprichwörter`.
  - Speicherung in `Sense.idioms`.
- Qualifier:
  - `Gebrauch: Sport`.
  - `Kurzform für` wird nicht als Qualifier übernommen.
- Kurzform:
  - führendes Tuple `Kurzform für` wird Definition-Fallback.
- Bilder:
  - `figure.depiction a[href]`.
- Synonyme:
  - Split auf Komma/Semikolon außerhalb von Klammern.
  - Deduplizierung mit stabiler Reihenfolge.
  - Keine Aufnahme des `Zur Übersicht der Synonyme ...`-Links.
- Herkunft:
  - Header/Info-Icons werden ignoriert.
  - Herkunftstext wird korrekt zusammengesetzt.
- Fehlerfall:
  - kein Eintrag -> `ok=false`, `error=No matches found`.

### Hinweise zu den alten Expected-Dateien

Die alten `.el`-Expected-Dateien sind keine 1:1-Rust-Snapshot-Vorlage, aber sie sind die fachliche Oracle-Referenz.

Besonders hilfreiche Dateien:

- `../../emacs-lisp/tests/files/duden/Haus/duden-Haus-definitions-expected.el`
- `../../emacs-lisp/tests/files/duden/Haus/duden-Haus-examples-expected.el`
- `../../emacs-lisp/tests/files/duden/Haus/duden-Haus-idioms-expected.el`
- `../../emacs-lisp/tests/files/duden/Haus/duden-Haus-origin-expected.el`
- `../../emacs-lisp/tests/files/duden/Haus/duden-Haus-synonyms-expected.el`
- `../../emacs-lisp/tests/files/duden/Bank/duden-Bank-definitions-expected.el`
- `../../emacs-lisp/tests/files/duden/Bank/duden-Bank-examples-expected.el`
- `../../emacs-lisp/tests/files/duden/Bank/duden-Bank-origin-expected.el`
- `../../emacs-lisp/tests/files/duden/Bank/duden-Bank-synonyms-expected.el`
- `../../emacs-lisp/tests/files/duden/Nixdaexistiert/duden-Nixdaexistiert-definitions-expected.el`

Die Rust-Snapshots sollen die gleichen fachlichen Daten enthalten, aber im vorhandenen Rust-Snapshot-Stil.

### Workflow für Codex

Beim Umsetzen:

1. `.project/agents/AGENTS.md` und `.project/agents/repository.md` lesen.
2. Task-Datei nach Template anlegen, zum Beispiel `.project/tasks/2026-06-09--duden-parser.md`.
3. Diese TODO-Heading beim Start mit der Task-Datei verlinken.
4. Duden-Parser implementieren.
5. Fixtures/Snapshots/Unit-Tests ergänzen.
6. Relevante Checks ausführen.
7. Task-Datei mit Result, Changes, Checks und Open Points aktualisieren.
8. TODO-Status am Ende auf `REVIEW` setzen.

### Akzeptanzkriterien

- Duden nutzt live die AMP-Eintragsseiten mit `?amp`.
- Direkte Duden-URL wird zuerst versucht.
- Bei `404` wird die Duden-Suchseite verwendet.
- Exakte Suchtreffer werden erkannt.
- Homographen wie `Bank` ergeben mehrere `DictionaryEntry`-Einträge.
- `Bank` liefert genau die beiden alten Duden-Einträge `Bank_Sitzgelegenheit` und `Bank_Geldinstitut`.
- Nicht exakt passende Suchtreffer werden ignoriert.
- Parser-Logik ist unabhängig vom HTTP-Code testbar.
- Tests laufen offline.
- Lokale Duden-Fixtures existieren unter `tests/fixtures/duden/` oder einem konsistenten bestehenden Fixture-Ort.
- Duden-Snapshots existieren unter `tests/snapshots/duden/`.
- `DictionaryEntry.headword`, `title`, `part_of_speech`, `grammar`, `etymology`, `url`, `synonym_groups` und `senses` werden sinnvoll befüllt.
- Bedeutungen und Unterbedeutungen werden rekursiv als `Sense`/`subsenses` abgebildet.
- Duden-Bedeutungs-IDs wie `Bedeutung-1a` werden in `Sense.source_id` erhalten.
- Labels wie `1`, `1a`, `2b` werden stabil gesetzt.
- Beispiele werden der passenden Bedeutung zugeordnet.
- Redewendungen werden der passenden Bedeutung als `Sense.idioms` zugeordnet.
- Qualifier wie `Gebrauch: Sport` werden extrahiert.
- Kurzform-Tuples werden als Definition-Fallback verwendet.
- Bild-URLs werden extrahiert, soweit vorhanden.
- Synonyme werden dedupliziert und stabil sortierungsfrei gespeichert.
- Herkunft wird extrahiert, soweit vorhanden.
- Fehlende Sektionen führen nicht zu Fehlern oder Panics.
- Duden-Artefakte wie Anzeige, Navigation, Share-Buttons und `Zur Übersicht der Synonyme ...` erscheinen nicht in Snapshots.
- `Nixdaexistiert` ergibt `ok=false` und `error=No matches found`.
- `cargo test duden` läuft erfolgreich.
- Wenn möglich, läuft auch `cargo test` erfolgreich.

### Out of scope

- Kein Lemmatizer.
- Kein Caching.
- Kein Rate Limiting.
- Keine große CLI-Überarbeitung.
- Keine DWDS-Implementierung in dieser Aufgabe.
- Keine Änderungen an OpenThesaurus oder Wiktionary, außer wenn ein winziger gemeinsamer Test-/Snapshot-Helfer wirklich sinnvoll ist.
- Keine vollständige 1:1-Kompatibilität mit dem alten Emacs-Lisp-Plist-Output.
- Keine Live-Requests in Tests.
- Keine Umstellung auf `insta` oder ein neues Snapshot-Framework.
- Keine Umstellung der Duden-Ergebnis-URLs von `?amp` auf Nicht-AMP in dieser Aufgabe.

## DONE Wiktionary: REST-HTML-Parser + Snapshot-Tests [archive/2026-06-09--wiktionary-parser.md](archive/2026-06-09--wiktionary-parser.md) 

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
