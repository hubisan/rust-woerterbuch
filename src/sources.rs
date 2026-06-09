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

fn source_timeout(source: Source) -> u64 {
    match source {
        Source::Dwds => 10,
        Source::Duden => 20,
        Source::Openthesaurus => 5,
        Source::Wiktionary => 10,
    }
}
