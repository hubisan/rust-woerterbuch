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

    if !source.ok {
        println!(
            "Error: {}",
            source.error.as_deref().unwrap_or("unknown error")
        );
        return;
    }

    if source.entries.is_empty() {
        println!("No results.");
        return;
    }

    for entry in &source.entries {
        print_entry(entry);
    }
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
