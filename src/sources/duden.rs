use crate::models::{DictionaryEntry, Sense, Source, SourceResult, SynonymGroup, UrlValue};
use anyhow::{anyhow, Result};
use futures::future::try_join_all;
use reqwest::{Client, StatusCode};
use scraper::{ElementRef, Html, Selector};

const DUDEN_ENTRY_BASE: &str = "https://www.duden.de/rechtschreibung/";
const DUDEN_SEARCH_BASE: &str = "https://www.duden.de/suchen/dudenonline/";
const DUDEN_BASE: &str = "https://www.duden.de";

pub async fn lookup(client: &Client, query: &str) -> Result<SourceResult> {
    let entry_url = build_url(query);
    let (status, body) = fetch_response(client, &entry_url).await?;

    if status == StatusCode::NOT_FOUND {
        return lookup_via_search(client, query).await;
    }
    if !status.is_success() {
        return Err(http_error(status, &body));
    }

    match parse_entry(query, &entry_url, &body, 1) {
        Some(entry) => Ok(SourceResult::ok(
            Source::Duden,
            Some(UrlValue::One(entry_url)),
            vec![entry],
        )),
        None => Ok(no_match_result()),
    }
}

pub fn build_url(lemma: &str) -> String {
    let normalized = lemma.split_whitespace().collect::<Vec<_>>().join("_");
    format!(
        "{}{normalized}?amp",
        DUDEN_ENTRY_BASE,
        normalized = urlencoding::encode(&normalized)
    )
}

pub fn build_search_url(lemma: &str) -> String {
    format!(
        "{}{lemma}",
        DUDEN_SEARCH_BASE,
        lemma = urlencoding::encode(lemma)
    )
}

async fn lookup_via_search(client: &Client, query: &str) -> Result<SourceResult> {
    let search_url = build_search_url(query);
    let (status, body) = fetch_response(client, &search_url).await?;

    if status == StatusCode::NOT_FOUND {
        return Ok(no_match_result());
    }
    if !status.is_success() {
        return Err(http_error(status, &body));
    }

    let urls = parse_search_results(&Html::parse_document(&body), query);
    if urls.is_empty() {
        return Ok(no_match_result());
    }

    let pages = try_join_all(urls.iter().map(|url| async move {
        let (status, body) = fetch_response(client, url).await?;
        if !status.is_success() {
            return Err(http_error(status, &body));
        }
        Ok::<String, anyhow::Error>(body)
    }))
    .await?;

    let entries = pages
        .iter()
        .enumerate()
        .filter_map(|(index, body)| parse_entry(query, &urls[index], body, index + 1))
        .collect::<Vec<_>>();

    if entries.is_empty() {
        return Ok(no_match_result());
    }

    Ok(SourceResult::ok(
        Source::Duden,
        Some(one_or_many_urls(urls)),
        entries,
    ))
}

async fn fetch_response(client: &Client, url: &str) -> Result<(StatusCode, String)> {
    let response = client.get(url).send().await?;
    let status = response.status();
    let body = response.text().await?;
    Ok((status, body))
}

fn parse_entry(query: &str, url: &str, html: &str, id: usize) -> Option<DictionaryEntry> {
    let document = Html::parse_document(html);
    let title_node = extract_title_node(&document);
    let lemma = extract_lemma(title_node.as_ref(), query);
    let title = extract_title(title_node.as_ref(), &lemma);
    let grammar = field_value(&document, "Wortart");
    let part_of_speech = grammar.as_deref().and_then(wortart_from_grammar);
    let etymology = extract_origin(&document);
    let synonyms = extract_synonyms(&document);
    let senses = parse_definitions(&document);

    let mut entry = DictionaryEntry::new(id, lemma);
    entry.title = Some(title);
    entry.part_of_speech = part_of_speech;
    entry.grammar = grammar;
    entry.etymology = etymology;
    entry.url = Some(url.to_owned());
    if !synonyms.is_empty() {
        entry.synonym_groups.push(SynonymGroup::items(synonyms));
    }
    entry.senses = senses;

    (!entry.is_empty()).then_some(entry)
}

fn parse_definitions(document: &Html) -> Vec<Sense> {
    let Some(section) = section_by_id(document, &["bedeutungen", "bedeutung"]) else {
        return Vec::new();
    };

    if let Some(list) = direct_child_by_tag_and_class(&section, "ol", Some("enumeration")) {
        let items = direct_children_by_tag_and_class(&list, "li", Some("enumeration__item"));
        if !items.is_empty() {
            return items
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    extract_definition_node(&item, index + 1, (index + 1).to_string())
                })
                .collect();
        }
    }

    parse_single_definition_section(&section)
}

fn parse_single_definition_section(section: &ElementRef<'_>) -> Vec<Sense> {
    let notes = notes(section);
    let definition = direct_child_by_tag_and_class(section, "p", None)
        .map(|node| text(&node))
        .filter(|value| !value.is_empty())
        .or_else(|| {
            direct_child_by_tag_and_class(section, "div", Some("enumeration__text"))
                .map(|node| text(&node))
                .filter(|value| !value.is_empty())
        })
        .or_else(|| extract_shortform_definition(section));

    definition
        .map(|definition| {
            let mut sense = Sense::simple(1, definition);
            sense.label = Some("1".to_owned());
            sense.qualifiers = extract_qualifiers(section);
            sense.examples = note_values(&notes, &["Beispiele", "Beispiel"]);
            sense.idioms = note_values(&notes, &["Wendungen, Redensarten, Sprichwörter"]);
            sense.image_url = extract_image_url(section);
            vec![sense]
        })
        .unwrap_or_default()
}

fn extract_definition_node(node: &ElementRef<'_>, id: usize, label: String) -> Sense {
    let notes = notes(node);
    let definition = direct_child_by_tag_and_class(node, "div", Some("enumeration__text"))
        .map(|child| text(&child))
        .filter(|value| !value.is_empty())
        .or_else(|| extract_shortform_definition(node));

    let subsenses = direct_child_by_tag_and_class(node, "ol", Some("enumeration__sub"))
        .map(|list| {
            direct_children_by_tag_and_class(&list, "li", Some("enumeration__sub-item"))
                .into_iter()
                .enumerate()
                .map(|(index, child)| {
                    let fallback = format!("{label}{}", (b'a' + index as u8) as char);
                    extract_definition_node(&child, index + 1, definition_label(&child, &fallback))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut sense = Sense {
        id,
        source_id: node.value().attr("id").map(str::to_owned),
        label: Some(label),
        definition,
        qualifiers: extract_qualifiers(node),
        examples: note_values(&notes, &["Beispiele", "Beispiel"]),
        idioms: note_values(&notes, &["Wendungen, Redensarten, Sprichwörter"]),
        synonyms: Vec::new(),
        image_url: extract_image_url(node),
        subsenses,
    };

    if sense.definition.is_none()
        && sense.qualifiers.is_empty()
        && sense.examples.is_empty()
        && sense.idioms.is_empty()
        && sense.image_url.is_none()
        && sense.subsenses.is_empty()
    {
        sense.definition = None;
    }

    sense
}

fn parse_search_results(document: &Html, lemma: &str) -> Vec<String> {
    let Some(segment) = find_search_segment(document) else {
        return Vec::new();
    };

    let mut urls = Vec::new();
    for section in direct_children_by_tag_and_class(&segment, "section", Some("vignette")) {
        let Some(label) =
            find_first_descendant(&section, &|node| has_class(node, "vignette__label"))
        else {
            continue;
        };
        let Some(href) = label.value().attr("href") else {
            continue;
        };
        if !href.starts_with("/rechtschreibung/") {
            continue;
        }

        let visible = search_result_lemma(&label);
        if clean_text(&visible) == clean_text(lemma) {
            urls.push(ensure_amp_url(&absolute_url(href)));
        }
    }

    urls
}

fn find_search_segment(document: &Html) -> Option<ElementRef<'_>> {
    let selector = selector("div.segment");
    document.select(&selector).find(|segment| {
        find_first_descendant(segment, &|node| has_class(node, "segment__title"))
            .map(|title| text(&title) == "Wörterbuch")
            .unwrap_or(false)
    })
}

fn search_result_lemma(label: &ElementRef<'_>) -> String {
    find_first_descendant(label, &|node| node.value().name() == "strong")
        .map(|strong| text(&strong))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| text(label))
}

fn extract_title_node(document: &Html) -> Option<ElementRef<'_>> {
    document.select(&selector("h1.lemma__title")).next()
}

fn extract_lemma(title_node: Option<&ElementRef<'_>>, fallback: &str) -> String {
    title_node
        .and_then(|node| find_first_descendant(node, &|child| has_class(child, "lemma__main")))
        .map(|node| text(&node))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_owned())
}

fn extract_title(title_node: Option<&ElementRef<'_>>, fallback: &str) -> String {
    title_node
        .map(text)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_owned())
}

fn field_value(document: &Html, wanted_label: &str) -> Option<String> {
    for tuple in document.select(&selector("dl.tuple")) {
        let key = find_first_descendant(&tuple, &|node| node.value().name() == "dt")
            .map(|node| normalize_key(&text(&node)))
            .unwrap_or_default();
        if key != wanted_label {
            continue;
        }

        let value = find_first_descendant(&tuple, &|node| node.value().name() == "dd")
            .map(|node| text(&node))
            .unwrap_or_default();
        if !value.is_empty() {
            return Some(value);
        }
    }

    None
}

fn wortart_from_grammar(grammar: &str) -> Option<String> {
    let value = grammar.split(',').next().unwrap_or_default().trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn extract_origin(document: &Html) -> Option<String> {
    let section = section_by_id(document, &["herkunft"])?;
    let parts = element_children(&section)
        .into_iter()
        .filter(|child| !matches!(child.value().name(), "header" | "small" | "nav"))
        .map(|child| text(&child))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    (!parts.is_empty()).then(|| parts.join(" "))
}

fn extract_synonyms(document: &Html) -> Vec<String> {
    let Some(section) = section_by_id(document, &["synonyme"]) else {
        return Vec::new();
    };
    let Some(list) = direct_child_by_tag_and_class(&section, "ul", None) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for item in direct_children_by_tag_and_class(&list, "li", None) {
        for synonym in split_synonym_text(&text(&item)) {
            if !out.contains(&synonym) {
                out.push(synonym);
            }
        }
    }
    out
}

fn split_synonym_text(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;

    for ch in input.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' | ';' if depth == 0 => {
                let value = clean_text(&current);
                if !value.is_empty() {
                    parts.push(value);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let value = clean_text(&current);
    if !value.is_empty() {
        parts.push(value);
    }

    parts
}

fn extract_image_url(node: &ElementRef<'_>) -> Option<String> {
    let figure = direct_child_by_tag_and_class(node, "figure", Some("depiction"))?;
    find_first_descendant(&figure, &|child| child.value().name() == "a")
        .and_then(|link| link.value().attr("href"))
        .map(str::to_owned)
        .filter(|href| !href.is_empty())
}

fn extract_qualifiers(node: &ElementRef<'_>) -> Vec<String> {
    tuple_pairs(node)
        .into_iter()
        .filter(|(key, _)| key != "Kurzform für")
        .map(|(key, value)| format!("{key}: {value}"))
        .collect()
}

fn extract_shortform_definition(node: &ElementRef<'_>) -> Option<String> {
    let first_child = element_children(node).into_iter().next()?;
    if first_child.value().name() != "dl" || !has_class(&first_child, "tuple") {
        return None;
    }

    let (key, value) = tuple_pairs(node).into_iter().next()?;
    (key == "Kurzform für").then(|| format!("{key}: {value}"))
}

fn tuple_pairs(node: &ElementRef<'_>) -> Vec<(String, String)> {
    let mut pairs = Vec::new();

    for dl in children_with_class(node, "tuple") {
        if dl.value().name() != "dl" {
            continue;
        }

        let key = find_first_descendant(&dl, &|child| child.value().name() == "dt")
            .map(|dt| normalize_key(&text(&dt)))
            .unwrap_or_default();
        let value = find_first_descendant(&dl, &|child| child.value().name() == "dd")
            .map(|dd| text(&dd))
            .unwrap_or_default();

        if !key.is_empty() && !value.is_empty() {
            pairs.push((key, value));
        }
    }

    pairs
}

#[derive(Debug, Clone)]
struct Note {
    title: String,
    items: Vec<String>,
}

fn notes(node: &ElementRef<'_>) -> Vec<Note> {
    let mut out = Vec::new();

    for dl in children_with_class(node, "note") {
        if dl.value().name() != "dl" {
            continue;
        }

        let title = find_first_descendant(&dl, &|child| child.value().name() == "dt")
            .map(|dt| normalize_key(&text(&dt)))
            .unwrap_or_default();
        if title.is_empty() {
            continue;
        }

        let items = dl
            .select(&selector("li"))
            .map(|li| text(&li))
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();

        out.push(Note { title, items });
    }

    out
}

fn note_values(notes: &[Note], titles: &[&str]) -> Vec<String> {
    notes
        .iter()
        .find(|note| titles.iter().any(|title| *title == note.title))
        .map(|note| note.items.clone())
        .unwrap_or_default()
}

fn definition_label(node: &ElementRef<'_>, fallback: &str) -> String {
    let Some(raw_id) = node.value().attr("id") else {
        return fallback.to_owned();
    };
    raw_id
        .strip_prefix("Bedeutung-")
        .filter(|value| {
            !value.is_empty()
                && value
                    .chars()
                    .all(|ch| ch.is_ascii_digit() || ch.is_ascii_lowercase())
        })
        .map(str::to_owned)
        .unwrap_or_else(|| fallback.to_owned())
}

fn section_by_id<'a>(document: &'a Html, ids: &[&str]) -> Option<ElementRef<'a>> {
    ids.iter().find_map(|id| {
        let selector = selector(&format!(r#"div[id="{id}"], section[id="{id}"]"#));
        document.select(&selector).next()
    })
}

fn normalize_key(input: &str) -> String {
    clean_text(input)
        .replace(" ⓘ", "")
        .trim_end_matches(':')
        .trim()
        .to_owned()
}

fn text(node: &ElementRef<'_>) -> String {
    clean_text(&node.text().collect::<Vec<_>>().join(" "))
}

fn clean_text(input: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;

    for ch in input.chars() {
        let ch = match ch {
            '\u{00a0}' => ' ',
            '〈' => '⟨',
            '〉' => '⟩',
            _ => ch,
        };

        if ch.is_whitespace() {
            pending_space = !out.is_empty();
            continue;
        }

        if pending_space && !matches!(ch, ',' | '.' | ')' | ';' | ':') && !out.ends_with('(') {
            out.push(' ');
        }

        out.push(ch);
        pending_space = false;
    }

    out.trim().to_owned()
}

fn children_with_class<'a>(node: &'a ElementRef<'a>, class: &str) -> Vec<ElementRef<'a>> {
    element_children(node)
        .into_iter()
        .filter(|child| has_class(child, class))
        .collect()
}

fn element_children<'a>(node: &'a ElementRef<'a>) -> Vec<ElementRef<'a>> {
    node.children().filter_map(ElementRef::wrap).collect()
}

fn direct_child_by_tag_and_class<'a>(
    node: &'a ElementRef<'a>,
    tag: &str,
    class: Option<&str>,
) -> Option<ElementRef<'a>> {
    direct_children_by_tag_and_class(node, tag, class)
        .into_iter()
        .next()
}

fn direct_children_by_tag_and_class<'a>(
    node: &'a ElementRef<'a>,
    tag: &str,
    class: Option<&str>,
) -> Vec<ElementRef<'a>> {
    element_children(node)
        .into_iter()
        .filter(|child| child.value().name() == tag)
        .filter(|child| class.is_none_or(|class| has_class(child, class)))
        .collect()
}

fn has_class(node: &ElementRef<'_>, class: &str) -> bool {
    node.value()
        .attr("class")
        .unwrap_or_default()
        .split_whitespace()
        .any(|candidate| candidate == class)
}

fn find_first_descendant<'a>(
    node: &'a ElementRef<'a>,
    predicate: &dyn Fn(&ElementRef<'_>) -> bool,
) -> Option<ElementRef<'a>> {
    let mut descendants = node.descendants().filter_map(ElementRef::wrap);
    let _ = descendants.next();
    descendants.find(|child| predicate(child))
}

fn absolute_url(href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_owned()
    } else {
        format!("{DUDEN_BASE}{href}")
    }
}

fn ensure_amp_url(url: &str) -> String {
    if url.contains("?amp") || url.contains("&amp") {
        url.to_owned()
    } else if url.contains('?') {
        format!("{url}&amp")
    } else {
        format!("{url}?amp")
    }
}

fn one_or_many_urls(urls: Vec<String>) -> UrlValue {
    if urls.len() == 1 {
        UrlValue::One(urls.into_iter().next().unwrap_or_default())
    } else {
        UrlValue::Many(urls)
    }
}

fn no_match_result() -> SourceResult {
    SourceResult::error(Source::Duden, "No matches found")
}

fn http_error(status: StatusCode, body: &str) -> anyhow::Error {
    let preview = body.chars().take(200).collect::<String>();
    anyhow!("HTTP error: {} {}", status.as_u16(), preview)
}

fn selector(input: &str) -> Selector {
    Selector::parse(input).expect("valid selector")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    struct SnapshotCase {
        word: &'static str,
        response: SourceResult,
        expected: &'static str,
    }

    #[test]
    fn matches_expected_snapshots_for_local_fixtures() {
        let cases = [
            SnapshotCase {
                word: "Bank",
                response: parse_bank_fixture(),
                expected: include_str!("../../tests/snapshots/duden/Bank.snap"),
            },
            SnapshotCase {
                word: "Haus",
                response: parse_single_fixture(
                    "Haus",
                    &build_url("Haus"),
                    include_str!("../../../woerterbuch/tests/files/duden/Haus/duden-Haus.html"),
                ),
                expected: include_str!("../../tests/snapshots/duden/Haus.snap"),
            },
            SnapshotCase {
                word: "verlieben",
                response: parse_single_fixture(
                    "verlieben",
                    &build_url("verlieben"),
                    include_str!(
                        "../../../woerterbuch/tests/files/duden/verlieben/duden-verlieben.html"
                    ),
                ),
                expected: include_str!("../../tests/snapshots/duden/verlieben.snap"),
            },
            SnapshotCase {
                word: "springen",
                response: parse_single_fixture(
                    "springen",
                    &build_url("springen"),
                    include_str!(
                        "../../../woerterbuch/tests/files/duden/springen/duden-springen.html"
                    ),
                ),
                expected: include_str!("../../tests/snapshots/duden/springen.snap"),
            },
            SnapshotCase {
                word: "Wolke",
                response: parse_single_fixture(
                    "Wolke",
                    &build_url("Wolke"),
                    include_str!("../../../woerterbuch/tests/files/duden/Wolke/duden-Wolke.html"),
                ),
                expected: include_str!("../../tests/snapshots/duden/Wolke.snap"),
            },
            SnapshotCase {
                word: "Zaun",
                response: parse_single_fixture(
                    "Zaun",
                    &build_url("Zaun"),
                    include_str!("../../../woerterbuch/tests/files/duden/Zaun/duden-Zaun.html"),
                ),
                expected: include_str!("../../tests/snapshots/duden/Zaun.snap"),
            },
            SnapshotCase {
                word: "Nixdaexistiert",
                response: no_match_result(),
                expected: include_str!("../../tests/snapshots/duden/Nixdaexistiert.snap"),
            },
        ];

        for case in cases {
            assert_eq!(
                render_snapshot(&case.response),
                case.expected.trim_end(),
                "snapshot mismatch for {}",
                case.word
            );
        }
    }

    #[test]
    fn search_results_only_keep_exact_duden_homographs() {
        let urls = parse_search_results(
            &Html::parse_document(include_str!(
                "../../../woerterbuch/tests/files/duden/Bank/duden-Bank-search.html"
            )),
            "Bank",
        );

        assert_eq!(
            urls,
            vec![
                "https://www.duden.de/rechtschreibung/Bank_Sitzgelegenheit?amp".to_owned(),
                "https://www.duden.de/rechtschreibung/Bank_Geldinstitut?amp".to_owned(),
            ]
        );
    }

    #[test]
    fn bank_fixture_keeps_nested_labels_examples_and_idioms() {
        let response = parse_bank_fixture();
        assert!(response.ok);
        assert!(matches!(response.url, Some(UrlValue::Many(_))));
        assert_eq!(response.entries.len(), 2);

        let seat_entry = &response.entries[0];
        assert_eq!(seat_entry.headword, "Bank");
        assert_eq!(seat_entry.senses[0].label.as_deref(), Some("1"));
        assert_eq!(
            seat_entry.senses[0].subsenses[0].label.as_deref(),
            Some("1a")
        );
        assert_eq!(
            seat_entry.senses[0].subsenses[0].examples[0],
            "sich auf eine Bank setzen"
        );
        assert_eq!(
            seat_entry.senses[0].subsenses[0].idioms[0],
            "etwas auf die lange Bank schieben (umgangssprachlich: etwas Unangenehmes aufschieben, hinauszögern: er schob den Arztbesuch auf die lange Bank; eigentlich = bis zur Bearbeitung in den langen Aktentruhen der Gerichte aufbewahren lassen)"
        );

        let money_entry = &response.entries[1];
        assert_eq!(
            money_entry.synonym_groups[0].items,
            vec![
                "Bankhaus".to_owned(),
                "Geldinstitut".to_owned(),
                "Kreditanstalt".to_owned(),
                "Kreditinstitut".to_owned(),
            ]
        );
    }

    fn parse_bank_fixture() -> SourceResult {
        let urls = parse_search_results(
            &Html::parse_document(include_str!(
                "../../../woerterbuch/tests/files/duden/Bank/duden-Bank-search.html"
            )),
            "Bank",
        );

        let pages = [
            include_str!("../../../woerterbuch/tests/files/duden/Bank/duden-Bank-1.html"),
            include_str!("../../../woerterbuch/tests/files/duden/Bank/duden-Bank-2.html"),
        ];

        let entries = pages
            .into_iter()
            .enumerate()
            .filter_map(|(index, html)| parse_entry("Bank", &urls[index], html, index + 1))
            .collect::<Vec<_>>();

        SourceResult::ok(Source::Duden, Some(UrlValue::Many(urls)), entries)
    }

    fn parse_single_fixture(word: &str, url: &str, html: &str) -> SourceResult {
        let entry = parse_entry(word, url, html, 1).expect("fixture should produce one entry");
        SourceResult::ok(
            Source::Duden,
            Some(UrlValue::One(url.to_owned())),
            vec![entry],
        )
    }

    fn render_snapshot(response: &SourceResult) -> String {
        let mut lines = vec![
            format!("source={:?}", response.source),
            format!("ok={}", response.ok),
            format!("url={}", render_url(response.url.as_ref())),
        ];

        if let Some(error) = &response.error {
            lines.push(format!("error={error}"));
        }

        for entry in &response.entries {
            lines.push(format!(
                "entry {} headword={} title={} part_of_speech={} grammar={} url={}",
                entry.id,
                entry.headword,
                entry.title.as_deref().unwrap_or("-"),
                entry.part_of_speech.as_deref().unwrap_or("-"),
                entry.grammar.as_deref().unwrap_or("-"),
                entry.url.as_deref().unwrap_or("-"),
            ));

            if let Some(etymology) = &entry.etymology {
                lines.push(format!("etymology={etymology}"));
            }

            for group in &entry.synonym_groups {
                lines.push(format!(
                    "synonyms sense={} items=[{}]",
                    group.sense.as_deref().unwrap_or("-"),
                    group.items.join(" | ")
                ));
            }

            for sense in &entry.senses {
                render_sense(&mut lines, sense, 0);
            }
        }

        lines.join("\n")
    }

    fn render_sense(lines: &mut Vec<String>, sense: &Sense, depth: usize) {
        let prefix = "  ".repeat(depth);
        lines.push(format!(
            "{prefix}sense {} source_id={} label={} definition={}",
            sense.id,
            sense.source_id.as_deref().unwrap_or("-"),
            sense.label.as_deref().unwrap_or("-"),
            sense.definition.as_deref().unwrap_or("-"),
        ));

        if !sense.qualifiers.is_empty() {
            lines.push(format!(
                "{prefix}qualifiers=[{}]",
                sense.qualifiers.join(" | ")
            ));
        }
        if !sense.examples.is_empty() {
            lines.push(format!("{prefix}examples=[{}]", sense.examples.join(" | ")));
        }
        if !sense.idioms.is_empty() {
            lines.push(format!("{prefix}idioms=[{}]", sense.idioms.join(" | ")));
        }
        if let Some(image_url) = &sense.image_url {
            lines.push(format!("{prefix}image={image_url}"));
        }

        for subsense in &sense.subsenses {
            render_sense(lines, subsense, depth + 1);
        }
    }

    fn render_url(url: Option<&UrlValue>) -> String {
        match url {
            Some(UrlValue::One(url)) => url.to_owned(),
            Some(UrlValue::Many(urls)) => urls.join(" | "),
            None => "-".to_owned(),
        }
    }
}
