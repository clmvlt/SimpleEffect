//! The tool catalogue: one tool per file, documented in `docs/ai-tools.md` (the contract with
//! agents). Adding a tool: recipe in `crates/se-mcp/CLAUDE.md`.

pub mod add_image;
pub mod add_marker;
pub mod add_shape;
pub mod add_text;
pub mod add_transition;
pub mod apply_animation_preset;
pub mod apply_batch;
pub mod delete_items;
pub mod get_audio_analysis;
pub mod get_brand_kit;
pub mod get_frames;
pub mod get_item;
pub mod get_project_overview;
pub mod get_scene_changes;
pub mod get_source_frames;
pub mod get_task_changes;
pub mod get_timeline;
pub mod get_transcript;
pub mod insert_gap;
pub mod list_animation_presets;
pub mod list_assets;
pub mod list_templates;
pub mod move_item;
pub mod ripple_delete_range;
pub mod search_transcript;
pub mod set_keyframes;
pub mod set_volume;
pub mod split_at;
pub mod trim_item;
pub mod update_item;

use crate::registry::ToolEntry;

/// Every tool served, in the order agents see them: broad reads first, then writes.
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
        ToolEntry::of::<search_transcript::SearchTranscript>(),
        ToolEntry::of::<list_animation_presets::ListAnimationPresets>(),
        ToolEntry::of::<get_brand_kit::GetBrandKit>(),
        ToolEntry::of::<list_templates::ListTemplates>(),
        ToolEntry::of::<get_task_changes::GetTaskChanges>(),
        ToolEntry::of::<add_text::AddText>(),
        ToolEntry::of::<add_shape::AddShape>(),
        ToolEntry::of::<add_image::AddImage>(),
        ToolEntry::of::<update_item::UpdateItem>(),
        ToolEntry::of::<set_keyframes::SetKeyframes>(),
        ToolEntry::of::<apply_animation_preset::ApplyAnimationPreset>(),
        ToolEntry::of::<set_volume::SetVolume>(),
        ToolEntry::of::<move_item::MoveItem>(),
        ToolEntry::of::<trim_item::TrimItem>(),
        ToolEntry::of::<split_at::SplitAt>(),
        ToolEntry::of::<delete_items::DeleteItems>(),
        ToolEntry::of::<ripple_delete_range::RippleDeleteRange>(),
        ToolEntry::of::<insert_gap::InsertGap>(),
        ToolEntry::of::<add_transition::AddTransition>(),
        ToolEntry::of::<add_marker::AddMarker>(),
        ToolEntry::of::<apply_batch::ApplyBatch>(),
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

    /// Tools that change the project.
    const WRITE_TOOLS: [&str; 16] = [
        "add_text",
        "add_shape",
        "add_image",
        "update_item",
        "set_keyframes",
        "apply_animation_preset",
        "set_volume",
        "move_item",
        "trim_item",
        "split_at",
        "delete_items",
        "ripple_delete_range",
        "insert_gap",
        "add_transition",
        "add_marker",
        "apply_batch",
    ];

    #[test]
    fn every_tool_has_an_object_schema_and_a_unique_name() {
        let tools = catalogue();
        assert_eq!(tools.len(), 30);
        let mut names: Vec<&str> = tools.iter().map(ToolEntry::name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 30);
        for tool in &tools {
            assert_eq!(
                tool.definition.input_schema["type"],
                "object",
                "{}",
                tool.name()
            );
            let read_only = tool
                .definition
                .annotations
                .as_ref()
                .and_then(|annotations| annotations.read_only_hint)
                .unwrap_or(false);
            assert_eq!(
                read_only,
                !WRITE_TOOLS.contains(&tool.name()),
                "{} is annotated read-only = {read_only}",
                tool.name()
            );
        }
    }

    #[test]
    fn every_write_tool_can_run_in_a_batch() {
        for tool in WRITE_TOOLS.iter().filter(|tool| **tool != "apply_batch") {
            assert!(
                super::apply_batch::batchable(tool),
                "{tool} is missing from apply_batch"
            );
        }
    }
}
