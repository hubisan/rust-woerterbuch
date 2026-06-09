use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    Openthesaurus,
    Dwds,
    Duden,
    Wiktionary,
}

impl Source {
    /// Default order used by the CLI.
    pub const ALL: [Source; 4] = [
        Source::Openthesaurus,
        Source::Dwds,
        Source::Duden,
        Source::Wiktionary,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Section {
    Definitions,
    Examples,
    Synonyms,
    Origin,
    Idioms,
}

impl Section {
    pub const DEFAULTS: [Section; 5] = [
        Section::Definitions,
        Section::Examples,
        Section::Synonyms,
        Section::Origin,
        Section::Idioms,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupResponse {
    /// The exact word or expression provided on the command line.
    pub query: String,

    /// One result object per selected source, in stable source order.
    pub results: Vec<SourceResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceResult {
    pub source: Source,
    pub ok: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<UrlValue>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<DictionaryEntry>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl SourceResult {
    pub fn ok(source: Source, url: Option<UrlValue>, entries: Vec<DictionaryEntry>) -> Self {
        Self {
            source,
            ok: true,
            url,
            entries,
            error: None,
        }
    }

    pub fn error(source: Source, message: impl Into<String>) -> Self {
        Self {
            source,
            ok: false,
            url: None,
            entries: Vec::new(),
            error: Some(message.into()),
        }
    }

    pub fn retain_sections(&mut self, wanted: &[Section]) {
        for entry in &mut self.entries {
            entry.retain_sections(wanted);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UrlValue {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DictionaryEntry {
    pub id: usize,

    /// Source-specific homograph marker, for example DWDS hidx "1" or "2".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homograph: Option<String>,

    /// The headword as shown by the source.
    pub headword: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_of_speech: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grammar: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub etymology: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub idioms: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub synonym_groups: Vec<SynonymGroup>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub senses: Vec<Sense>,
}

impl DictionaryEntry {
    pub fn new(id: usize, headword: impl Into<String>) -> Self {
        Self {
            id,
            headword: headword.into(),
            ..Self::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.etymology.as_deref().unwrap_or_default().is_empty()
            && self.idioms.is_empty()
            && self.synonym_groups.is_empty()
            && self.senses.is_empty()
    }

    pub fn retain_sections(&mut self, wanted: &[Section]) {
        if !wanted.contains(&Section::Origin) {
            self.etymology = None;
        }
        if !wanted.contains(&Section::Idioms) {
            self.idioms.clear();
            for sense in &mut self.senses {
                sense.clear_idioms_recursive();
            }
        }
        if !wanted.contains(&Section::Synonyms) {
            self.synonym_groups.clear();
            for sense in &mut self.senses {
                sense.clear_synonyms_recursive();
            }
        }
        if !wanted.contains(&Section::Examples) {
            for sense in &mut self.senses {
                sense.clear_examples_recursive();
            }
        }
        if !wanted.contains(&Section::Definitions) {
            self.senses.clear();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Sense {
    pub id: usize,

    /// Source-specific identifier, for example a DWDS or Duden meaning id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub qualifiers: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub idioms: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subsenses: Vec<Sense>,
}

impl Sense {
    pub fn simple(id: usize, definition: impl Into<String>) -> Self {
        Self {
            id,
            definition: Some(definition.into()),
            ..Self::default()
        }
    }

    fn clear_examples_recursive(&mut self) {
        self.examples.clear();
        for child in &mut self.subsenses {
            child.clear_examples_recursive();
        }
    }

    fn clear_idioms_recursive(&mut self) {
        self.idioms.clear();
        for child in &mut self.subsenses {
            child.clear_idioms_recursive();
        }
    }

    fn clear_synonyms_recursive(&mut self) {
        self.synonyms.clear();
        for child in &mut self.subsenses {
            child.clear_synonyms_recursive();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SynonymGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sense: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,

    pub items: Vec<String>,
}

impl SynonymGroup {
    pub fn items(items: Vec<String>) -> Self {
        Self {
            items,
            ..Self::default()
        }
    }
}

pub fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn dedupe(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in values {
        if !value.is_empty() && !out.contains(&value) {
            out.push(value);
        }
    }
    out
}
