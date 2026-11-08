//! `search_transcript`: where words are said in the sequence. The project's transcripts are
//! searched (full-text index: accents and case ignored, the last word as a prefix), then each
//! matching phrase is placed where the sequence plays it (none when it was cut, several when it
//! is used twice), with the times of the matching words.

use schemars::JsonSchema;
use se_domain::{SequenceSegment, SequenceWord};
use serde::{Deserialize, Serialize};

use crate::{
    context::{ToolContext, find_sequence},
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::{RangeInput, TimeOutput, seconds},
};

/// Results by default.
const DEFAULT_LIMIT: usize = 20;
/// Most results.
const MAX_LIMIT: usize = 50;

/// The tool.
pub struct SearchTranscript;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Words to find, e.g. "abonnez-vous" or "prix euros" (every word must appear in the phrase;
    /// accents and case are ignored; the last word may be the start of a word).
    pub query: String,
    /// Sequence id or exact name; default: your task's sequence.
    pub sequence: Option<String>,
    /// Only phrases intersecting this range of the sequence; default: the whole sequence.
    pub range: Option<RangeInput>,
    /// Most results (default 20, at most 50).
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct Answer {
    sequence_id: String,
    time_base: &'static str,
    results: Vec<Found>,
    /// Matching phrases of the project's media that this sequence does not play (cut or unused).
    elsewhere: usize,
}

#[derive(Debug, Serialize)]
struct Found {
    start: TimeOutput,
    end: TimeOutput,
    text: String,
    item_id: String,
    asset_id: String,
    /// The words of the phrase that match the query.
    matches: Vec<MatchedWord>,
}

#[derive(Debug, Serialize)]
struct MatchedWord {
    text: String,
    start_seconds: f64,
    end_seconds: f64,
}

impl McpTool for SearchTranscript {
    type Input = Input;

    const NAME: &'static str = "search_transcript";

    const DESCRIPTION: &'static str = "Finds where words are said in the sequence: phrases of \
        the transcripts holding every word of `query` (accents and case ignored), each with its \
        sequence times and the times of the matching words, sorted by time. Use it to locate a \
        passage before cutting or titling it; then read around it with get_transcript.";

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let terms: Vec<String> = fold(&input.query)
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        if terms.is_empty() {
            return Err(ToolError::invalid_arguments("query has no words to search"));
        }
        let project = context.project().await?;
        let sequence = find_sequence(&project, &context.agent, input.sequence.as_deref())?;
        let range = match &input.range {
            Some(range) => Some(range.to_range(sequence.format.frame_rate)?),
            None => None,
        };
        let transcription = &context.deps.transcription;
        let hits = transcription
            .search(context.agent.project_id, &input.query)
            .await?;
        let words = transcription
            .sequence_words(context.agent.project_id, sequence.id)
            .await?
            .transcript;
        let mut results = Vec::new();
        let mut elsewhere = 0;
        for hit in &hits {
            let placed: Vec<&SequenceSegment> = words
                .segments
                .iter()
                .filter(|segment| {
                    segment.asset_id == hit.asset_id && segment.source_segment == hit.segment
                })
                .filter(|segment| {
                    range.is_none_or(|range| {
                        segment.start < range.end() && range.start() < segment.end
                    })
                })
                .collect();
            if placed.is_empty() {
                elsewhere += 1;
            }
            results.extend(
                placed
                    .into_iter()
                    .map(|segment| found(segment, &words.words, &terms)),
            );
        }
        results.sort_by(|left, right| left.start.seconds.total_cmp(&right.start.seconds));
        results.truncate(input.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT));
        ToolOutput::json(&Answer {
            sequence_id: sequence.id.to_string(),
            time_base: "sequence",
            results,
            elsewhere,
        })
    }
}

/// A placed phrase and its matching words.
fn found(segment: &SequenceSegment, words: &[SequenceWord], terms: &[String]) -> Found {
    let first = usize::try_from(segment.first_word).unwrap_or(usize::MAX);
    let count = usize::try_from(segment.word_count).unwrap_or(0);
    let phrase: Vec<&SequenceWord> = words.iter().skip(first).take(count).collect();
    let matches = phrase
        .iter()
        .filter(|word| {
            let folded = fold(&word.text);
            terms.iter().any(|term| {
                folded
                    .split_whitespace()
                    .any(|part| part.starts_with(term.as_str()) || part == term)
            })
        })
        .map(|word| MatchedWord {
            text: word.text.clone(),
            start_seconds: seconds(word.start),
            end_seconds: seconds(word.end),
        })
        .collect();
    Found {
        start: segment.start.into(),
        end: segment.end.into(),
        text: phrase
            .iter()
            .map(|word| word.text.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        item_id: segment.item_id.to_string(),
        asset_id: segment.asset_id.to_string(),
        matches,
    }
}

/// Lowercase, French accents removed, punctuation and apostrophes as spaces.
fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' | 'á' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' | 'í' => 'i',
            'ô' | 'ö' | 'ó' => 'o',
            'ù' | 'û' | 'ü' | 'ú' => 'u',
            'ç' => 'c',
            'ÿ' => 'y',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::fold;

    #[test]
    fn folds_accents_case_and_punctuation() {
        assert_eq!(
            fold("Abonnez-vous à la Chaîne !"),
            "abonnez vous a la chaine  "
        );
        assert_eq!(fold("l'Été"), "l ete");
    }
}
