use crate::models::{DictionaryEntry, LookupResponse, Sense, SourceResult, SynonymGroup};

pub fn print_human(response: &LookupResponse) {
    println!("Dictionary lookup: {}", response.query);

    if response.results.is_empty() {
        println!("No sources selected.");
        return;
    }

    for source in &response.results {
        print_source(source);
    }
}

fn print_source(source: &SourceResult) {
    println!("\n== {:?} ==", source.source);

    if let Some(message) = source_status_message(source) {
        println!("{message}");
        return;
    }

    for entry in &source.entries {
        print_entry(entry);
    }
}

fn source_status_message(source: &SourceResult) -> Option<String> {
    if !source.ok {
        let error = source.error.as_deref().unwrap_or("unknown error");
        return Some(if is_not_found_message(error) {
            "No entry found on source.".to_owned()
        } else {
            format!("Error: {error}")
        });
    }

    if source.entries.is_empty() {
        return Some(if source.url.is_none() {
            "Skipped: source does not support requested sections.".to_owned()
        } else {
            "No content for requested sections.".to_owned()
        });
    }

    None
}

fn is_not_found_message(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("no matches found")
}

fn print_entry(entry: &DictionaryEntry) {
    println!("\n-- Entry {}: {} --", entry.id, entry.headword);

    if let Some(title) = &entry.title {
        println!("Title: {title}");
    }
    if let Some(part_of_speech) = &entry.part_of_speech {
        println!("Part of speech: {part_of_speech}");
    }
    if let Some(grammar) = &entry.grammar {
        println!("Grammar: {grammar}");
    }

    print_optional("Etymology", entry.etymology.as_deref());
    print_list("Idioms", &entry.idioms);
    print_synonym_groups("Synonyms", &entry.synonym_groups);

    if !entry.senses.is_empty() {
        println!("\nDefinitions:");
        for sense in &entry.senses {
            print_sense(sense, 1);
        }
    }
}

fn print_sense(sense: &Sense, depth: usize) {
    let indent = "  ".repeat(depth);
    let label = sense.label.as_deref().unwrap_or("-");
    let text = sense.definition.as_deref().unwrap_or("");
    println!("{indent}- {label} {text}");

    print_indented_list("Examples", &sense.examples, depth + 1);
    print_indented_list("Idioms", &sense.idioms, depth + 1);
    print_indented_list("Synonyms", &sense.synonyms, depth + 1);

    for child in &sense.subsenses {
        print_sense(child, depth + 1);
    }
}

fn print_optional(title: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        println!("\n{title}:\n  {value}");
    }
}

fn print_list(title: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    println!("\n{title}:");
    for value in values {
        println!("  - {value}");
    }
}

fn print_indented_list(title: &str, values: &[String], depth: usize) {
    if values.is_empty() {
        return;
    }
    let indent = "  ".repeat(depth);
    println!("{indent}{title}:");
    for value in values {
        println!("{indent}  - {value}");
    }
}

fn print_synonym_groups(title: &str, groups: &[SynonymGroup]) {
    if groups.is_empty() {
        return;
    }
    println!("\n{title}:");
    for group in groups {
        match &group.sense {
            Some(sense) => println!("  - Sense {sense}: {}", group.items.join(", ")),
            None => println!("  - {}", group.items.join(", ")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Source, UrlValue};

    #[test]
    fn source_status_reports_skipped_sources_clearly() {
        let source = SourceResult::ok(Source::Dwds, None, Vec::new());

        assert_eq!(
            source_status_message(&source).as_deref(),
            Some("Skipped: source does not support requested sections.")
        );
    }

    #[test]
    fn source_status_reports_filtered_empty_results_clearly() {
        let source = SourceResult::ok(
            Source::Duden,
            Some(UrlValue::One("https://example.test".to_owned())),
            Vec::new(),
        );

        assert_eq!(
            source_status_message(&source).as_deref(),
            Some("No content for requested sections.")
        );
    }

    #[test]
    fn source_status_normalizes_not_found_errors() {
        let source = SourceResult {
            source: Source::Wiktionary,
            ok: false,
            url: Some(UrlValue::One("https://example.test".to_owned())),
            entries: Vec::new(),
            error: Some("No matches found".to_owned()),
        };

        assert_eq!(
            source_status_message(&source).as_deref(),
            Some("No entry found on source.")
        );
    }

    #[test]
    fn source_status_keeps_real_errors_visible() {
        let source = SourceResult {
            source: Source::Dwds,
            ok: false,
            url: None,
            entries: Vec::new(),
            error: Some("HTTP error: 404 Not Found".to_owned()),
        };

        assert_eq!(
            source_status_message(&source).as_deref(),
            Some("Error: HTTP error: 404 Not Found")
        );
    }

    #[test]
    fn source_status_is_none_for_sources_with_entries() {
        let source = SourceResult::ok(
            Source::Openthesaurus,
            Some(UrlValue::One("https://example.test".to_owned())),
            vec![DictionaryEntry::new(1, "Bank")],
        );

        assert_eq!(source_status_message(&source), None);
    }
}
