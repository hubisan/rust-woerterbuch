const MAX_SHARP_S_CANDIDATES: usize = 16;

pub fn generate_sharp_s_candidates(query: &str) -> Vec<String> {
    let occurrences = sharp_s_occurrences(query);
    let mut masks: Vec<Vec<usize>> = vec![Vec::new()];

    for (index, _occurrence) in occurrences.iter().enumerate() {
        let snapshot = masks.clone();
        for mask in snapshot {
            if masks.len() >= MAX_SHARP_S_CANDIDATES {
                return build_candidates(query, &occurrences, &masks);
            }

            let mut next = mask;
            next.push(index);
            masks.push(next);
        }
    }

    build_candidates(query, &occurrences, &masks)
}

#[derive(Clone, Copy)]
struct SharpSOccurrence {
    start: usize,
    end: usize,
}

fn sharp_s_occurrences(query: &str) -> Vec<SharpSOccurrence> {
    let mut out = Vec::new();
    let mut search_from = 0usize;

    while let Some(relative) = query[search_from..].find("ss") {
        let start = search_from + relative;
        let end = start + 2;
        if sharp_s_candidate_p(query, start, end) {
            out.push(SharpSOccurrence { start, end });
        }
        search_from = end;
    }

    out
}

fn sharp_s_candidate_p(query: &str, start: usize, end: usize) -> bool {
    if high_confidence_sharp_s(query, start) {
        return true;
    }

    medium_confidence_sharp_s(query, start, end)
}

fn high_confidence_sharp_s(query: &str, start: usize) -> bool {
    let prefix = query[..start].to_lowercase();
    ["ei", "au", "eu", "äu", "ie"]
        .iter()
        .any(|pattern| prefix.ends_with(pattern))
}

fn medium_confidence_sharp_s(query: &str, start: usize, end: usize) -> bool {
    let prefix_char = query[..start].chars().next_back();
    if !prefix_char.is_some_and(is_vowel) {
        return false;
    }

    let suffix = token_suffix(&query[end..]).to_lowercase();
    matches!(suffix.as_str(), "" | "e" | "em" | "en" | "er" | "es")
}

fn token_suffix(input: &str) -> &str {
    let mut end = 0usize;
    for (index, ch) in input.char_indices() {
        if !(ch.is_alphabetic() || ch == '\'') {
            break;
        }
        end = index + ch.len_utf8();
    }
    &input[..end]
}

fn is_vowel(ch: char) -> bool {
    matches!(
        ch.to_ascii_lowercase(),
        'a' | 'e' | 'i' | 'o' | 'u' | 'ä' | 'ö' | 'ü' | 'y'
    )
}

fn build_candidates(
    query: &str,
    occurrences: &[SharpSOccurrence],
    masks: &[Vec<usize>],
) -> Vec<String> {
    let mut candidates = Vec::new();

    for mask in masks {
        let candidate = apply_occurrence_mask(query, occurrences, mask);
        if !candidates.contains(&candidate) {
            candidates.push(candidate);
        }
    }

    candidates
}

fn apply_occurrence_mask(query: &str, occurrences: &[SharpSOccurrence], mask: &[usize]) -> String {
    let mut out = String::with_capacity(query.len());
    let mut cursor = 0usize;

    for index in mask {
        let occurrence = occurrences[*index];
        out.push_str(&query[cursor..occurrence.start]);
        out.push('ß');
        cursor = occurrence.end;
    }

    out.push_str(&query[cursor..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn keeps_original_first_and_generates_common_sharp_s_candidates() {
        assert_eq!(
            generate_sharp_s_candidates("Strasse"),
            vec!["Strasse".to_owned(), "Straße".to_owned()]
        );
        assert_eq!(
            generate_sharp_s_candidates("Gruss"),
            vec!["Gruss".to_owned(), "Gruß".to_owned()]
        );
        assert_eq!(
            generate_sharp_s_candidates("heiss"),
            vec!["heiss".to_owned(), "heiß".to_owned()]
        );
        assert_eq!(
            generate_sharp_s_candidates("draussen"),
            vec!["draussen".to_owned(), "draußen".to_owned()]
        );
        assert_eq!(
            generate_sharp_s_candidates("fliessen"),
            vec!["fliessen".to_owned(), "fließen".to_owned()]
        );
    }

    #[test]
    fn supports_multiple_occurrences_deterministically() {
        assert_eq!(
            generate_sharp_s_candidates("heisse Strasse"),
            vec![
                "heisse Strasse".to_owned(),
                "heiße Strasse".to_owned(),
                "heisse Straße".to_owned(),
                "heiße Straße".to_owned(),
            ]
        );
    }

    #[test]
    fn preserves_ambiguous_original_and_only_adds_candidates() {
        let candidates = generate_sharp_s_candidates("Masse");
        assert_eq!(candidates[0], "Masse");
        assert!(candidates.contains(&"Maße".to_owned()));
    }

    #[test]
    fn caps_candidate_growth() {
        let candidates = generate_sharp_s_candidates("heisse heisse heisse heisse heisse");
        assert!(candidates.len() <= MAX_SHARP_S_CANDIDATES);
        assert_eq!(candidates[0], "heisse heisse heisse heisse heisse");
    }
}
