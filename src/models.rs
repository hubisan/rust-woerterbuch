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
        let keep_definitions = wanted.contains(&Section::Definitions);
        let keep_examples = wanted.contains(&Section::Examples);
        let keep_synonyms = wanted.contains(&Section::Synonyms);
        let keep_origin = wanted.contains(&Section::Origin);
        let keep_idioms = wanted.contains(&Section::Idioms);

        if !keep_origin {
            self.etymology = None;
        }
        if !keep_idioms {
            self.idioms.clear();
        }
        if !keep_synonyms {
            self.synonym_groups.clear();
        }

        for sense in &mut self.senses {
            sense.retain_sections_recursive(
                keep_definitions,
                keep_examples,
                keep_synonyms,
                keep_idioms,
            );
        }

        self.senses.retain(Sense::has_requested_content_recursive);
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

    fn retain_sections_recursive(
        &mut self,
        keep_definitions: bool,
        keep_examples: bool,
        keep_synonyms: bool,
        keep_idioms: bool,
    ) {
        if !keep_definitions {
            self.definition = None;
        }
        if !keep_examples {
            self.examples.clear();
        }
        if !keep_synonyms {
            self.synonyms.clear();
        }
        if !keep_idioms {
            self.idioms.clear();
        }

        for child in &mut self.subsenses {
            child.retain_sections_recursive(
                keep_definitions,
                keep_examples,
                keep_synonyms,
                keep_idioms,
            );
        }

        self.subsenses
            .retain(Sense::has_requested_content_recursive);
    }

    fn has_requested_content_recursive(&self) -> bool {
        self.definition
            .as_deref()
            .is_some_and(|value| !value.is_empty())
            || !self.examples.is_empty()
            || !self.idioms.is_empty()
            || !self.synonyms.is_empty()
            || self
                .subsenses
                .iter()
                .any(Sense::has_requested_content_recursive)
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

pub fn dedupe(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in values {
        if !value.is_empty() && !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry() -> DictionaryEntry {
        DictionaryEntry {
            id: 1,
            homograph: None,
            headword: "Bank".to_owned(),
            title: Some("Bank".to_owned()),
            part_of_speech: Some("Substantiv".to_owned()),
            grammar: Some("f".to_owned()),
            etymology: Some("aus dem Italienischen".to_owned()),
            idioms: vec!["auf die lange Bank schieben".to_owned()],
            synonym_groups: vec![SynonymGroup::items(vec!["Geldinstitut".to_owned()])],
            url: Some("https://example.test/bank".to_owned()),
            senses: vec![
                Sense {
                    id: 1,
                    source_id: Some("1".to_owned()),
                    label: Some("1".to_owned()),
                    definition: Some("Sitzgelegenheit".to_owned()),
                    qualifiers: vec!["allgemein".to_owned()],
                    examples: vec!["Sie sitzt auf der Bank.".to_owned()],
                    idioms: vec!["unter der Bank".to_owned()],
                    synonyms: vec!["Sitz".to_owned()],
                    image_url: Some("https://example.test/bank.jpg".to_owned()),
                    subsenses: vec![Sense {
                        id: 2,
                        source_id: Some("1a".to_owned()),
                        label: Some("1a".to_owned()),
                        definition: Some("ohne Lehne".to_owned()),
                        qualifiers: Vec::new(),
                        examples: vec!["Die Bank im Park".to_owned()],
                        idioms: vec!["auf der Bank sitzen".to_owned()],
                        synonyms: vec!["Parkbank".to_owned()],
                        image_url: None,
                        subsenses: Vec::new(),
                    }],
                },
                Sense {
                    id: 3,
                    source_id: Some("2".to_owned()),
                    label: Some("2".to_owned()),
                    definition: Some("Geldinstitut".to_owned()),
                    qualifiers: Vec::new(),
                    examples: Vec::new(),
                    idioms: Vec::new(),
                    synonyms: vec!["Kreditinstitut".to_owned()],
                    image_url: None,
                    subsenses: Vec::new(),
                },
            ],
        }
    }

    #[test]
    fn definitions_only_keeps_definitions() {
        let mut entry = sample_entry();

        entry.retain_sections(&[Section::Definitions]);

        assert_eq!(entry.etymology, None);
        assert!(entry.idioms.is_empty());
        assert!(entry.synonym_groups.is_empty());
        assert_eq!(entry.senses.len(), 2);
        assert!(entry.senses.iter().all(|sense| sense.examples.is_empty()));
        assert!(entry.senses.iter().all(|sense| sense.idioms.is_empty()));
        assert!(entry.senses.iter().all(|sense| sense.synonyms.is_empty()));
        assert_eq!(
            entry.senses[0].definition.as_deref(),
            Some("Sitzgelegenheit")
        );
        assert_eq!(
            entry.senses[0].subsenses[0].definition.as_deref(),
            Some("ohne Lehne")
        );
    }

    #[test]
    fn examples_only_keeps_example_bearing_senses_without_definitions() {
        let mut entry = sample_entry();

        entry.retain_sections(&[Section::Examples]);

        assert_eq!(entry.etymology, None);
        assert!(entry.idioms.is_empty());
        assert!(entry.synonym_groups.is_empty());
        assert_eq!(entry.senses.len(), 1);
        assert_eq!(entry.senses[0].definition, None);
        assert_eq!(
            entry.senses[0].examples,
            vec!["Sie sitzt auf der Bank.".to_owned()]
        );
        assert_eq!(entry.senses[0].subsenses.len(), 1);
        assert_eq!(
            entry.senses[0].subsenses[0].examples,
            vec!["Die Bank im Park".to_owned()]
        );
        assert_eq!(entry.senses[0].source_id.as_deref(), Some("1"));
        assert_eq!(entry.senses[0].label.as_deref(), Some("1"));
    }

    #[test]
    fn idioms_only_keeps_entry_and_sense_level_idioms() {
        let mut entry = sample_entry();

        entry.retain_sections(&[Section::Idioms]);

        assert_eq!(entry.idioms, vec!["auf die lange Bank schieben".to_owned()]);
        assert!(entry.synonym_groups.is_empty());
        assert_eq!(entry.senses.len(), 1);
        assert_eq!(entry.senses[0].definition, None);
        assert_eq!(entry.senses[0].idioms, vec!["unter der Bank".to_owned()]);
        assert_eq!(
            entry.senses[0].subsenses[0].idioms,
            vec!["auf der Bank sitzen".to_owned()]
        );
    }

    #[test]
    fn synonyms_only_keeps_entry_and_sense_level_synonyms() {
        let mut entry = sample_entry();

        entry.retain_sections(&[Section::Synonyms]);

        assert_eq!(entry.synonym_groups.len(), 1);
        assert_eq!(entry.senses.len(), 2);
        assert_eq!(entry.senses[0].definition, None);
        assert_eq!(entry.senses[0].synonyms, vec!["Sitz".to_owned()]);
        assert_eq!(
            entry.senses[0].subsenses[0].synonyms,
            vec!["Parkbank".to_owned()]
        );
        assert_eq!(entry.senses[1].synonyms, vec!["Kreditinstitut".to_owned()]);
    }

    #[test]
    fn origin_only_keeps_etymology() {
        let mut entry = sample_entry();

        entry.retain_sections(&[Section::Origin]);

        assert_eq!(entry.etymology.as_deref(), Some("aus dem Italienischen"));
        assert!(entry.idioms.is_empty());
        assert!(entry.synonym_groups.is_empty());
        assert!(entry.senses.is_empty());
    }

    #[test]
    fn examples_and_synonyms_can_survive_without_definitions() {
        let mut entry = sample_entry();

        entry.retain_sections(&[Section::Examples, Section::Synonyms]);

        assert_eq!(entry.senses.len(), 2);
        assert_eq!(entry.senses[0].definition, None);
        assert_eq!(
            entry.senses[0].examples,
            vec!["Sie sitzt auf der Bank.".to_owned()]
        );
        assert_eq!(entry.senses[0].synonyms, vec!["Sitz".to_owned()]);
        assert_eq!(entry.senses[1].definition, None);
        assert!(entry.senses[1].examples.is_empty());
        assert_eq!(entry.senses[1].synonyms, vec!["Kreditinstitut".to_owned()]);
    }

    #[test]
    fn empty_sections_prune_all_content_without_panicking() {
        let mut entry = sample_entry();

        entry.retain_sections(&[]);

        assert_eq!(entry.etymology, None);
        assert!(entry.idioms.is_empty());
        assert!(entry.synonym_groups.is_empty());
        assert!(entry.senses.is_empty());
    }
}
