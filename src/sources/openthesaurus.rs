use crate::http::fetch_html;
use crate::models::{
    dedupe, normalize_text, DictionaryEntry, Source, SourceResult, SynonymGroup, UrlValue,
};
use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};

pub async fn lookup(client: &Client, query: &str) -> Result<SourceResult> {
    let encoded = urlencoding::encode(query);
    let url = format!("https://www.openthesaurus.de/synonyme/{encoded}");
    let html = fetch_html(client, &url).await?;
    let entries = parse(query, &url, &html);
    Ok(SourceResult::ok(
        Source::Openthesaurus,
        Some(UrlValue::One(url)),
        entries,
    ))
}

pub fn parse(query: &str, url: &str, html: &str) -> Vec<DictionaryEntry> {
    let document = Html::parse_document(html);
    let mut entry = DictionaryEntry::new(1, query);
    entry.url = Some(url.to_owned());

    // OpenThesaurus primarily contributes synonym groups. Replace these tolerant
    // selectors with source-specific logic ported from woerterbuch-openthesaurus.el.
    let selectors = [
        "a.synset-term",
        ".synset a",
        "ul.synonyms li a",
        "#content a[href^='/synonyme/']",
    ];

    let mut synonyms = Vec::new();
    for selector in selectors.iter().filter_map(|s| Selector::parse(s).ok()) {
        synonyms.extend(
            document
                .select(&selector)
                .map(|node| normalize_text(&node.text().collect::<Vec<_>>().join(" "))),
        );
        synonyms = dedupe(synonyms);
        if !synonyms.is_empty() {
            break;
        }
    }

    if !synonyms.is_empty() {
        entry.synonym_groups.push(SynonymGroup::items(synonyms));
    }

    if entry.is_empty() {
        Vec::new()
    } else {
        vec![entry]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_synonyms() {
        let html = r#"<ul class="synonyms"><li><a>Auto</a></li><li><a>Wagen</a></li></ul>"#;
        let entries = parse("PKW", "https://www.openthesaurus.de/synonyme/PKW", html);
        assert_eq!(entries[0].synonym_groups[0].items, vec!["Auto", "Wagen"]);
    }
}
