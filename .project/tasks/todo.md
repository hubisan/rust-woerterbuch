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

### NEXT Openthesaurus

In dem Ordner sind die Emasc Lisp Dateien für das Scrapen/Parsen [emacs-lisp](../../emacs-lisp). Für OpenThesaurus ist relevant:

- [../../emacs-lisp/lisp/woerterbuch-openthesaurus.el](../../emacs-lisp/lisp/woerterbuch-openthesaurus.el) Logik für Scrapen/Parsen
- Testdateien, inbesondere den erwarteten Output je Section:
  [tests/openthesaurus](../../emacs-lisp/tests/files/openthesaurus/)
    
Bitte übernehme einfach die Logik, aber mache es so wie in Rust normal. Wenn du in der Logik Schwachstellen siehst, dann verbessere dies.

### TODO Wiktionary

Schau dir meine Datei `woerterbuch-wiktionary.el` an. Im Gegensatz zum alten Elisp-Code, wo die normale Webseite gescraped wurde, nutzen wir in Rust nun die offizielle deutsche HTML-API unter `https://de.wiktionary.org/api/rest_v1/page/html/{wort}`. Das liefert uns das nackte Inhalts-HTML ohne den Wikipedia-Seitenballast.

## Schritt 3: Asynchrones Zusammenspiel & CLI-Interface

Nun führen wir die Konfigurationen und die parallele Ausführung in der Hauptfunktion zusammen.

### Prompt für die KI

    Nun führen wir alles zusammen. In meinem Elisp-Code gibt es `woerterbuch-sources` (Reihenfolge der Quellen) und `woerterbuch-default-sections` (welche Inhalte standardmässig gewünscht sind).

    Bitte baue die `main`-Funktion so um, dass:
    1. Über `clap` das Suchwort direkt entgegengenommen wird, zusammen mit optionalen Flags (wie `--json` oder `--sources`).
    2. Die aktivierten Quellen (Duden via AMP, Wiktionary via REST-HTML, etc.) vollkommen parallel mit `tokio::join!` abgefragt und geparst werden.
    3. Die Ergebnisse gemäss der gewünschten Sektionen gefiltert, zusammengeführt und entweder schön formatiert im Terminal oder als JSON auf stdout ausgegeben werden.

# Abgeschlossen
