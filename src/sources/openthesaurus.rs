use crate::http::fetch_html;
use crate::models::{DictionaryEntry, Source, SourceResult, SynonymGroup, UrlValue};
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

pub async fn lookup(client: &Client, query: &str) -> Result<SourceResult> {
    let encoded = urlencoding::encode(query);
    let api_url =
        format!("https://www.openthesaurus.de/synonyme/search?format=application/json&q={encoded}");
    let page_url = format!("https://www.openthesaurus.de/synonyme/{encoded}");
    let body = fetch_html(client, &api_url).await?;
    parse(query, &page_url, &body)
}

pub fn parse(query: &str, url: &str, body: &str) -> Result<SourceResult> {
    let payload: OpenThesaurusResponse = serde_json::from_str(body)?;
    let mut entry = DictionaryEntry::new(1, query);
    entry.url = Some(url.to_owned());

    for synset in payload.synsets {
        entry.synonym_groups.push(SynonymGroup {
            sense: None,
            categories: synset.categories,
            items: extract_synonyms(synset.terms, query),
        });
    }

    if entry.synonym_groups.is_empty() {
        Ok(SourceResult {
            source: Source::Openthesaurus,
            ok: false,
            url: Some(UrlValue::One(url.to_owned())),
            entries: Vec::new(),
            error: Some("No matches found".to_owned()),
        })
    } else {
        Ok(SourceResult::ok(
            Source::Openthesaurus,
            Some(UrlValue::One(url.to_owned())),
            vec![entry],
        ))
    }
}

fn extract_synonyms(terms: Vec<OpenThesaurusTerm>, query: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let query = query.to_lowercase();
    let mut synonyms = Vec::new();

    for term in terms {
        let candidate = term.term.trim();
        if candidate.is_empty() || candidate.to_lowercase() == query {
            continue;
        }
        if seen.insert(candidate.to_owned()) {
            synonyms.push(candidate.to_owned());
        }
    }

    synonyms
}

#[derive(Debug, Deserialize)]
struct OpenThesaurusResponse {
    #[serde(default)]
    synsets: Vec<OpenThesaurusSynset>,
}

#[derive(Debug, Deserialize)]
struct OpenThesaurusSynset {
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    terms: Vec<OpenThesaurusTerm>,
}

#[derive(Debug, Deserialize)]
struct OpenThesaurusTerm {
    term: String,
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
                fixture: include_str!(
                    "../../emacs-lisp/tests/files/openthesaurus/Bank/openthesaurus-Bank.json"
                ),
                expected: include_str!(
                    "../../tests/snapshots/openthesaurus/Bank.snap"
                ),
            },
            SnapshotCase {
                word: "Haus",
                fixture: include_str!(
                    "../../emacs-lisp/tests/files/openthesaurus/Haus/openthesaurus-Haus.json"
                ),
                expected: include_str!(
                    "../../tests/snapshots/openthesaurus/Haus.snap"
                ),
            },
            SnapshotCase {
                word: "Nixdaexistiert",
                fixture: include_str!(
                    "../../emacs-lisp/tests/files/openthesaurus/Nixdaexistiert/openthesaurus-Nixdaexistiert.json"
                ),
                expected: include_str!(
                    "../../tests/snapshots/openthesaurus/Nixdaexistiert.snap"
                ),
            },
            SnapshotCase {
                word: "Wolke",
                fixture: include_str!(
                    "../../emacs-lisp/tests/files/openthesaurus/Wolke/openthesaurus-Wolke.json"
                ),
                expected: include_str!(
                    "../../tests/snapshots/openthesaurus/Wolke.snap"
                ),
            },
            SnapshotCase {
                word: "Zaun",
                fixture: include_str!(
                    "../../emacs-lisp/tests/files/openthesaurus/Zaun/openthesaurus-Zaun.json"
                ),
                expected: include_str!(
                    "../../tests/snapshots/openthesaurus/Zaun.snap"
                ),
            },
            SnapshotCase {
                word: "springen",
                fixture: include_str!(
                    "../../emacs-lisp/tests/files/openthesaurus/springen/openthesaurus-springen.json"
                ),
                expected: include_str!(
                    "../../tests/snapshots/openthesaurus/springen.snap"
                ),
            },
            SnapshotCase {
                word: "verlieben",
                fixture: include_str!(
                    "../../emacs-lisp/tests/files/openthesaurus/verlieben/openthesaurus-verlieben.json"
                ),
                expected: include_str!(
                    "../../tests/snapshots/openthesaurus/verlieben.snap"
                ),
            },
        ];

        for case in cases {
            let response = parse(
                case.word,
                &format!("https://www.openthesaurus.de/synonyme/{}", case.word),
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
    fn parses_bank_fixture_into_synonym_groups() {
        let response = parse(
            "Bank",
            "https://www.openthesaurus.de/synonyme/Bank",
            include_str!("../../emacs-lisp/tests/files/openthesaurus/Bank/openthesaurus-Bank.json"),
        )
        .expect("fixture parses");

        assert!(response.ok);
        assert_eq!(response.entries.len(), 1);

        let entry = &response.entries[0];
        assert_eq!(entry.synonym_groups.len(), 2);
        assert_eq!(entry.synonym_groups[0].categories, Vec::<String>::new());
        assert_eq!(
            entry.synonym_groups[0].items,
            vec!["Parkbank".to_owned(), "Sitzbank".to_owned()]
        );
        assert_eq!(
            entry.synonym_groups[1].categories,
            vec!["Ökonomie".to_owned()]
        );
        assert_eq!(
            entry.synonym_groups[1].items,
            vec![
                "Bankhaus".to_owned(),
                "Finanzinstitut".to_owned(),
                "Finanzinstitution".to_owned(),
                "Geldhaus".to_owned(),
                "Geldinstitut".to_owned(),
                "Geschäftsbank".to_owned(),
                "Kreditanstalt".to_owned(),
                "Kreditinstitut".to_owned(),
                "Sparkasse".to_owned(),
            ]
        );
    }

    #[test]
    fn preserves_empty_group_when_only_the_lemma_matches() {
        let response = parse(
            "Wolke",
            "https://www.openthesaurus.de/synonyme/Wolke",
            include_str!(
                "../../emacs-lisp/tests/files/openthesaurus/Wolke/openthesaurus-Wolke.json"
            ),
        )
        .expect("fixture parses");

        assert!(response.ok);
        let entry = &response.entries[0];
        assert_eq!(entry.synonym_groups.len(), 2);
        assert!(entry.synonym_groups[0].items.is_empty());
        assert_eq!(
            entry.synonym_groups[1].items,
            vec![
                "Cloud".to_owned(),
                "Datenwolke".to_owned(),
                "Rechnerwolke".to_owned(),
            ]
        );
    }

    #[test]
    fn reports_no_matches_for_empty_api_response() {
        let response = parse(
            "Nixdaexistiert",
            "https://www.openthesaurus.de/synonyme/Nixdaexistiert",
            include_str!(
                "../../emacs-lisp/tests/files/openthesaurus/Nixdaexistiert/openthesaurus-Nixdaexistiert.json"
            ),
        )
        .expect("fixture parses");

        assert!(!response.ok);
        assert_eq!(response.error.as_deref(), Some("No matches found"));
        assert!(response.entries.is_empty());
    }

    fn render_snapshot(response: &SourceResult) -> String {
        let mut lines = vec![
            format!("source={:?}", response.source),
            format!("ok={}", response.ok),
            format!(
                "url={}",
                match response.url.as_ref() {
                    Some(UrlValue::One(url)) => url.as_str(),
                    Some(UrlValue::Many(urls)) => {
                        panic!("unexpected multi-url snapshot: {urls:?}")
                    }
                    None => "-",
                }
            ),
        ];

        if let Some(error) = &response.error {
            lines.push(format!("error={error}"));
        }

        for entry in &response.entries {
            lines.push(format!("entry {} headword={}", entry.id, entry.headword));
            for (index, group) in entry.synonym_groups.iter().enumerate() {
                lines.push(format!(
                    "group {} categories=[{}] items=[{}]",
                    index + 1,
                    group.categories.join(", "),
                    group.items.join(", ")
                ));
            }
        }

        lines.join("\n")
    }
}
