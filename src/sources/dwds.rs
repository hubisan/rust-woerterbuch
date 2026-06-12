use crate::http::fetch_html;
use crate::models::{DictionaryEntry, Sense, Source, SourceResult, UrlValue};
use anyhow::Result;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};

const DWDS_BASE_URL: &str = "https://www.dwds.de/wb/";
const DEFINITION_SKIP_CLASSES: &[&str] = &["dwdswb-binnenquelle", "dwdswb-paraphrase"];

pub async fn lookup(client: &Client, query: &str) -> Result<SourceResult> {
    let url = build_url(query);
    let html = fetch_html(client, &url).await?;
    parse(query, &url, &html)
}

pub fn parse(query: &str, page_url: &str, html: &str) -> Result<SourceResult> {
    let document = Html::parse_document(html);
    if !entry_page_p(&document) {
        return Ok(SourceResult::error(Source::Dwds, "No matches found"));
    }

    let canonical_url = canonical_url(&document, page_url);
    let entries = article_scopes(&document)
        .into_iter()
        .enumerate()
        .filter_map(|(index, scope)| parse_homograph(&scope, index + 1, &canonical_url, query))
        .collect::<Vec<_>>();

    if entries.is_empty() {
        Ok(SourceResult::error(Source::Dwds, "No matches found"))
    } else {
        Ok(SourceResult::ok(
            Source::Dwds,
            Some(UrlValue::One(canonical_url)),
            entries,
        ))
    }
}

fn build_url(lemma: &str) -> String {
    format!("{DWDS_BASE_URL}{}", urlencoding::encode(lemma))
}

fn parse_homograph(
    scope: &ElementRef<'_>,
    id: usize,
    canonical_url: &str,
    fallback_lemma: &str,
) -> Option<DictionaryEntry> {
    let article = find_first_descendant(scope, &|node| has_class(node, "dwdswb-artikel"))?;
    let bookmark = find_first_descendant(&article, &|node| has_class(node, "dwds-bookmark-button"));
    let heading = find_first_descendant(&article, &|node| has_class(node, "dwdswb-ft-lemmaansatz"));
    let lemma_node = heading.and_then(|node| node.select(&selector("b")).next());

    let headword = lemma_node
        .map(|node| text(&node))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback_lemma.to_owned());
    let title = heading
        .map(|node| text(&node))
        .filter(|value| !value.is_empty());
    let grammar = field_text(&article, "Grammatik");
    let part_of_speech = grammar.as_deref().and_then(wortart_from_grammar);
    let etymology = parse_etymology(scope);
    let idioms = parse_idioms(&article);
    let senses = find_first_descendant(scope, &|node| has_class(node, "dwdswb-lesarten"))
        .map(|root| parse_sense_list(&children_with_class(&root, "dwdswb-lesart")))
        .unwrap_or_default();

    let mut entry = DictionaryEntry::new(id, headword);
    entry.homograph = bookmark
        .and_then(|node| node.value().attr("data-hidx").map(str::to_owned))
        .or_else(|| scope.value().attr("id").map(str::to_owned))
        .filter(|value| !value.is_empty());
    entry.title = title;
    entry.part_of_speech = part_of_speech;
    entry.grammar = grammar;
    entry.etymology = etymology;
    entry.idioms = idioms;
    entry.url = Some(canonical_url.to_owned());
    entry.senses = senses;

    (!entry.is_empty()).then_some(entry)
}

fn parse_sense_list(nodes: &[ElementRef<'_>]) -> Vec<Sense> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| parse_sense(node, index + 1))
        .collect()
}

fn parse_sense(node: &ElementRef<'_>, id: usize) -> Sense {
    let label_node = direct_child_by_class(node, "dwdswb-lesart-n");
    let content_node = direct_child_by_class(node, "dwdswb-lesart-content");
    let def_node = content_node
        .as_ref()
        .and_then(|content| direct_child_by_class(content, "dwdswb-lesart-def"));
    let usage_node = content_node
        .as_ref()
        .and_then(|content| direct_child_by_class(content, "dwdswb-verwendungsbeispiele"));
    let child_nodes = content_node
        .as_ref()
        .map(|content| children_with_class(content, "dwdswb-lesart"))
        .unwrap_or_default();

    Sense {
        id,
        source_id: node.value().attr("id").map(str::to_owned),
        label: label_node
            .map(|node| text(&node))
            .filter(|value| !value.is_empty()),
        definition: def_node
            .as_ref()
            .and_then(|def| extract_definition_text(def, content_node.as_ref())),
        qualifiers: def_node
            .as_ref()
            .map(extract_qualifiers)
            .unwrap_or_default(),
        examples: usage_node
            .as_ref()
            .map(extract_examples)
            .unwrap_or_default(),
        idioms: Vec::new(),
        synonyms: Vec::new(),
        image_url: None,
        subsenses: parse_sense_list(&child_nodes),
    }
}

fn canonical_url(document: &Html, fallback: &str) -> String {
    document
        .select(&selector(r#"link[rel="canonical"]"#))
        .next()
        .and_then(|link| link.value().attr("href"))
        .map(str::to_owned)
        .unwrap_or_else(|| fallback.to_owned())
}

fn field_text(article: &ElementRef<'_>, label: &str) -> Option<String> {
    for block in descendants_with_class(article, "dwdswb-ft-block") {
        let block_label =
            find_first_descendant(&block, &|node| has_class(node, "dwdswb-ft-blocklabel"))
                .map(|node| text(&node))
                .unwrap_or_default();
        if !block_label.contains(label) {
            continue;
        }

        let block_text =
            find_first_descendant(&block, &|node| has_class(node, "dwdswb-ft-blocktext"))
                .map(|node| text(&node))
                .unwrap_or_default();
        if !block_text.is_empty() {
            return Some(block_text);
        }
    }
    None
}

fn wortart_from_grammar(grammar: &str) -> Option<String> {
    let head = grammar.split('·').next().unwrap_or_default().trim();
    let value = head.split('(').next().unwrap_or_default().trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn extract_qualifiers(def_node: &ElementRef<'_>) -> Vec<String> {
    let mut qualifiers = Vec::new();

    for child in element_children(def_node) {
        if has_class(&child, "dwdswb-diasystematik") {
            qualifiers.extend(collect_qualifiers(&child));
        }
    }

    dedupe(qualifiers)
}

fn collect_qualifiers(node: &ElementRef<'_>) -> Vec<String> {
    let mut out = Vec::new();
    if qualifier_node(node) {
        let value = text(node);
        if !value.is_empty() {
            out.push(value);
        }
    }

    for child in element_children(node) {
        out.extend(collect_qualifiers(&child));
    }

    out
}

fn qualifier_node(node: &ElementRef<'_>) -> bool {
    let classes = class_list(node);
    !classes.is_empty()
        && !classes.contains(&"dwdswb-diasystematik")
        && classes.iter().any(|class| class.starts_with("dwdswb-"))
}

fn extract_definition_text(
    def_node: &ElementRef<'_>,
    content_node: Option<&ElementRef<'_>>,
) -> Option<String> {
    let mut parts = Vec::new();
    let mut saw_relevant = false;

    for child in element_children(def_node) {
        if has_class(&child, "dwdswb-verweise") {
            saw_relevant = true;
            let value = extract_reference_text(&child);
            if !value.is_empty() {
                parts.push(value);
            }
            continue;
        }

        if has_any_class(
            &child,
            &[
                "dwdswb-syntagmatik",
                "dwdswb-definitionen",
                "dwdswb-definition",
            ],
        ) {
            saw_relevant = true;
            let value = text_skipping_classes(&child, DEFINITION_SKIP_CLASSES);
            if !value.is_empty() {
                parts.push(value);
            }
        }
    }

    let definition = parts.join(" ");
    let mwa_definition = content_node.and_then(extract_mwa_text);
    let qualifiers = extract_qualifiers(def_node);

    if let Some(ref mwa) = mwa_definition {
        if content_node.is_some_and(explicit_phraseme_block) {
            return Some(mwa.clone());
        }
        if !definition.is_empty()
            && definition.starts_with('⟨')
            && qualifiers.iter().any(|item| item == "übertragen")
        {
            return Some(mwa.clone());
        }
    }

    if !definition.is_empty() {
        return Some(definition);
    }
    if let Some(mwa) = mwa_definition {
        return Some(mwa);
    }
    if saw_relevant || !qualifiers.is_empty() {
        return Some(String::new());
    }

    None
}

fn extract_reference_text(node: &ElementRef<'_>) -> String {
    let ref_node = if has_class(node, "dwdswb-verweis") {
        Some(*node)
    } else {
        find_first_descendant(node, &|child| has_class(child, "dwdswb-verweis"))
    };
    let headline = text(node);
    let definition = ref_node.and_then(reference_definition);

    match (headline.is_empty(), definition.unwrap_or_default()) {
        (false, def) if !def.is_empty() => format!("{headline} = {def}"),
        (false, _) => headline,
        (true, def) => def,
    }
}

fn reference_definition(node: ElementRef<'_>) -> Option<String> {
    let raw = node.value().attr("data-content")?;
    let stripped = strip_html_tags(raw);
    let value = clean_text(&stripped);
    (!value.is_empty()).then_some(value)
}

fn extract_mwa_text(content_node: &ElementRef<'_>) -> Option<String> {
    let local_scope = find_local_mwa_scope(content_node)?;
    if !mwa_marker(&local_scope) && !local_mwa_marker(content_node) {
        return None;
    }

    let phrase_scope = if has_class(&local_scope, "dwdswb-phrasem")
        || has_class(&local_scope, "dwdswb-konstruktionsmuster")
    {
        Some(local_scope)
    } else {
        find_first_descendant(&local_scope, &|child| {
            has_any_class(child, &["dwdswb-phrasem", "dwdswb-konstruktionsmuster"])
        })
    }?;

    let phrase_node =
        find_first_descendant(&phrase_scope, &|child| has_class(child, "dwdswb-belegtext"))?;
    let phrase = text_skipping_classes(&phrase_node, &["dwdswb-paraphrase"]);
    if phrase.is_empty() {
        return None;
    }

    let paraphrases = descendants_with_class(&phrase_scope, "dwdswb-paraphrase")
        .into_iter()
        .filter_map(|node| {
            let value = normalize_paraphrase_text(&text(&node));
            (!value.is_empty()).then_some(value)
        })
        .collect::<Vec<_>>();

    if paraphrases.is_empty() {
        Some(format!("{phrase} (MWA)"))
    } else {
        Some(format!("{phrase} (MWA) = {}", paraphrases.join("; ")))
    }
}

fn normalize_paraphrase_text(text: &str) -> String {
    clean_text(text)
        .trim_start_matches("(=")
        .trim_end_matches(')')
        .trim()
        .to_owned()
}

fn find_local_mwa_scope<'a>(content_node: &'a ElementRef<'a>) -> Option<ElementRef<'a>> {
    for child in element_children(content_node) {
        if has_class(&child, "dwdswb-lesart") {
            continue;
        }

        if has_any_class(
            &child,
            &[
                "dwdswb-phraseme",
                "dwdswb-syntagmatik",
                "dwdswb-konstruktionsmuster",
            ],
        ) {
            return Some(child);
        }

        if let Some(found) = find_first_descendant(&child, &|node| {
            has_any_class(node, &["dwdswb-phrasem", "dwdswb-konstruktionsmuster"])
        }) {
            let _ = found;
            return Some(child);
        }
    }

    None
}

fn explicit_phraseme_block(content_node: &ElementRef<'_>) -> bool {
    element_children(content_node)
        .iter()
        .any(|child| has_class(child, "dwdswb-phraseme"))
}

fn local_mwa_marker(content_node: &ElementRef<'_>) -> bool {
    for child in element_children(content_node) {
        if has_class(&child, "dwdswb-lesart") {
            continue;
        }
        if mwa_marker(&child) {
            return true;
        }
    }
    false
}

fn mwa_marker(node: &ElementRef<'_>) -> bool {
    find_first_descendant(node, &|child| {
        child
            .value()
            .attr("src")
            .is_some_and(|src| src.contains("letter-mwa.svg"))
            || child
                .value()
                .attr("title")
                .or_else(|| child.value().attr("data-original-title"))
                .is_some_and(|title| title.contains("Mehrwortausdruck"))
    })
    .is_some()
}

fn extract_examples(usage_node: &ElementRef<'_>) -> Vec<String> {
    descendants_with_class(usage_node, "dwdswb-belegtext")
        .into_iter()
        .map(|node| text(&node))
        .filter(|value| !value.is_empty())
        .collect()
}

fn parse_idioms(article: &ElementRef<'_>) -> Vec<String> {
    let Some(block) = find_first_descendant(article, &|node| {
        node.value()
            .attr("id")
            .is_some_and(|id| id.starts_with("relation-block-") && id.ends_with("-mwa"))
    }) else {
        return Vec::new();
    };

    let mut idioms = Vec::new();
    for link in block.select(&selector("a")) {
        let href = link.value().attr("href").unwrap_or_default();
        let value = text(&link);
        if href.starts_with("/wb/") && !value.is_empty() && !idioms.contains(&value) {
            idioms.push(value);
        }
    }
    idioms
}

fn parse_etymology(scope: &ElementRef<'_>) -> Option<String> {
    let entry = find_first_descendant(scope, &|node| has_class(node, "etymwb-entry"))?;
    let value = text(&entry);
    (!value.is_empty()).then_some(value)
}

fn article_scopes<'a>(document: &'a Html) -> Vec<ElementRef<'a>> {
    let panes = document
        .select(&selector(".tab-pane"))
        .filter(article_scope)
        .collect::<Vec<_>>();

    if panes.is_empty() && entry_page_p(document) {
        vec![document.root_element()]
    } else {
        panes
    }
}

fn article_scope(node: &ElementRef<'_>) -> bool {
    node.value().attr("id") != Some("0")
        && find_first_descendant(node, &|child| has_class(child, "dwdswb-artikel")).is_some()
}

fn entry_page_p(document: &Html) -> bool {
    find_first_descendant(&document.root_element(), &|node| {
        has_class(node, "dwdswb-artikel")
    })
    .is_some()
}

fn text(node: &ElementRef<'_>) -> String {
    clean_text(&node.text().collect::<Vec<_>>().join(" "))
}

fn text_skipping_classes(node: &ElementRef<'_>, classes: &[&str]) -> String {
    let mut out = String::new();
    append_text_skipping(node, classes, &mut out);
    clean_text(&out)
}

fn append_text_skipping(node: &ElementRef<'_>, classes: &[&str], out: &mut String) {
    if has_any_class(node, classes) {
        return;
    }

    for child in node.children() {
        match child.value() {
            scraper::node::Node::Text(text) => {
                out.push_str(text.text.as_ref());
                out.push(' ');
            }
            scraper::node::Node::Element(_) => {
                if let Some(element) = ElementRef::wrap(child) {
                    append_text_skipping(&element, classes, out);
                }
            }
            _ => {}
        }
    }
}

fn clean_text(text: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;

    for ch in text.chars() {
        let ch = if ch == '\u{00a0}' { ' ' } else { ch };
        if ch.is_whitespace() {
            pending_space = !out.is_empty();
            continue;
        }

        if pending_space
            && !matches!(ch, ',' | '.' | ')' | '⟩')
            && !out.ends_with('(')
            && !out.ends_with('⟨')
        {
            out.push(' ');
        }

        out.push(ch);
        pending_space = false;
    }

    out.trim().to_owned()
}

fn strip_html_tags(text: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for ch in text.chars() {
        match ch {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(ch),
            _ => {}
        }
    }
    out
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in values {
        if !value.is_empty() && !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

fn selector(input: &str) -> Selector {
    Selector::parse(input).expect("valid selector")
}

fn class_list<'a>(node: &'a ElementRef<'a>) -> Vec<&'a str> {
    node.value()
        .attr("class")
        .unwrap_or_default()
        .split_whitespace()
        .collect()
}

fn has_class(node: &ElementRef<'_>, class: &str) -> bool {
    class_list(node).contains(&class)
}

fn has_any_class(node: &ElementRef<'_>, classes: &[&str]) -> bool {
    classes.iter().any(|class| has_class(node, class))
}

fn element_children<'a>(node: &'a ElementRef<'a>) -> Vec<ElementRef<'a>> {
    node.children().filter_map(ElementRef::wrap).collect()
}

fn children_with_class<'a>(node: &'a ElementRef<'a>, class: &str) -> Vec<ElementRef<'a>> {
    element_children(node)
        .into_iter()
        .filter(|child| has_class(child, class))
        .collect()
}

fn direct_child_by_class<'a>(node: &'a ElementRef<'a>, class: &str) -> Option<ElementRef<'a>> {
    children_with_class(node, class).into_iter().next()
}

fn descendants_with_class<'a>(node: &'a ElementRef<'a>, class: &str) -> Vec<ElementRef<'a>> {
    node.descendants()
        .filter_map(ElementRef::wrap)
        .filter(|child| has_class(child, class))
        .collect()
}

fn find_first_descendant<'a>(
    node: &'a ElementRef<'a>,
    predicate: &dyn Fn(&ElementRef<'_>) -> bool,
) -> Option<ElementRef<'a>> {
    node.descendants()
        .filter_map(ElementRef::wrap)
        .skip(1)
        .find(|child| predicate(child))
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
    fn matches_expected_json_for_local_fixtures() {
        let cases = [
            SnapshotCase {
                word: "Bank",
                fixture: include_str!("../../tests/fixtures/dwds/Bank/page.html"),
                expected: include_str!("../../tests/expected/dwds/Bank.json"),
            },
            SnapshotCase {
                word: "Haus",
                fixture: include_str!("../../tests/fixtures/dwds/Haus/page.html"),
                expected: include_str!("../../tests/expected/dwds/Haus.json"),
            },
            SnapshotCase {
                word: "springen",
                fixture: include_str!("../../tests/fixtures/dwds/springen/page.html"),
                expected: include_str!("../../tests/expected/dwds/springen.json"),
            },
            SnapshotCase {
                word: "verlieben",
                fixture: include_str!("../../tests/fixtures/dwds/verlieben/page.html"),
                expected: include_str!("../../tests/expected/dwds/verlieben.json"),
            },
            SnapshotCase {
                word: "Wolke",
                fixture: include_str!("../../tests/fixtures/dwds/Wolke/page.html"),
                expected: include_str!("../../tests/expected/dwds/Wolke.json"),
            },
            SnapshotCase {
                word: "Zaun",
                fixture: include_str!("../../tests/fixtures/dwds/Zaun/page.html"),
                expected: include_str!("../../tests/expected/dwds/Zaun.json"),
            },
            SnapshotCase {
                word: "Nixdaexistiert",
                fixture: include_str!("../../tests/fixtures/dwds/Nixdaexistiert/page.html"),
                expected: include_str!("../../tests/expected/dwds/Nixdaexistiert.json"),
            },
        ];

        for case in cases {
            let response = parse(
                case.word,
                &format!("https://www.dwds.de/wb/{}", case.word),
                case.fixture,
            )
            .expect("fixture parses");
            assert_eq!(
                serde_json::to_string_pretty(&response).expect("response serializes"),
                case.expected.trim_end(),
                "expected JSON mismatch for {}",
                case.word
            );
        }
    }

    #[test]
    fn parses_bank_into_two_homographs_with_stable_hidx_values() {
        let response = parse(
            "Bank",
            "https://www.dwds.de/wb/Bank",
            include_str!("../../tests/fixtures/dwds/Bank/page.html"),
        )
        .expect("fixture parses");

        assert!(response.ok);
        assert_eq!(response.entries.len(), 2);
        assert_eq!(response.entries[0].homograph.as_deref(), Some("1"));
        assert_eq!(response.entries[1].homograph.as_deref(), Some("2"));
        assert_eq!(response.entries[0].headword, "Bank");
        assert_eq!(response.entries[1].headword, "Bank");
    }

    #[test]
    fn no_match_html_without_article_returns_error_result() {
        let response = parse(
            "Nixdaexistiert",
            "https://www.dwds.de/wb/Nixdaexistiert",
            include_str!("../../tests/fixtures/dwds/Nixdaexistiert/page.html"),
        )
        .expect("fixture parses");

        assert!(!response.ok);
        assert_eq!(response.error.as_deref(), Some("No matches found"));
        assert!(response.entries.is_empty());
    }
}
