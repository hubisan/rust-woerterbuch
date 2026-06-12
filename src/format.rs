use crate::models::{DictionaryEntry, LookupResponse, Sense, Source, SourceResult, SynonymGroup};
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    #[value(name = "human")]
    Human,
    #[value(name = "json")]
    Json,
    #[value(name = "markdown")]
    Markdown,
    #[value(name = "org")]
    Org,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputLayout {
    #[value(name = "by-source")]
    BySource,
    #[value(name = "by-section")]
    BySection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentSection {
    Etymology,
    Synonyms,
    Definitions,
    Idioms,
}

impl ContentSection {
    const ALL: [Self; 4] = [
        Self::Etymology,
        Self::Synonyms,
        Self::Definitions,
        Self::Idioms,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Etymology => "Etymology",
            Self::Synonyms => "Synonyms",
            Self::Definitions => "Definitions",
            Self::Idioms => "Idioms",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextSyntax {
    Human,
    Markdown,
    Org,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IdiomItem {
    reference: Option<String>,
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SynonymItem {
    reference: Option<String>,
    text: String,
}

pub fn render(
    response: &LookupResponse,
    output_format: OutputFormat,
    layout: OutputLayout,
    max_examples: Option<usize>,
) -> serde_json::Result<String> {
    match output_format {
        OutputFormat::Json => render_json(response),
        OutputFormat::Human => Ok(render_text(
            response,
            TextSyntax::Human,
            layout,
            max_examples,
        )),
        OutputFormat::Markdown => Ok(render_text(
            response,
            TextSyntax::Markdown,
            layout,
            max_examples,
        )),
        OutputFormat::Org => Ok(render_text(response, TextSyntax::Org, layout, max_examples)),
    }
}

fn render_json(response: &LookupResponse) -> serde_json::Result<String> {
    let mut output = serde_json::to_string_pretty(response)?;
    output.push('\n');
    Ok(output)
}

fn render_text(
    response: &LookupResponse,
    syntax: TextSyntax,
    layout: OutputLayout,
    max_examples: Option<usize>,
) -> String {
    let mut output = String::new();

    if syntax == TextSyntax::Human {
        output.push_str(&format!("Dictionary lookup: {}\n", response.query));

        if response.results.is_empty() {
            output.push_str("No sources selected.\n");
            return output;
        }
    } else if response.results.is_empty() {
        output.push_str("No sources selected.\n");
        return output;
    }

    match layout {
        OutputLayout::BySource => {
            render_text_by_source(response, syntax, max_examples, &mut output)
        }
        OutputLayout::BySection => {
            render_text_by_section(response, syntax, max_examples, &mut output)
        }
    }

    if !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

fn render_text_by_source(
    response: &LookupResponse,
    syntax: TextSyntax,
    max_examples: Option<usize>,
    output: &mut String,
) {
    for source in &response.results {
        push_heading(output, syntax, 1, source_title(source.source));

        if let Some(message) = source_status_message(source) {
            push_paragraph(output, &message);
            continue;
        }

        if source.entries.is_empty() {
            push_paragraph(output, "No content for requested sections.");
            continue;
        }

        let show_entry_headings = source.entries.len() > 1;
        for entry in &source.entries {
            let section_heading_level = if show_entry_headings { 3 } else { 2 };
            if show_entry_headings {
                push_entry_heading(output, syntax, 2, entry);
                render_entry_metadata(entry, output);
            } else {
                render_entry_metadata(entry, output);
            }

            let mut wrote_section = false;
            for section in ContentSection::ALL {
                let mut section_output = String::new();
                render_entry_section_content(
                    entry,
                    section,
                    syntax,
                    max_examples,
                    &mut section_output,
                );
                if section_output.is_empty() {
                    continue;
                }

                push_heading(output, syntax, section_heading_level, section.title());
                output.push('\n');
                output.push_str(&section_output);
                wrote_section = true;
            }

            if !wrote_section {
                push_paragraph(output, "No content for requested sections.");
            }
        }
    }
}

fn render_text_by_section(
    response: &LookupResponse,
    syntax: TextSyntax,
    max_examples: Option<usize>,
    output: &mut String,
) {
    let mut wrote_any = false;

    for section in ContentSection::ALL {
        let mut wrote_section = false;
        for source in &response.results {
            if source_status_message(source).is_some() {
                continue;
            }

            let entries_with_content: Vec<&DictionaryEntry> = source
                .entries
                .iter()
                .filter(|entry| entry_has_content_for_section(entry, section))
                .collect();
            if entries_with_content.is_empty() {
                continue;
            }

            if !wrote_section {
                push_heading(output, syntax, 1, section.title());
                wrote_any = true;
                wrote_section = true;
            }

            push_heading(output, syntax, 2, source_title(source.source));
            let show_entry_headings = source.entries.len() > 1;
            for entry in entries_with_content {
                if show_entry_headings {
                    push_entry_heading(output, syntax, 3, entry);
                }
                render_entry_section_content(entry, section, syntax, max_examples, output);
            }
        }
    }

    if !wrote_any {
        push_paragraph(output, "No content for requested sections.");
    }
}

fn render_entry_section_content(
    entry: &DictionaryEntry,
    section: ContentSection,
    syntax: TextSyntax,
    max_examples: Option<usize>,
    output: &mut String,
) {
    match section {
        ContentSection::Etymology => {
            if let Some(etymology) = entry.etymology.as_deref().filter(|value| !value.is_empty()) {
                push_paragraph(output, etymology);
            }
        }
        ContentSection::Synonyms => render_synonyms(entry, syntax, output),
        ContentSection::Definitions => render_definitions(entry, syntax, max_examples, output),
        ContentSection::Idioms => render_idioms(entry, syntax, output),
    }
}

fn render_synonyms(entry: &DictionaryEntry, syntax: TextSyntax, output: &mut String) {
    let mut items = Vec::new();

    for group in &entry.synonym_groups {
        if let Some(item) = synonym_group_item(group) {
            items.push(item);
        }
    }

    collect_sense_synonym_items(&entry.senses, &mut items);

    if items.is_empty() {
        return;
    }

    ensure_blank_line(output);
    for item in items {
        push_referenced_bullet(output, syntax, 0, item.reference.as_deref(), &item.text);
    }
}

fn synonym_group_item(group: &SynonymGroup) -> Option<SynonymItem> {
    if group.items.is_empty() {
        return None;
    }

    Some(SynonymItem {
        reference: group.sense.clone(),
        text: group.items.join(", "),
    })
}

fn collect_sense_synonym_items(senses: &[Sense], out: &mut Vec<SynonymItem>) {
    for sense in senses {
        if !sense.synonyms.is_empty() {
            out.push(SynonymItem {
                reference: sense.label.clone(),
                text: sense.synonyms.join(", "),
            });
        }
        collect_sense_synonym_items(&sense.subsenses, out);
    }
}

fn render_definitions(
    entry: &DictionaryEntry,
    syntax: TextSyntax,
    max_examples: Option<usize>,
    output: &mut String,
) {
    for sense in &entry.senses {
        render_sense_definition(sense, syntax, 0, max_examples, output);
    }
}

fn render_sense_definition(
    sense: &Sense,
    syntax: TextSyntax,
    depth: usize,
    max_examples: Option<usize>,
    output: &mut String,
) {
    if !sense_has_definition_section_content(sense) {
        return;
    }

    ensure_blank_line(output);
    let line = sense_definition_line(sense, syntax);
    if !line.is_empty() {
        push_bullet(output, syntax, depth, &line);
    }

    let rendered_examples = sense
        .examples
        .iter()
        .take(max_examples.unwrap_or(usize::MAX));

    if rendered_examples.len() > 0 {
        ensure_blank_line(output);
        push_label(output, syntax, depth + 1, "Examples:");
        for example in rendered_examples {
            push_bullet(output, syntax, depth + 2, example);
        }
    }

    for child in &sense.subsenses {
        render_sense_definition(child, syntax, depth + 1, max_examples, output);
    }
}

fn sense_has_definition_section_content(sense: &Sense) -> bool {
    sense
        .definition
        .as_deref()
        .is_some_and(|value| !value.is_empty())
        || !sense.examples.is_empty()
        || sense
            .subsenses
            .iter()
            .any(sense_has_definition_section_content)
}

fn sense_definition_line(sense: &Sense, syntax: TextSyntax) -> String {
    let label = sense.label.as_deref().unwrap_or_default();
    let definition = sense.definition.as_deref().unwrap_or_default();
    let qualifiers = if sense.qualifiers.is_empty() {
        String::new()
    } else {
        format!(" [{}]", sense.qualifiers.join(", "))
    };

    match (label.is_empty(), definition.is_empty()) {
        (true, true) => String::new(),
        (true, false) => format!("{definition}{qualifiers}"),
        (false, true) => format_label(label, syntax),
        (false, false) => format!("{} {definition}{qualifiers}", format_label(label, syntax)),
    }
}

fn render_idioms(entry: &DictionaryEntry, syntax: TextSyntax, output: &mut String) {
    let idioms = collect_idioms(entry);
    if idioms.is_empty() {
        return;
    }

    ensure_blank_line(output);
    for item in idioms {
        push_referenced_bullet(output, syntax, 0, item.reference.as_deref(), &item.text);
    }
}

fn collect_idioms(entry: &DictionaryEntry) -> Vec<IdiomItem> {
    let mut out: Vec<IdiomItem> = entry
        .idioms
        .iter()
        .filter(|idiom| !idiom.is_empty())
        .map(|idiom| IdiomItem {
            reference: None,
            text: idiom.clone(),
        })
        .collect();

    collect_sense_idioms(&entry.senses, &mut out);
    out
}

fn collect_sense_idioms(senses: &[Sense], out: &mut Vec<IdiomItem>) {
    for sense in senses {
        for idiom in &sense.idioms {
            if idiom.is_empty() {
                continue;
            }
            out.push(IdiomItem {
                reference: sense.label.clone(),
                text: idiom.clone(),
            });
        }
        collect_sense_idioms(&sense.subsenses, out);
    }
}

fn push_referenced_bullet(
    output: &mut String,
    syntax: TextSyntax,
    depth: usize,
    reference: Option<&str>,
    text: &str,
) {
    let text = match reference.filter(|value| !value.is_empty()) {
        Some(reference) => format!("{}: {text}", format_label(reference, syntax)),
        None => text.to_owned(),
    };
    push_bullet(output, syntax, depth, &text);
}

fn push_entry_heading(
    output: &mut String,
    syntax: TextSyntax,
    level: usize,
    entry: &DictionaryEntry,
) {
    push_heading(output, syntax, level, &format!("Entry {}", entry.id));
}

fn push_heading(output: &mut String, syntax: TextSyntax, level: usize, text: &str) {
    ensure_blank_line(output);

    match syntax {
        TextSyntax::Human => match level {
            1 => output.push_str(&format!("== {text} ==\n")),
            2 => output.push_str(&format!("-- {text} --\n")),
            _ => output.push_str(&format!("{text}\n")),
        },
        TextSyntax::Markdown => output.push_str(&format!("{} {text}\n", "#".repeat(level))),
        TextSyntax::Org => output.push_str(&format!("{} {text}\n", "*".repeat(level))),
    }
}

fn push_paragraph(output: &mut String, text: &str) {
    ensure_blank_line(output);
    output.push_str(text);
    output.push('\n');
}

fn push_label(output: &mut String, _syntax: TextSyntax, depth: usize, text: &str) {
    output.push_str(&format!("{}{}\n", "  ".repeat(depth), text));
}

fn push_bullet(output: &mut String, _syntax: TextSyntax, depth: usize, text: &str) {
    output.push_str(&format!("{}- {text}\n", "  ".repeat(depth)));
}

fn ensure_blank_line(output: &mut String) {
    if output.is_empty() || output.ends_with("\n\n") {
        return;
    }

    if !output.ends_with('\n') {
        output.push('\n');
    }
    output.push('\n');
}

fn format_label(label: &str, syntax: TextSyntax) -> String {
    match syntax {
        TextSyntax::Human => format!("'{label}'"),
        TextSyntax::Markdown => format!("`{label}`"),
        TextSyntax::Org => format!("~{label}~"),
    }
}

fn render_entry_metadata(entry: &DictionaryEntry, output: &mut String) {
    let fields = [
        ("Title", entry.title.as_deref()),
        ("Part of speech", entry.part_of_speech.as_deref()),
        ("Grammar", entry.grammar.as_deref()),
    ];

    let mut wrote_any = false;
    for (label, value) in fields {
        let Some(value) = value.filter(|value| !value.is_empty()) else {
            continue;
        };

        if !wrote_any {
            ensure_blank_line(output);
            wrote_any = true;
        }

        output.push_str(&format!("{label}: {value}\n"));
    }
}

fn entry_has_content_for_section(entry: &DictionaryEntry, section: ContentSection) -> bool {
    match section {
        ContentSection::Etymology => entry
            .etymology
            .as_deref()
            .is_some_and(|text| !text.is_empty()),
        ContentSection::Synonyms => {
            !entry.synonym_groups.is_empty() || entry.senses.iter().any(sense_has_synonym_content)
        }
        ContentSection::Definitions => entry
            .senses
            .iter()
            .any(sense_has_definition_section_content),
        ContentSection::Idioms => !collect_idioms(entry).is_empty(),
    }
}

fn sense_has_synonym_content(sense: &Sense) -> bool {
    !sense.synonyms.is_empty() || sense.subsenses.iter().any(sense_has_synonym_content)
}

fn source_title(source: Source) -> &'static str {
    match source {
        Source::Openthesaurus => "Openthesaurus",
        Source::Dwds => "Dwds",
        Source::Duden => "Duden",
        Source::Wiktionary => "Wiktionary",
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

    #[test]
    fn duden_style_sense_idioms_are_collected_with_reference() {
        let entry = DictionaryEntry {
            id: 1,
            headword: "Bank".to_owned(),
            senses: vec![Sense {
                id: 1,
                label: Some("1a".to_owned()),
                idioms: vec!["durch die Bank".to_owned()],
                ..Sense::default()
            }],
            ..DictionaryEntry::default()
        };

        assert_eq!(
            collect_idioms(&entry),
            vec![IdiomItem {
                reference: Some("1a".to_owned()),
                text: "durch die Bank".to_owned(),
            }]
        );
    }

    #[test]
    fn output_layout_uses_new_cli_names() {
        assert_eq!(
            OutputLayout::BySource
                .to_possible_value()
                .expect("value")
                .get_name(),
            "by-source"
        );
        assert_eq!(
            OutputLayout::BySection
                .to_possible_value()
                .expect("value")
                .get_name(),
            "by-section"
        );
    }

    #[test]
    fn markdown_by_source_renders_single_entry_without_heading() {
        let response = LookupResponse {
            query: "Bank".to_owned(),
            results: vec![SourceResult::ok(
                Source::Dwds,
                Some(UrlValue::One("https://example.test".to_owned())),
                vec![DictionaryEntry {
                    id: 1,
                    headword: "Bank".to_owned(),
                    title: Some("Bank, die".to_owned()),
                    part_of_speech: Some("Substantiv".to_owned()),
                    grammar: Some("feminin".to_owned()),
                    etymology: Some("von alt".to_owned()),
                    senses: vec![Sense {
                        id: 1,
                        label: Some("1.".to_owned()),
                        definition: Some("erste Bedeutung".to_owned()),
                        examples: vec!["ein Beispiel".to_owned()],
                        ..Sense::default()
                    }],
                    idioms: vec!["durch die Bank".to_owned()],
                    ..DictionaryEntry::default()
                }],
            )],
        };

        let rendered = render(
            &response,
            OutputFormat::Markdown,
            OutputLayout::BySource,
            None,
        )
        .expect("markdown render");

        assert_eq!(
            rendered,
            "# Dwds\n\nTitle: Bank, die\nPart of speech: Substantiv\nGrammar: feminin\n\n## Etymology\n\nvon alt\n\n## Definitions\n\n- `1.` erste Bedeutung\n\n  Examples:\n    - ein Beispiel\n\n## Idioms\n\n- durch die Bank\n"
        );
    }

    #[test]
    fn markdown_by_source_omits_single_entry_heading() {
        let response = LookupResponse {
            query: "Buch".to_owned(),
            results: vec![SourceResult::ok(
                Source::Dwds,
                Some(UrlValue::One("https://example.test".to_owned())),
                vec![DictionaryEntry {
                    id: 1,
                    headword: "Buch".to_owned(),
                    title: Some("Buch, das".to_owned()),
                    part_of_speech: Some("Substantiv".to_owned()),
                    grammar: Some("neutrum".to_owned()),
                    etymology: Some("von alt".to_owned()),
                    senses: vec![Sense {
                        id: 1,
                        label: Some("1.".to_owned()),
                        definition: Some("ein Werk".to_owned()),
                        ..Sense::default()
                    }],
                    ..DictionaryEntry::default()
                }],
            )],
        };

        let rendered = render(
            &response,
            OutputFormat::Markdown,
            OutputLayout::BySource,
            None,
        )
        .expect("markdown render");

        assert!(!rendered.contains("## Entry 1"));
        assert!(rendered.contains("# Dwds\n\nTitle: Buch, das"));
        assert!(rendered.contains("\n## Etymology\n\nvon alt\n"));
        assert!(rendered.contains("\n## Definitions\n\n- `1.` ein Werk\n"));
    }

    #[test]
    fn markdown_by_section_renders_single_entry_without_heading() {
        let response = LookupResponse {
            query: "Bank".to_owned(),
            results: vec![SourceResult::ok(
                Source::Openthesaurus,
                Some(UrlValue::One("https://example.test".to_owned())),
                vec![DictionaryEntry {
                    id: 1,
                    headword: "Bank".to_owned(),
                    synonym_groups: vec![SynonymGroup {
                        sense: None,
                        categories: Vec::new(),
                        items: vec!["Parkbank".to_owned(), "Sitzbank".to_owned()],
                    }],
                    ..DictionaryEntry::default()
                }],
            )],
        };

        let rendered = render(
            &response,
            OutputFormat::Markdown,
            OutputLayout::BySection,
            None,
        )
        .expect("markdown render");

        assert_eq!(
            rendered,
            "# Synonyms\n\n## Openthesaurus\n\n- Parkbank, Sitzbank\n"
        );
    }

    #[test]
    fn markdown_by_source_keeps_headings_for_multiple_entries() {
        let response = LookupResponse {
            query: "Bank".to_owned(),
            results: vec![SourceResult::ok(
                Source::Dwds,
                Some(UrlValue::Many(vec![
                    "https://example.test/1".to_owned(),
                    "https://example.test/2".to_owned(),
                ])),
                vec![
                    DictionaryEntry {
                        id: 1,
                        headword: "Bank".to_owned(),
                        senses: vec![Sense {
                            id: 1,
                            label: Some("1.".to_owned()),
                            definition: Some("erste Bedeutung".to_owned()),
                            ..Sense::default()
                        }],
                        ..DictionaryEntry::default()
                    },
                    DictionaryEntry {
                        id: 2,
                        headword: "Bank".to_owned(),
                        senses: vec![Sense {
                            id: 1,
                            label: Some("1.".to_owned()),
                            definition: Some("zweite Bedeutung".to_owned()),
                            ..Sense::default()
                        }],
                        ..DictionaryEntry::default()
                    },
                ],
            )],
        };

        let rendered = render(
            &response,
            OutputFormat::Markdown,
            OutputLayout::BySource,
            None,
        )
        .expect("markdown render");

        assert!(rendered.contains("## Entry 1"));
        assert!(rendered.contains("## Entry 2"));
        assert!(rendered.contains("\n### Definitions\n\n- `1.` erste Bedeutung\n"));
        assert!(rendered.contains("\n### Definitions\n\n- `1.` zweite Bedeutung\n"));
    }

    #[test]
    fn markdown_by_section_keeps_headings_for_multiple_entries() {
        let response = LookupResponse {
            query: "Bank".to_owned(),
            results: vec![SourceResult::ok(
                Source::Openthesaurus,
                Some(UrlValue::Many(vec![
                    "https://example.test/1".to_owned(),
                    "https://example.test/2".to_owned(),
                ])),
                vec![
                    DictionaryEntry {
                        id: 1,
                        headword: "Bank".to_owned(),
                        synonym_groups: vec![SynonymGroup {
                            sense: None,
                            categories: Vec::new(),
                            items: vec!["Parkbank".to_owned()],
                        }],
                        ..DictionaryEntry::default()
                    },
                    DictionaryEntry {
                        id: 2,
                        headword: "Bank".to_owned(),
                        synonym_groups: vec![SynonymGroup {
                            sense: None,
                            categories: Vec::new(),
                            items: vec!["Sitzbank".to_owned()],
                        }],
                        ..DictionaryEntry::default()
                    },
                ],
            )],
        };

        let rendered = render(
            &response,
            OutputFormat::Markdown,
            OutputLayout::BySection,
            None,
        )
        .expect("markdown render");

        assert!(rendered.contains("### Entry 1"));
        assert!(rendered.contains("### Entry 2"));
        assert!(rendered.contains("- Parkbank"));
        assert!(rendered.contains("- Sitzbank"));
    }

    #[test]
    fn markdown_by_section_omits_single_entry_heading() {
        let response = LookupResponse {
            query: "Buch".to_owned(),
            results: vec![SourceResult::ok(
                Source::Dwds,
                Some(UrlValue::One("https://example.test".to_owned())),
                vec![DictionaryEntry {
                    id: 1,
                    headword: "Buch".to_owned(),
                    senses: vec![Sense {
                        id: 1,
                        label: Some("1.".to_owned()),
                        definition: Some("ein Werk".to_owned()),
                        ..Sense::default()
                    }],
                    ..DictionaryEntry::default()
                }],
            )],
        };

        let rendered = render(
            &response,
            OutputFormat::Markdown,
            OutputLayout::BySection,
            None,
        )
        .expect("markdown render");

        assert!(!rendered.contains("### Entry 1"));
        assert!(rendered.contains("# Definitions\n\n## Dwds\n\n- `1.` ein Werk\n"));
    }

    #[test]
    fn org_uses_tildes_for_labels() {
        let response = LookupResponse {
            query: "Bank".to_owned(),
            results: vec![SourceResult::ok(
                Source::Duden,
                Some(UrlValue::One("https://example.test".to_owned())),
                vec![DictionaryEntry {
                    id: 1,
                    headword: "Bank".to_owned(),
                    senses: vec![Sense {
                        id: 1,
                        label: Some("1a".to_owned()),
                        definition: Some("erste Bedeutung".to_owned()),
                        ..Sense::default()
                    }],
                    idioms: vec!["durch die Bank".to_owned()],
                    ..DictionaryEntry::default()
                }],
            )],
        };

        let rendered =
            render(&response, OutputFormat::Org, OutputLayout::BySource, None).expect("org render");

        assert!(rendered.contains("- ~1a~ erste Bedeutung"));
    }

    #[test]
    fn human_uses_single_quotes_for_labels() {
        let response = LookupResponse {
            query: "Bank".to_owned(),
            results: vec![SourceResult::ok(
                Source::Duden,
                Some(UrlValue::One("https://example.test".to_owned())),
                vec![DictionaryEntry {
                    id: 1,
                    headword: "Bank".to_owned(),
                    senses: vec![Sense {
                        id: 1,
                        label: Some("1a".to_owned()),
                        definition: Some("erste Bedeutung".to_owned()),
                        ..Sense::default()
                    }],
                    ..DictionaryEntry::default()
                }],
            )],
        };

        let rendered = render(&response, OutputFormat::Human, OutputLayout::BySource, None)
            .expect("human render");

        assert!(rendered.contains("- '1a' erste Bedeutung"));
        assert!(!rendered.contains("- `1a` erste Bedeutung"));
    }

    #[test]
    fn org_uses_tildes_for_references() {
        let response = LookupResponse {
            query: "Bank".to_owned(),
            results: vec![SourceResult::ok(
                Source::Duden,
                Some(UrlValue::One("https://example.test".to_owned())),
                vec![DictionaryEntry {
                    id: 1,
                    headword: "Bank".to_owned(),
                    senses: vec![Sense {
                        id: 1,
                        label: Some("1a".to_owned()),
                        idioms: vec!["durch die Bank".to_owned()],
                        ..Sense::default()
                    }],
                    ..DictionaryEntry::default()
                }],
            )],
        };

        let rendered =
            render(&response, OutputFormat::Org, OutputLayout::BySource, None).expect("org render");

        assert!(rendered.contains("- ~1a~: durch die Bank"));
    }

    #[test]
    fn human_uses_single_quotes_for_references() {
        let response = LookupResponse {
            query: "Bank".to_owned(),
            results: vec![SourceResult::ok(
                Source::Duden,
                Some(UrlValue::One("https://example.test".to_owned())),
                vec![DictionaryEntry {
                    id: 1,
                    headword: "Bank".to_owned(),
                    senses: vec![Sense {
                        id: 1,
                        label: Some("1a".to_owned()),
                        idioms: vec!["durch die Bank".to_owned()],
                        ..Sense::default()
                    }],
                    ..DictionaryEntry::default()
                }],
            )],
        };

        let rendered = render(&response, OutputFormat::Human, OutputLayout::BySource, None)
            .expect("human render");

        assert!(rendered.contains("- '1a': durch die Bank"));
        assert!(!rendered.contains("- `1a`: durch die Bank"));
    }

    #[test]
    fn max_examples_limits_examples_per_definition() {
        let response = LookupResponse {
            query: "Bank".to_owned(),
            results: vec![SourceResult::ok(
                Source::Dwds,
                Some(UrlValue::One("https://example.test".to_owned())),
                vec![DictionaryEntry {
                    id: 1,
                    headword: "Bank".to_owned(),
                    senses: vec![Sense {
                        id: 1,
                        label: Some("1.".to_owned()),
                        definition: Some("erste Bedeutung".to_owned()),
                        examples: vec!["eins".to_owned(), "zwei".to_owned(), "drei".to_owned()],
                        ..Sense::default()
                    }],
                    ..DictionaryEntry::default()
                }],
            )],
        };

        let rendered = render(
            &response,
            OutputFormat::Markdown,
            OutputLayout::BySource,
            Some(2),
        )
        .expect("markdown render");

        assert!(rendered.contains("    - eins\n    - zwei\n"));
        assert!(!rendered.contains("    - drei\n"));
    }

    #[test]
    fn max_examples_zero_omits_example_blocks() {
        let response = LookupResponse {
            query: "Bank".to_owned(),
            results: vec![SourceResult::ok(
                Source::Dwds,
                Some(UrlValue::One("https://example.test".to_owned())),
                vec![DictionaryEntry {
                    id: 1,
                    headword: "Bank".to_owned(),
                    senses: vec![Sense {
                        id: 1,
                        label: Some("1.".to_owned()),
                        definition: Some("erste Bedeutung".to_owned()),
                        examples: vec!["eins".to_owned()],
                        ..Sense::default()
                    }],
                    ..DictionaryEntry::default()
                }],
            )],
        };

        let rendered = render(
            &response,
            OutputFormat::Org,
            OutputLayout::BySource,
            Some(0),
        )
        .expect("org render");

        assert!(rendered.contains("- ~1.~ erste Bedeutung"));
        assert!(!rendered.contains("Examples:"));
        assert!(!rendered.contains("- eins"));
    }

    #[test]
    fn json_output_ignores_max_examples() {
        let response = LookupResponse {
            query: "Bank".to_owned(),
            results: vec![SourceResult::ok(
                Source::Dwds,
                Some(UrlValue::One("https://example.test".to_owned())),
                vec![DictionaryEntry {
                    id: 1,
                    headword: "Bank".to_owned(),
                    senses: vec![Sense {
                        id: 1,
                        label: Some("1.".to_owned()),
                        definition: Some("erste Bedeutung".to_owned()),
                        examples: vec!["eins".to_owned(), "zwei".to_owned()],
                        ..Sense::default()
                    }],
                    ..DictionaryEntry::default()
                }],
            )],
        };

        let full = render(&response, OutputFormat::Json, OutputLayout::BySource, None)
            .expect("json render");
        let limited = render(
            &response,
            OutputFormat::Json,
            OutputLayout::BySource,
            Some(1),
        )
        .expect("json render");
        let parsed: serde_json::Value = serde_json::from_str(&limited).expect("valid json");

        assert_eq!(full, limited);
        assert_eq!(
            parsed["results"][0]["entries"][0]["senses"][0]["examples"],
            serde_json::json!(["eins", "zwei"])
        );
    }
}
