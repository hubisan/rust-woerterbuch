use crate::http::fetch_html;
use crate::models::{
    dedupe, normalize_text, DictionaryEntry, Sense, Source, SourceResult, SynonymGroup, UrlValue,
};
use anyhow::Result;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};

pub async fn lookup(client: &Client, query: &str) -> Result<SourceResult> {
    let encoded = urlencoding::encode(query);
    let api_url = format!("https://de.wiktionary.org/api/rest_v1/page/html/{encoded}");
    let page_url = format!("https://de.wiktionary.org/wiki/{encoded}");
    let html = fetch_html(client, &api_url).await?;
    let entries = parse(query, &page_url, &html);
    Ok(SourceResult::ok(
        Source::Wiktionary,
        Some(UrlValue::One(page_url)),
        entries,
    ))
}

pub fn parse(query: &str, page_url: &str, html: &str) -> Vec<DictionaryEntry> {
    let document = Html::parse_document(html);
    let heading_selector = Selector::parse("h2, h3, h4").expect("valid selector");
    let list_selector = Selector::parse("ol li, ul li").expect("valid selector");
    let paragraph_selector = Selector::parse("p").expect("valid selector");

    let mut entries = Vec::new();
    let mut current: Option<DictionaryEntry> = None;

    for heading in document.select(&heading_selector) {
        let heading_text = normalize_text(&heading.text().collect::<Vec<_>>().join(" "));
        if looks_like_word_section(&heading_text) {
            if let Some(entry) = current.take().filter(|entry| !entry.is_empty()) {
                entries.push(entry);
            }

            let mut entry = DictionaryEntry::new(entries.len() + 1, query);
            entry.title = Some(heading_text.clone());
            entry.part_of_speech = extract_word_class(&heading_text);
            entry.grammar = Some(heading_text);
            entry.url = Some(page_url.to_owned());
            current = Some(entry);
            continue;
        }

        let Some(entry) = current.as_mut() else {
            continue;
        };

        let lower = heading_text.to_lowercase();
        let following_html = collect_following_fragment(&heading);
        let fragment = Html::parse_fragment(&following_html);

        if lower.contains("bedeutung") {
            let items = extract_items(&fragment, &list_selector);
            for (idx, item) in items.into_iter().enumerate() {
                let sense_id = entry.senses.len() + idx + 1;
                let mut sense = Sense::simple(sense_id, item);
                sense.label = Some(sense_id.to_string());
                entry.senses.push(sense);
            }
        } else if lower.contains("beispiel") {
            let examples = extract_items(&fragment, &list_selector);
            if let Some(sense) = entry.senses.first_mut() {
                sense.examples.extend(examples);
                sense.examples = dedupe(std::mem::take(&mut sense.examples));
            }
        } else if lower.contains("synonym") || lower.contains("sinnverwand") {
            let items = extract_items(&fragment, &list_selector);
            if !items.is_empty() {
                entry.synonym_groups.push(SynonymGroup::items(items));
            }
        } else if lower.contains("herkunft") || lower.contains("etymologie") {
            entry.etymology = extract_paragraphs(&fragment, &paragraph_selector)
                .into_iter()
                .next();
        } else if lower.contains("redewendung") || lower.contains("sprichw") {
            entry
                .idioms
                .extend(extract_items(&fragment, &list_selector));
            entry.idioms = dedupe(std::mem::take(&mut entry.idioms));
        }
    }

    if let Some(entry) = current.take().filter(|entry| !entry.is_empty()) {
        entries.push(entry);
    }

    entries
}

fn looks_like_word_section(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("substantiv")
        || lower.contains("verb")
        || lower.contains("adjektiv")
        || lower.contains("adverb")
        || lower.contains("pronomen")
        || lower.contains("präposition")
        || lower.contains("konjunktion")
}

fn extract_word_class(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    for candidate in [
        "Substantiv",
        "Verb",
        "Adjektiv",
        "Adverb",
        "Pronomen",
        "Präposition",
        "Konjunktion",
    ] {
        if lower.contains(&candidate.to_lowercase()) {
            return Some(candidate.to_owned());
        }
    }
    None
}

fn collect_following_fragment(heading: &ElementRef<'_>) -> String {
    let mut html = String::new();
    let heading_level = heading.value().name();

    for sibling in heading.next_siblings() {
        if let Some(element) = ElementRef::wrap(sibling) {
            let name = element.value().name();
            if matches!(name, "h2" | "h3" | "h4") && name <= heading_level {
                break;
            }
            html.push_str(&element.html());
        }
    }

    html
}

fn extract_items(fragment: &Html, selector: &Selector) -> Vec<String> {
    dedupe(
        fragment
            .select(selector)
            .map(|node| normalize_text(&node.text().collect::<Vec<_>>().join(" "))),
    )
}

fn extract_paragraphs(fragment: &Html, selector: &Selector) -> Vec<String> {
    dedupe(
        fragment
            .select(selector)
            .map(|node| normalize_text(&node.text().collect::<Vec<_>>().join(" "))),
    )
}
