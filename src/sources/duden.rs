use crate::http::fetch_html;
use crate::models::{
    dedupe, normalize_text, DictionaryEntry, Sense, Source, SourceResult, SynonymGroup, UrlValue,
};
use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};

pub async fn lookup(client: &Client, query: &str) -> Result<SourceResult> {
    let encoded = urlencoding::encode(query);
    let url = format!("https://www.duden.de/rechtschreibung/{encoded}?amp");
    let html = fetch_html(client, &url).await?;
    let entries = parse(query, &url, &html);
    Ok(SourceResult::ok(
        Source::Duden,
        Some(UrlValue::One(url)),
        entries,
    ))
}

pub fn parse(query: &str, url: &str, html: &str) -> Vec<DictionaryEntry> {
    let document = Html::parse_document(html);
    let mut entry = DictionaryEntry::new(1, query);
    entry.url = Some(url.to_owned());
    entry.title = first_text(&document, &["h1", ".lemma__main"]);
    entry.part_of_speech = first_text(&document, &[".tuple__wortart", ".lemma__word-class"]);
    entry.grammar = first_text(&document, &[".tuple__grammatik", ".lemma__grammar"]);
    entry.etymology = first_text(
        &document,
        &["section[id*=Herkunft]", ".tuple__herkunft", ".word-origin"],
    );
    entry.idioms = collect_text(
        &document,
        &["section[id*=Redewendungen] li", ".phrase", ".idiom"],
    );

    let synonyms = collect_text(&document, &["a[href*='/synonyme/']", ".synonyms a"]);
    if !synonyms.is_empty() {
        entry.synonym_groups.push(SynonymGroup::items(synonyms));
    }

    let definitions = collect_text(
        &document,
        &[
            ".tuple__valenz",
            ".enumeration__text",
            ".division__definition",
            "section[id*=Bedeutung] li",
        ],
    );
    let examples = collect_text(
        &document,
        &[".example", ".tuple__example", "section[id*=Beispiele] li"],
    );

    for (idx, text) in definitions.into_iter().enumerate() {
        let mut sense = Sense::simple(idx + 1, text);
        if idx == 0 {
            sense.examples = examples.clone();
            sense.idioms = entry.idioms.clone();
        }
        entry.senses.push(sense);
    }

    if entry.senses.is_empty() && !examples.is_empty() {
        let mut sense = Sense::default();
        sense.id = 1;
        sense.examples = examples;
        entry.senses.push(sense);
    }

    if entry.is_empty() {
        Vec::new()
    } else {
        vec![entry]
    }
}

fn first_text(document: &Html, selectors: &[&str]) -> Option<String> {
    selectors
        .iter()
        .filter_map(|selector| Selector::parse(selector).ok())
        .find_map(|selector| {
            document.select(&selector).find_map(|node| {
                let text = normalize_text(&node.text().collect::<Vec<_>>().join(" "));
                (!text.is_empty()).then_some(text)
            })
        })
}

fn collect_text(document: &Html, selectors: &[&str]) -> Vec<String> {
    for selector in selectors.iter().filter_map(|s| Selector::parse(s).ok()) {
        let values = dedupe(
            document
                .select(&selector)
                .map(|node| normalize_text(&node.text().collect::<Vec<_>>().join(" "))),
        );
        if !values.is_empty() {
            return values;
        }
    }
    Vec::new()
}
