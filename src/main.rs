mod format;
mod http;
mod models;
mod sources;

use anyhow::{bail, Result};
use clap::Parser;
use format::{OutputFormat, OutputLayout};
use futures::future::join_all;
use models::{LookupResponse, Section, Source};

#[derive(Debug, Parser)]
#[command(name = "woerterbuch")]
#[command(about = "Async CLI for German dictionary lookups", long_about = None)]
struct Cli {
    /// Lookup word or expression, for example "Bank".
    query: String,

    /// Print structured JSON instead of human-readable terminal output.
    ///
    /// This is kept as a backwards-compatible shortcut for `--format json`.
    #[arg(long, conflicts_with = "format")]
    json: bool,

    /// Output format: human,json,markdown,org.
    #[arg(long, value_enum, default_value = "human")]
    format: OutputFormat,

    /// Output layout: by-source or by-section.
    #[arg(long, value_enum)]
    layout: Option<OutputLayout>,

    /// Limit rendered examples per definition in text-like output formats.
    #[arg(long)]
    max_examples: Option<usize>,

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
    let output_format = if cli.json {
        OutputFormat::Json
    } else {
        cli.format
    };
    let layout = resolve_layout(output_format, cli.layout)?;

    print!(
        "{}",
        format::render(&response, output_format, layout, cli.max_examples)?
    );

    Ok(())
}

fn resolve_layout(
    output_format: OutputFormat,
    requested_layout: Option<OutputLayout>,
) -> Result<OutputLayout> {
    match output_format {
        OutputFormat::Json => {
            if requested_layout.is_some() {
                bail!("`--layout` is only supported for human, markdown, and org output.");
            }
            Ok(OutputLayout::BySource)
        }
        OutputFormat::Human | OutputFormat::Markdown | OutputFormat::Org => {
            Ok(requested_layout.unwrap_or(OutputLayout::BySource))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn markdown_defaults_to_by_source_layout() {
        let cli = Cli::parse_from(["woerterbuch", "Bank", "--format", "markdown"]);

        let output_format = if cli.json {
            OutputFormat::Json
        } else {
            cli.format
        };

        assert_eq!(
            resolve_layout(output_format, cli.layout).expect("layout"),
            OutputLayout::BySource
        );
    }

    #[test]
    fn markdown_accepts_by_section_layout() {
        let cli = Cli::parse_from([
            "woerterbuch",
            "Bank",
            "--format",
            "markdown",
            "--layout",
            "by-section",
        ]);

        let output_format = if cli.json {
            OutputFormat::Json
        } else {
            cli.format
        };

        assert_eq!(
            resolve_layout(output_format, cli.layout).expect("layout"),
            OutputLayout::BySection
        );
    }

    #[test]
    fn json_rejects_explicit_layout() {
        let error = resolve_layout(OutputFormat::Json, Some(OutputLayout::BySource))
            .expect_err("json layout should fail");

        assert_eq!(
            error.to_string(),
            "`--layout` is only supported for human, markdown, and org output."
        );
    }
}
