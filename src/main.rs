mod format;
mod http;
mod models;
mod sources;

use anyhow::Result;
use clap::Parser;
use futures::future::join_all;
use models::{LookupResponse, Section, Source};

#[derive(Debug, Parser)]
#[command(name = "woerterbuch")]
#[command(about = "Async CLI for German dictionary lookups", long_about = None)]
struct Cli {
    /// Lookup word or expression, for example "Bank".
    query: String,

    /// Print structured JSON instead of human-readable terminal output.
    #[arg(long)]
    json: bool,

    /// Comma-separated sources: openthesaurus,dwds,duden,wiktionary.
    #[arg(long, value_delimiter = ',', value_enum)]
    sources: Option<Vec<Source>>,

    /// Comma-separated sections: definitions,examples,synonyms,origin,idioms.
    #[arg(long, value_delimiter = ',', value_enum)]
    sections: Option<Vec<Section>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = http::build_client()?;

    let selected_sources = cli.sources.unwrap_or_else(|| Source::ALL.to_vec());
    let selected_sections = cli.sections.unwrap_or_else(|| Section::DEFAULTS.to_vec());
    let query = cli.query;

    let jobs: Vec<_> = selected_sources
        .into_iter()
        .map(|source| {
            let client = client.clone();
            let query = query.clone();
            let sections = selected_sections.clone();
            async move { sources::lookup_source(&client, source, &query, &sections).await }
        })
        .collect();

    let results = join_all(jobs).await;

    let response = LookupResponse { query, results };

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        format::print_human(&response);
    }

    Ok(())
}
