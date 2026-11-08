//! The tool catalogue: one tool per file, documented in `docs/ai-tools.md` (the contract with
//! agents). Adding a tool: recipe in `crates/se-mcp/CLAUDE.md`.

pub mod get_audio_analysis;
pub mod get_brand_kit;
pub mod get_frames;
pub mod get_item;
pub mod get_project_overview;
pub mod get_scene_changes;
pub mod get_source_frames;
pub mod get_timeline;
pub mod get_transcript;
pub mod list_animation_presets;
pub mod list_assets;
pub mod list_templates;

use crate::registry::ToolEntry;

/// Every tool served, in the order agents see them (broad reads first).
#[must_use]
pub fn catalogue() -> Vec<ToolEntry> {
    let entries = [
        ToolEntry::of::<get_project_overview::GetProjectOverview>(),
        ToolEntry::of::<get_timeline::GetTimeline>(),
        ToolEntry::of::<get_item::GetItem>(),
        ToolEntry::of::<list_assets::ListAssets>(),
        ToolEntry::of::<get_frames::GetFrames>(),
        ToolEntry::of::<get_source_frames::GetSourceFrames>(),
        ToolEntry::of::<get_audio_analysis::GetAudioAnalysis>(),
        ToolEntry::of::<get_scene_changes::GetSceneChanges>(),
        ToolEntry::of::<get_transcript::GetTranscript>(),
        ToolEntry::of::<list_animation_presets::ListAnimationPresets>(),
        ToolEntry::of::<get_brand_kit::GetBrandKit>(),
        ToolEntry::of::<list_templates::ListTemplates>(),
    ];
    entries
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(problem) => {
                // Caught by `every_tool_has_an_object_schema`; never ship a broken tool.
                tracing::error!(%problem, "an MCP tool has an invalid input schema");
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_has_an_object_schema_and_a_unique_name() {
        let tools = catalogue();
        assert_eq!(tools.len(), 12);
        let mut names: Vec<&str> = tools.iter().map(ToolEntry::name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 12);
        for tool in &tools {
            assert_eq!(
                tool.definition.input_schema["type"],
                "object",
                "{}",
                tool.name()
            );
            assert!(
                tool.definition
                    .annotations
                    .as_ref()
                    .and_then(|annotations| annotations.read_only_hint)
                    .unwrap_or(false),
                "{} must be read-only",
                tool.name()
            );
        }
    }
}
