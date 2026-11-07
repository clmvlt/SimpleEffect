//! Graphics and keyframe commands (prompt 15): new texts, shapes and images, batched keyframes,
//! animation presets, and the font catalogue. Edits are one undoable transaction each, through
//! `se_app::ProjectService`.

use se_app::animation_preset_infos;
use se_domain::{AnimationPhase, Flicks, ItemId, ProjectId};
use se_protocol::{AddGraphicRequest, CommandResult, ItemAnimation, ProtocolError};

use super::HandlerContext;

/// `setItemAnimations`.
pub async fn set_item_animations(
    context: &HandlerContext,
    project_id: ProjectId,
    animations: &[ItemAnimation],
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .set_item_animations(project_id, animations)
        .await?;
    Ok(CommandResult::SetItemAnimations)
}

/// `addGraphicItem`.
pub async fn add_graphic_item(
    context: &HandlerContext,
    request: &AddGraphicRequest,
) -> Result<CommandResult, ProtocolError> {
    let placed = context.projects.add_graphic_item(request).await?;
    Ok(CommandResult::AddGraphicItem(placed))
}

/// The arguments of `applyAnimationPreset` after the project.
pub struct PresetRequest<'a> {
    /// The items animated.
    pub item_ids: &'a [ItemId],
    /// Id of the preset.
    pub preset: &'a str,
    /// Entrance, exit or emphasis.
    pub phase: AnimationPhase,
    /// How long it lasts.
    pub duration: Flicks,
    /// Start relative to each item.
    pub start: Option<Flicks>,
}

/// `applyAnimationPreset`.
pub async fn apply_animation_preset(
    context: &HandlerContext,
    project_id: ProjectId,
    request: PresetRequest<'_>,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .apply_animation_preset(
            project_id,
            request.item_ids,
            request.preset,
            request.phase,
            request.duration,
            request.start,
        )
        .await?;
    Ok(CommandResult::ApplyAnimationPreset)
}

/// `listAnimationPresets`.
pub fn list_animation_presets() -> Result<CommandResult, ProtocolError> {
    Ok(CommandResult::ListAnimationPresets {
        presets: animation_preset_infos()?,
    })
}

/// `listFonts`.
pub async fn list_fonts(context: &HandlerContext) -> CommandResult {
    CommandResult::ListFonts {
        families: context.fonts.families().await,
    }
}
