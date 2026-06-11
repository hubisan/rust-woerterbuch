pub mod duden;
pub mod dwds;
pub mod openthesaurus;
pub mod wiktionary;

use crate::models::{Section, Source, SourceResult};
use reqwest::Client;
use std::time::Duration;
use tokio::time::timeout;

pub async fn lookup_source(
    client: &Client,
    source: Source,
    query: &str,
    sections: &[Section],
) -> SourceResult {
    if !source_supports_any_section(source, sections) {
        return SourceResult::ok(source, None, Vec::new());
    }

    let timeout_seconds = source_timeout(source);
    let result = timeout(Duration::from_secs(timeout_seconds), async {
        match source {
            Source::Duden => duden::lookup(client, query).await,
            Source::Dwds => dwds::lookup(client, query).await,
            Source::Wiktionary => wiktionary::lookup(client, query).await,
            Source::Openthesaurus => openthesaurus::lookup(client, query).await,
        }
    })
    .await;

    match result {
        Ok(Ok(mut source_result)) => {
            source_result.retain_sections(sections);
            source_result
        }
        Ok(Err(err)) => SourceResult::error(source, err.to_string()),
        Err(_) => SourceResult::error(source, format!("Timeout after {timeout_seconds}s")),
    }
}

pub fn source_supports_any_section(source: Source, sections: &[Section]) -> bool {
    let supported = match source {
        Source::Openthesaurus => &[Section::Synonyms][..],
        Source::Dwds => &[
            Section::Definitions,
            Section::Examples,
            Section::Origin,
            Section::Idioms,
        ][..],
        Source::Duden | Source::Wiktionary => &[
            Section::Definitions,
            Section::Examples,
            Section::Synonyms,
            Section::Origin,
            Section::Idioms,
        ][..],
    };

    sections.iter().any(|section| supported.contains(section))
}

fn source_timeout(source: Source) -> u64 {
    match source {
        Source::Dwds => 10,
        Source::Duden => 20,
        Source::Openthesaurus => 5,
        Source::Wiktionary => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_capabilities_match_expected_sections() {
        assert!(source_supports_any_section(
            Source::Openthesaurus,
            &[Section::Synonyms]
        ));
        assert!(!source_supports_any_section(
            Source::Openthesaurus,
            &[Section::Examples]
        ));
        assert!(source_supports_any_section(
            Source::Dwds,
            &[Section::Idioms]
        ));
        assert!(!source_supports_any_section(
            Source::Dwds,
            &[Section::Synonyms]
        ));
        assert!(source_supports_any_section(
            Source::Duden,
            &[Section::Synonyms]
        ));
        assert!(source_supports_any_section(
            Source::Wiktionary,
            &[Section::Origin]
        ));
        assert!(!source_supports_any_section(Source::Duden, &[]));
    }

    #[tokio::test]
    async fn skipped_sources_return_empty_success_results() {
        let client = Client::new();

        let result =
            lookup_source(&client, Source::Openthesaurus, "Bank", &[Section::Examples]).await;

        assert!(result.ok);
        assert_eq!(result.source, Source::Openthesaurus);
        assert!(result.url.is_none());
        assert!(result.entries.is_empty());
        assert!(result.error.is_none());
    }
}
