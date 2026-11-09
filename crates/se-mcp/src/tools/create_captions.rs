//! `create_captions`: makes captions from the transcript of a range, on the caption track, with
//! a predefined or existing style; captions of the track crossing the range are replaced.

use schemars::JsonSchema;
use se_app::agent_edits::{AgentEdit, NewAgentCaptions};
use se_domain::{CaptionStyleId, CaptionWord, ItemKind, TimeRange};
use se_protocol::{CaptionSegmentation, CaptionStyleChoice};
use serde::{Deserialize, Serialize};

use crate::{
    context::{ToolContext, default_range, find_sequence},
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::{RangeInput, TimeOutput},
    writes::{app_error, apply},
};

/// Predefined styles agents may name.
pub const CAPTION_PRESETS: [&str; 4] = ["tiktokBold", "minimal", "karaoke", "box"];

/// The tool.
pub struct CreateCaptions;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The range to caption (default: your task's range, else the whole sequence). Captions of
    /// the caption track crossing it are replaced, so the range grows to cover them whole.
    pub range: Option<RangeInput>,
    /// A predefined style — `tiktokBold` (one bold upper-case line, words popping),
    /// `minimal` (two plain lines), `karaoke` (spoken words filling in yellow), `box` (lines on
    /// dark boxes) —, `brand_kit` (the project's brand kit look) or the `style_id` of a caption
    /// style of the project. Default: the style of the captions replaced, else the brand kit's
    /// look when the project has a kit, else `minimal`.
    pub style: Option<String>,
    /// Longest line in characters (4–80; default: the style's).
    pub max_chars_per_line: Option<u32>,
    /// Lines per caption (1–4; default: the style's).
    pub max_lines: Option<u32>,
}

#[derive(Debug, Serialize)]
struct CaptionLine {
    item_id: String,
    start: TimeOutput,
    end: TimeOutput,
    text: String,
}

#[derive(Debug, Serialize)]
struct Output {
    revision: Option<u64>,
    captions: Vec<CaptionLine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    style_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    track_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    untranscribed: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

impl McpTool for CreateCaptions {
    type Input = Input;

    const NAME: &'static str = "create_captions";

    const DESCRIPTION: &'static str = "Makes captions from the transcript of a range: words \
        grouped into readable captions (line length, lines, pauses, punctuation), each word timed \
        so styles can highlight the word being spoken. They go on the caption track (a video \
        track « Sous-titres » created on top when missing; creating it needs a task over every \
        track) and replace the captions of that track crossing the range, in one undoable \
        transaction. Returns each caption's id, time and text, and the style used. Media not \
        transcribed yet get no caption (listed in `untranscribed`).";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let sequence = find_sequence(&project, &context.agent, None)?;
        let range = match &input.range {
            Some(range) => range.to_range(sequence.format.frame_rate)?,
            None => default_range(&context.agent, sequence)?,
        };
        let style = input.style.as_deref().map(style_choice).transpose()?;
        let (words, untranscribed) = context
            .deps
            .captions
            .spoken_words(context.agent.project_id, sequence.id)
            .await
            .map_err(app_error)?;
        let untranscribed: Vec<String> = untranscribed.iter().map(ToString::to_string).collect();
        let spoken = words
            .iter()
            .any(|word| range.contains(CaptionWord::new("", word.start, word.end).middle()));
        if !spoken {
            return ToolOutput::json(&Output {
                revision: None,
                captions: Vec::new(),
                style_id: None,
                track_id: None,
                note: Some(nothing_spoken(range, untranscribed.is_empty())),
                untranscribed,
            });
        }
        let edit = AgentEdit::AddCaptions(Box::new(NewAgentCaptions {
            range,
            words,
            style,
            segmentation: Some(CaptionSegmentation {
                max_chars_per_line: input.max_chars_per_line,
                max_lines: input.max_lines,
                ..CaptionSegmentation::default()
            }),
        }));
        let report = apply(&context, Self::NAME, &[edit], |_| None).await?;
        let created = report.created.first().cloned().unwrap_or_default();
        let mut output = Output {
            revision: Some(report.revision),
            captions: Vec::new(),
            style_id: None,
            track_id: None,
            untranscribed,
            note: None,
        };
        for track in report.project.sequences.iter().flat_map(|s| &s.tracks) {
            for item in track.items.iter().filter(|item| created.contains(&item.id)) {
                if let ItemKind::Caption(caption) = &item.kind {
                    output.style_id = Some(caption.style_id.to_string());
                    output.track_id = Some(track.id.to_string());
                    output.captions.push(CaptionLine {
                        item_id: item.id.to_string(),
                        start: item.timeline_range.start().into(),
                        end: item.timeline_range.end().into(),
                        text: caption.text(),
                    });
                }
            }
        }
        ToolOutput::json(&output)
    }
}

/// The `style` naming the project's brand kit look (prompt 25).
pub const BRAND_KIT_STYLE: &str = "brand_kit";

/// A preset name, `brand_kit` (the project's brand kit look) or a caption style id.
///
/// # Errors
/// [`ToolError::invalid_arguments`] listing the presets.
pub fn style_choice(text: &str) -> Result<CaptionStyleChoice, ToolError> {
    if text == BRAND_KIT_STYLE {
        return Ok(CaptionStyleChoice::BrandKit);
    }
    if CAPTION_PRESETS.contains(&text) {
        return Ok(CaptionStyleChoice::Preset {
            preset: text.to_owned(),
        });
    }
    CaptionStyleId::parse(text)
        .map(|style_id| CaptionStyleChoice::Existing { style_id })
        .map_err(|_| {
            ToolError::invalid_arguments(format!(
                "style '{text}' is neither a preset ({}), `brand_kit` nor a caption style id \
                 (get_item of a caption shows its style_id)",
                CAPTION_PRESETS.join(", ")
            ))
        })
}

fn nothing_spoken(range: TimeRange, all_transcribed: bool) -> String {
    let place = format!(
        "between {} and {}",
        TimeOutput::from(range.start()).timecode,
        TimeOutput::from(range.end()).timecode
    );
    if all_transcribed {
        format!("no word is spoken {place}: nothing was captioned")
    } else {
        format!(
            "no transcribed word {place}: the media listed in `untranscribed` must be \
             transcribed first (transcribe_media)"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_are_presets_or_ids() {
        assert!(matches!(
            style_choice("karaoke"),
            Ok(CaptionStyleChoice::Preset { .. })
        ));
        assert!(matches!(
            style_choice("00000000-0001-7000-8000-000000000000"),
            Ok(CaptionStyleChoice::Existing { .. })
        ));
        assert!(matches!(
            style_choice("brand_kit"),
            Ok(CaptionStyleChoice::BrandKit)
        ));
        assert!(style_choice("disco").is_err());
    }
}
