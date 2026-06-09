use crate::models::{dedupe, DictionaryEntry, Sense, Source, SourceResult, SynonymGroup, UrlValue};
use anyhow::Result;
use reqwest::{Client, StatusCode};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;

const WORD_CLASSES: &[&str] = &[
    "Substantiv",
    "Verb",
    "Adjektiv",
    "Adverb",
    "Pronomen",
    "Präposition",
    "Konjunktion",
];

pub async fn lookup(client: &Client, query: &str) -> Result<SourceResult> {
    let encoded = urlencoding::encode(query);
    let api_url = format!("https://de.wiktionary.org/api/rest_v1/page/html/{encoded}");
    let page_url = format!("https://de.wiktionary.org/wiki/{encoded}");

    let response = client.get(&api_url).send().await?;
    let status = response.status();
    let body = response.text().await?;

    if status == StatusCode::NOT_FOUND {
        return Ok(not_found_result(query, &page_url));
    }

    response_error(status, &body)?;
    parse(query, &page_url, &body)
}

pub fn parse(query: &str, page_url: &str, html: &str) -> Result<SourceResult> {
    let document = Html::parse_document(html);
    let lemma = extract_page_title(&document).unwrap_or_else(|| query.to_owned());
    let entries = extract_entries(&document, &lemma, page_url);

    if entries.is_empty() {
        Ok(not_found_result(&lemma, page_url))
    } else {
        Ok(SourceResult::ok(
            Source::Wiktionary,
            Some(UrlValue::One(page_url.to_owned())),
            entries,
        ))
    }
}

fn response_error(status: StatusCode, body: &str) -> Result<()> {
    if status.is_success() {
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "HTTP error: {} {}",
        status.as_u16(),
        body.chars().take(200).collect::<String>()
    ))
}

fn not_found_result(query: &str, page_url: &str) -> SourceResult {
    SourceResult {
        source: Source::Wiktionary,
        ok: false,
        url: Some(UrlValue::One(page_url.to_owned())),
        entries: Vec::new(),
        error: Some(format!("No matches found for {query}").replace(&format!(" for {query}"), "")),
    }
}

fn extract_entries(document: &Html, lemma: &str, page_url: &str) -> Vec<DictionaryEntry> {
    let section_selector = Selector::parse("section").expect("valid selector");
    let mut entries = Vec::new();

    for language_section in document.select(&section_selector).filter(|section| {
        section_heading(section, "h2").is_some_and(|text| is_german_heading(&text))
    }) {
        for entry_section in child_sections(&language_section) {
            let Some(heading) = section_heading(&entry_section, "h3") else {
                continue;
            };
            if !supported_heading(&heading) {
                continue;
            }

            let blocks = collect_labeled_blocks(&entry_section);
            let entry = parse_entry(lemma, page_url, entries.len() + 1, &heading, &blocks);
            if !entry.is_empty() {
                entries.push(entry);
            }
        }
    }

    entries
}

fn parse_entry(
    lemma: &str,
    page_url: &str,
    id: usize,
    heading: &str,
    blocks: &[Block],
) -> DictionaryEntry {
    let mut entry = DictionaryEntry::new(id, lemma);
    entry.title = Some(format!("{lemma}, {heading}"));
    entry.part_of_speech = extract_word_class(heading);
    entry.grammar = Some(heading.to_owned());
    entry.url = Some(page_url.to_owned());
    entry.etymology = parse_origin(blocks);
    entry.idioms = parse_idioms(blocks);
    entry.synonym_groups = parse_synonyms(blocks);
    entry.senses = parse_senses(blocks);
    entry
}

fn section_heading(section: &ElementRef<'_>, tag: &str) -> Option<String> {
    let selector = Selector::parse(tag).expect("valid selector");
    section
        .children()
        .filter_map(ElementRef::wrap)
        .find(|child| child.value().name() == tag)
        .or_else(|| section.select(&selector).next())
        .map(|heading| clean_text(&heading.text().collect::<Vec<_>>().join(" ")))
        .filter(|text| !text.is_empty())
}

fn child_sections<'a>(section: &'a ElementRef<'a>) -> Vec<ElementRef<'a>> {
    section
        .children()
        .filter_map(ElementRef::wrap)
        .filter(|child| child.value().name() == "section")
        .collect()
}

fn is_german_heading(text: &str) -> bool {
    text.contains("(Deutsch)")
}

fn supported_heading(heading: &str) -> bool {
    extract_word_class(heading).is_some()
}

fn extract_word_class(heading: &str) -> Option<String> {
    let first = heading.split(',').next()?.trim();
    WORD_CLASSES
        .iter()
        .find(|candidate| first == **candidate)
        .map(|candidate| (*candidate).to_owned())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockLabel {
    Definitions,
    Examples,
    Synonyms,
    RelatedSynonyms,
    Idioms,
    Origin,
}

#[derive(Debug)]
struct Block {
    label: BlockLabel,
    html: String,
}

fn collect_labeled_blocks(section: &ElementRef<'_>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current_label = None;
    let mut current_html = String::new();
    let mut seen_entry_heading = false;

    for child in section.children().filter_map(ElementRef::wrap) {
        if matches!(child.value().name(), "h2" | "h3" | "h4" | "h5" | "h6") {
            if !seen_entry_heading {
                seen_entry_heading = true;
                continue;
            }
            break;
        }

        if is_heading_like_node(&child) {
            if let Some(previous) = current_label.take() {
                blocks.push(Block {
                    label: previous,
                    html: std::mem::take(&mut current_html),
                });
            }
            current_label = heading_like_label(&child);
            continue;
        }

        if current_label.is_some() {
            current_html.push_str(&child.html());
        }
    }

    if let Some(label) = current_label {
        blocks.push(Block {
            label,
            html: current_html,
        });
    }

    blocks
}

fn is_heading_like_node(node: &ElementRef<'_>) -> bool {
    if !matches!(node.value().name(), "p" | "div") {
        return false;
    }

    let text = clean_text(&node.text().collect::<Vec<_>>().join(" "));
    let style = node.value().attr("style").unwrap_or_default();
    let title = node.value().attr("title");

    text.ends_with(':') && (style.contains("font-weight:bold") || title.is_some())
}

fn heading_like_label(node: &ElementRef<'_>) -> Option<BlockLabel> {
    if !is_heading_like_node(node) {
        return None;
    }

    let text = clean_text(&node.text().collect::<Vec<_>>().join(" "));
    let normalized = text.trim_end_matches(':');

    match normalized {
        "Bedeutungen" => Some(BlockLabel::Definitions),
        "Beispiele" => Some(BlockLabel::Examples),
        "Synonyme" => Some(BlockLabel::Synonyms),
        "Sinnverwandte Wörter" => Some(BlockLabel::RelatedSynonyms),
        "Redewendungen" => Some(BlockLabel::Idioms),
        "Herkunft" => Some(BlockLabel::Origin),
        _ => None,
    }
}

fn parse_senses(blocks: &[Block]) -> Vec<Sense> {
    let definition_pairs = block_items(blocks, BlockLabel::Definitions)
        .into_iter()
        .filter_map(|text| parse_sense_text(&text))
        .collect::<Vec<_>>();
    let example_pairs = block_items(blocks, BlockLabel::Examples)
        .into_iter()
        .filter_map(|text| parse_sense_text(&text))
        .collect::<Vec<_>>();

    let mut example_map: HashMap<String, Vec<String>> = HashMap::new();
    for (label, example) in example_pairs {
        for expanded in expand_sense_label(&label) {
            example_map
                .entry(expanded)
                .or_default()
                .push(example.clone());
        }
    }

    definition_pairs
        .into_iter()
        .enumerate()
        .map(|(index, (label, definition))| {
            let mut sense = Sense::simple(index + 1, definition);
            sense.label = Some(label.clone());
            sense.examples = dedupe(example_map.remove(&label).unwrap_or_default());
            sense
        })
        .collect()
}

fn parse_origin(blocks: &[Block]) -> Option<String> {
    let fragments = fragments_for_label(blocks, BlockLabel::Origin);
    if fragments.is_empty() {
        return None;
    }

    let texts = fragments
        .into_iter()
        .flat_map(|fragment| {
            let items = list_item_texts(&fragment);
            if items.is_empty() {
                plain_texts(&fragment)
            } else {
                items
            }
        })
        .collect::<Vec<_>>();

    (!texts.is_empty()).then(|| clean_origin_text(&texts.join(" ")))
}

fn parse_idioms(blocks: &[Block]) -> Vec<String> {
    let mut idioms = Vec::new();

    for fragment in fragments_for_label(blocks, BlockLabel::Idioms) {
        for item in item_elements(&fragment) {
            let raw_text = text_without_sup(&item);
            let parsed = parse_sense_text(&raw_text);
            let links = extract_link_texts(&item);
            let fallback = clean_content_text(parsed.as_ref().map_or(&raw_text, |(_, rest)| rest));
            let fallback = split_on_spaced_dash(&fallback).unwrap_or(fallback);
            let idiom = if !links.is_empty() {
                links.join("; ")
            } else {
                fallback
            };

            if !idiom.is_empty() {
                idioms.push(idiom);
            }
        }
    }

    dedupe(idioms)
}

fn parse_synonyms(blocks: &[Block]) -> Vec<SynonymGroup> {
    let mut order = Vec::new();
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();

    for label in [BlockLabel::Synonyms, BlockLabel::RelatedSynonyms] {
        for fragment in fragments_for_label(blocks, label) {
            for item in item_elements(&fragment) {
                let raw_text = text_without_sup(&item);
                let Some((sense_label, rest)) = parse_sense_text(&raw_text) else {
                    continue;
                };
                let links = extract_link_texts(&item);
                let items = if !links.is_empty() {
                    links
                } else {
                    split_list_items(&rest)
                };
                if items.is_empty() {
                    continue;
                }

                for expanded in expand_sense_label(&sense_label) {
                    if !order.contains(&expanded) {
                        order.push(expanded.clone());
                    }
                    groups.entry(expanded).or_default().extend(items.clone());
                }
            }
        }
    }

    order
        .into_iter()
        .filter_map(|sense| {
            let items = dedupe(groups.remove(&sense).unwrap_or_default());
            (!items.is_empty()).then_some(SynonymGroup {
                sense: Some(sense),
                categories: Vec::new(),
                items,
            })
        })
        .collect()
}

fn block_items(blocks: &[Block], label: BlockLabel) -> Vec<String> {
    fragments_for_label(blocks, label)
        .into_iter()
        .flat_map(|fragment| list_item_texts(&fragment))
        .collect()
}

fn fragments_for_label(blocks: &[Block], label: BlockLabel) -> Vec<Html> {
    blocks
        .iter()
        .filter(|block| block.label == label)
        .map(|block| Html::parse_fragment(&block.html))
        .collect()
}

fn item_elements(fragment: &Html) -> Vec<ElementRef<'_>> {
    let dd_selector = Selector::parse("dd").expect("valid selector");
    let li_selector = Selector::parse("li").expect("valid selector");

    let dd_items = fragment.select(&dd_selector).collect::<Vec<_>>();
    if !dd_items.is_empty() {
        return dd_items;
    }

    fragment.select(&li_selector).collect()
}

fn list_item_texts(fragment: &Html) -> Vec<String> {
    item_elements(fragment)
        .into_iter()
        .map(|item| clean_text(&text_without_sup(&item)))
        .filter(|text| !text.is_empty())
        .collect()
}

fn plain_texts(fragment: &Html) -> Vec<String> {
    fragment
        .root_element()
        .children()
        .filter_map(ElementRef::wrap)
        .map(|element| clean_text(&text_without_sup(&element)))
        .filter(|text| !text.is_empty())
        .collect()
}

fn parse_sense_text(text: &str) -> Option<(String, String)> {
    let trimmed = clean_text(text);
    let rest = trimmed.strip_prefix('[')?;
    let end = rest.find(']')?;
    let label = rest[..end].trim();
    let content = clean_content_text(rest[end + 1..].trim());
    (!label.is_empty() && !content.is_empty()).then(|| (label.to_owned(), content))
}

fn expand_sense_label(label: &str) -> Vec<String> {
    let mut labels = Vec::new();

    for part in label
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        if let Some((start, end)) = split_range(part) {
            if start <= end {
                for number in start..=end {
                    labels.push(number.to_string());
                }
            }
        } else {
            labels.push(part.to_owned());
        }
    }

    labels
}

fn split_range(part: &str) -> Option<(usize, usize)> {
    let separator = part.find('–').or_else(|| part.find('-'))?;
    let start = part[..separator].trim().parse().ok()?;
    let end = part[separator + '–'.len_utf8()..].trim().parse().ok()?;
    Some((start, end))
}

fn split_list_items(text: &str) -> Vec<String> {
    text.split(',')
        .map(clean_content_text)
        .filter(|item| !item.is_empty())
        .collect()
}

fn split_on_spaced_dash(text: &str) -> Option<String> {
    text.split_once(" – ")
        .map(|(head, _)| head.to_owned())
        .or_else(|| text.split_once(" - ").map(|(head, _)| head.to_owned()))
}

fn extract_link_texts(item: &ElementRef<'_>) -> Vec<String> {
    let selector = Selector::parse("a").expect("valid selector");
    let mut items = Vec::new();

    for link in item.select(&selector) {
        if link
            .ancestors()
            .filter_map(ElementRef::wrap)
            .any(|ancestor| matches!(ancestor.value().name(), "i" | "em" | "sup"))
        {
            continue;
        }

        let href = link.value().attr("href").unwrap_or_default();
        let text = clean_content_text(&link.text().collect::<Vec<_>>().join(" "));
        if href.starts_with('#') || text.is_empty() || text.ends_with(':') {
            continue;
        }
        items.push(text);
    }

    dedupe(items)
}

fn extract_page_title(document: &Html) -> Option<String> {
    let selector = Selector::parse("title").expect("valid selector");
    document
        .select(&selector)
        .next()
        .map(|node| clean_text(&node.text().collect::<Vec<_>>().join(" ")))
        .filter(|text| !text.is_empty())
}

fn text_without_sup(element: &ElementRef<'_>) -> String {
    clean_text(&element.text().collect::<Vec<_>>().join(" "))
}

fn clean_content_text(text: &str) -> String {
    strip_footnote_refs(&clean_text(text))
}

fn clean_origin_text(text: &str) -> String {
    let mut value = clean_content_text(text);
    value = remove_arrow_codes(&value);
    value = value.replace("‚ ", "‚");
    value = value.replace(" ‘", "‘");
    value = value.replace("» ", "»");
    value = value.replace(" «", "«");
    value = value.replace(" ☆", "");
    clean_text(&value)
}

fn remove_arrow_codes(text: &str) -> String {
    let mut words = text.split_whitespace().peekable();
    let mut kept = Vec::new();

    while let Some(word) = words.next() {
        if word == "→" {
            if let Some(next) = words.peek() {
                if next.chars().all(|ch| ch.is_alphanumeric() || ch == '-') {
                    words.next();
                    continue;
                }
            }
        }
        kept.push(word);
    }

    kept.join(" ")
}

fn strip_footnote_refs(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' {
            let mut content = String::new();
            let mut is_ref = false;
            while let Some(next) = chars.peek().copied() {
                chars.next();
                if next == ']' {
                    is_ref = !content.is_empty()
                        && content
                            .chars()
                            .all(|inner| inner.is_ascii_digit() || inner.is_whitespace());
                    break;
                }
                content.push(next);
            }
            if is_ref {
                continue;
            }
            result.push('[');
            result.push_str(&content);
            if !content.ends_with(']') {
                result.push(']');
            }
        } else {
            result.push(ch);
        }
    }

    clean_text(&result)
}

fn clean_text(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" ,", ",")
        .replace(" .", ".")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" ;", ";")
        .replace(" :", ":")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    struct SnapshotCase {
        word: &'static str,
        fixture: &'static str,
        expected: &'static str,
    }

    #[test]
    fn matches_expected_snapshots_for_local_fixtures() {
        let cases = [
            SnapshotCase {
                word: "Bank",
                fixture: include_str!("../../tests/fixtures/wiktionary/Bank.html"),
                expected: include_str!("../../tests/snapshots/wiktionary/Bank.snap"),
            },
            SnapshotCase {
                word: "Haus",
                fixture: include_str!("../../tests/fixtures/wiktionary/Haus.html"),
                expected: include_str!("../../tests/snapshots/wiktionary/Haus.snap"),
            },
            SnapshotCase {
                word: "springen",
                fixture: include_str!("../../tests/fixtures/wiktionary/springen.html"),
                expected: include_str!("../../tests/snapshots/wiktionary/springen.snap"),
            },
            SnapshotCase {
                word: "Wolke",
                fixture: include_str!("../../tests/fixtures/wiktionary/Wolke.html"),
                expected: include_str!("../../tests/snapshots/wiktionary/Wolke.snap"),
            },
        ];

        for case in cases {
            let response = parse(
                case.word,
                &format!("https://de.wiktionary.org/wiki/{}", case.word),
                case.fixture,
            )
            .expect("fixture parses");

            assert_eq!(
                render_snapshot(&response),
                case.expected.trim_end(),
                "snapshot mismatch for {}",
                case.word
            );
        }
    }

    #[test]
    fn parses_missing_page_as_not_found() {
        let response = not_found_result(
            "Nixdaexistiert",
            "https://de.wiktionary.org/wiki/Nixdaexistiert",
        );

        assert_eq!(
            render_snapshot(&response),
            include_str!("../../tests/snapshots/wiktionary/Nixdaexistiert.snap").trim_end()
        );
    }

    #[test]
    fn parses_list_based_blocks_without_dl_wrappers() {
        let html = r#"
            <section>
              <h3>Substantiv</h3>
              <p style="font-weight:bold">Bedeutungen:</p>
              <ol><li>[1] erste Bedeutung</li><li>[2] zweite Bedeutung</li></ol>
              <p style="font-weight:bold">Beispiele:</p>
              <ul><li>[1] erstes Beispiel</li><li>[2] zweites Beispiel</li></ul>
              <p style="font-weight:bold">Synonyme:</p>
              <ul><li>[1] <a href="/wiki/Eins">Eins</a>, <a href="/wiki/Erstens">Erstens</a></li></ul>
              <p style="font-weight:bold">Redewendungen:</p>
              <ul><li>[1] <a href="/wiki/etwas_tun">etwas tun</a>/<a href="/wiki/anders_tun">anders tun</a></li></ul>
              <p style="font-weight:bold">Herkunft:</p>
              <ul><li>aus dem Testbestand</li></ul>
            </section>
        "#;

        let fragment = Html::parse_fragment(html);
        let section_selector = Selector::parse("section").expect("valid selector");
        let section = fragment
            .select(&section_selector)
            .next()
            .expect("section exists");
        let blocks = collect_labeled_blocks(&section);

        assert_eq!(
            parse_origin(&blocks),
            Some("aus dem Testbestand".to_owned())
        );
        assert_eq!(
            parse_idioms(&blocks),
            vec!["etwas tun; anders tun".to_owned()]
        );
        let synonyms = parse_synonyms(&blocks);
        assert_eq!(synonyms.len(), 1);
        assert_eq!(synonyms[0].sense.as_deref(), Some("1"));
        assert_eq!(
            synonyms[0].items,
            vec!["Eins".to_owned(), "Erstens".to_owned()]
        );

        let senses = parse_senses(&blocks);
        assert_eq!(senses.len(), 2);
        assert_eq!(senses[0].label.as_deref(), Some("1"));
        assert_eq!(senses[0].definition.as_deref(), Some("erste Bedeutung"));
        assert_eq!(senses[0].examples, vec!["erstes Beispiel".to_owned()]);
        assert_eq!(senses[1].label.as_deref(), Some("2"));
        assert_eq!(senses[1].definition.as_deref(), Some("zweite Bedeutung"));
        assert_eq!(senses[1].examples, vec!["zweites Beispiel".to_owned()]);
    }

    fn render_snapshot(response: &SourceResult) -> String {
        let mut lines = vec![
            format!("source={:?}", response.source),
            format!("ok={}", response.ok),
            format!(
                "url={}",
                match response.url.as_ref() {
                    Some(UrlValue::One(url)) => url.as_str(),
                    Some(UrlValue::Many(urls)) => panic!("unexpected multi-url snapshot: {urls:?}"),
                    None => "-",
                }
            ),
        ];

        if let Some(error) = &response.error {
            lines.push(format!("error={error}"));
        }

        for entry in &response.entries {
            lines.push(format!(
                "entry {} headword={} title={} part_of_speech={} grammar={}",
                entry.id,
                entry.headword,
                entry.title.as_deref().unwrap_or("-"),
                entry.part_of_speech.as_deref().unwrap_or("-"),
                entry.grammar.as_deref().unwrap_or("-"),
            ));

            if let Some(etymology) = &entry.etymology {
                lines.push(format!("etymology={etymology}"));
            }
            if !entry.idioms.is_empty() {
                lines.push(format!("idioms=[{}]", entry.idioms.join(" | ")));
            }
            for group in &entry.synonym_groups {
                lines.push(format!(
                    "synonyms sense={} items=[{}]",
                    group.sense.as_deref().unwrap_or("-"),
                    group.items.join(", ")
                ));
            }
            for sense in &entry.senses {
                lines.push(format!(
                    "sense {} label={} definition={}",
                    sense.id,
                    sense.label.as_deref().unwrap_or("-"),
                    sense.definition.as_deref().unwrap_or("-"),
                ));
                if !sense.examples.is_empty() {
                    lines.push(format!(
                        "examples {}=[{}]",
                        sense.id,
                        sense.examples.join(" | ")
                    ));
                }
            }
        }

        lines.join("\n")
    }
}
