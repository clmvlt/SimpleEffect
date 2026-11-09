//! Caption styles in plans: choosing the style of new captions, changing the look of some
//! captions with copy on write, removing the styles an edit left unused.

use std::collections::{BTreeMap, BTreeSet};

use se_domain::{
    AddCaptionStyle, CaptionPatch, CaptionStyle, CaptionStyleId, CaptionStylePatch,
    DEFAULT_CAPTION_PRESET, DomainError, Format, ItemId, ItemKind, ItemPatch, Project,
    RemoveCaptionStyle, UpdateCaptionStyle, UpdateItem, caption_style_from_preset,
};
use se_protocol::CaptionStyleChoice;

use crate::{
    AppError,
    edit_plan::{EditPlan, find_item},
};

/// The style new captions will use, before anything is planned.
#[derive(Debug, Clone)]
pub(crate) enum StyleToUse {
    /// A style of the project.
    Existing(CaptionStyle),
    /// A new style (from a preset), to add with [`StyleToUse::commit`].
    New(CaptionStyle),
}

impl StyleToUse {
    /// The style's values.
    pub(crate) const fn style(&self) -> &CaptionStyle {
        match self {
            Self::Existing(style) | Self::New(style) => style,
        }
    }

    /// Adds the style to the plan when it is new; returns its id.
    pub(crate) fn commit(self, plan: &mut EditPlan<'_>) -> CaptionStyleId {
        match self {
            Self::Existing(style) => style.id,
            Self::New(style) => {
                let id = style.id;
                plan.push(AddCaptionStyle { index: None, style });
                id
            }
        }
    }
}

/// Highest bottom of captions made from a preset in a portrait frame: the platforms of vertical
/// videos cover the bottom third with their interface.
pub const PORTRAIT_CAPTION_BOTTOM: f64 = 0.64;

/// The style asked for: a preset or the brand kit's look (a new style, raised above the bottom
/// third in portrait frames), a style of `project`, or `fallback` (the style of the captions
/// being replaced), else the brand kit's look, else the default preset.
///
/// # Errors
/// An unknown preset or style, id generator failures.
pub(crate) fn style_to_use(
    plan: &EditPlan<'_>,
    project: &Project,
    format: &Format,
    choice: Option<&CaptionStyleChoice>,
    fallback: Option<CaptionStyleId>,
) -> Result<StyleToUse, AppError> {
    let existing = |id: CaptionStyleId| {
        project
            .caption_styles
            .iter()
            .find(|style| style.id == id)
            .cloned()
            .map(StyleToUse::Existing)
            .ok_or(AppError::Domain(DomainError::CaptionStyleNotFound {
                style: id,
            }))
    };
    match (choice, fallback) {
        (Some(CaptionStyleChoice::Existing { style_id }), _) => existing(*style_id),
        (Some(CaptionStyleChoice::Preset { preset }), _) => new_from_preset(plan, preset, format),
        (Some(CaptionStyleChoice::BrandKit), _) => match &project.brand_kit {
            Some(kit) => new_from_look(plan, &kit.caption_style, format),
            None => Err(AppError::InvalidEdit {
                reason: "this project has no brand kit: choose a caption preset or a style of \
                         the project"
                    .to_owned(),
            }),
        },
        (None, Some(id)) => existing(id),
        (None, None) => match &project.brand_kit {
            Some(kit) => new_from_look(plan, &kit.caption_style, format),
            None => new_from_preset(plan, DEFAULT_CAPTION_PRESET, format),
        },
    }
}

/// A new style with the values of `look` (a brand kit's caption look), raised above the bottom
/// third in portrait frames like presets.
fn new_from_look(
    plan: &EditPlan<'_>,
    look: &CaptionStyle,
    format: &Format,
) -> Result<StyleToUse, AppError> {
    let mut style = look.clone();
    style.id = plan.ids().caption_style()?;
    if format.height > format.width {
        style.position.y = style.position.y.min(PORTRAIT_CAPTION_BOTTOM);
    }
    Ok(StyleToUse::New(style))
}

fn new_from_preset(
    plan: &EditPlan<'_>,
    preset: &str,
    format: &Format,
) -> Result<StyleToUse, AppError> {
    let id = plan.ids().caption_style()?;
    let mut style = caption_style_from_preset(preset, id)?;
    if format.height > format.width {
        style.position.y = style.position.y.min(PORTRAIT_CAPTION_BOTTOM);
    }
    Ok(StyleToUse::New(style))
}

/// The captions among `items` (refused otherwise), with their style.
///
/// # Errors
/// An empty list, unknown items, items that are not captions.
pub(crate) fn caption_styles_of(
    project: &Project,
    items: &[ItemId],
) -> Result<Vec<(ItemId, CaptionStyleId)>, AppError> {
    if items.is_empty() {
        return Err(AppError::InvalidEdit {
            reason: "no caption given: list the captions to change".to_owned(),
        });
    }
    let unique: BTreeSet<ItemId> = items.iter().copied().collect();
    unique
        .into_iter()
        .map(|id| match &find_item(project, id)?.item.kind {
            ItemKind::Caption(caption) => Ok((id, caption.style_id)),
            other => Err(not_a_caption(id, other.label())),
        })
        .collect()
}

/// The error for an item that is not a caption.
pub(crate) fn not_a_caption(item: ItemId, kind: &str) -> AppError {
    AppError::InvalidEdit {
        reason: format!("item {item} is a {kind}, not a caption; give caption ids"),
    }
}

/// Every caption of `project` using each style.
pub(crate) fn style_users(project: &Project) -> BTreeMap<CaptionStyleId, BTreeSet<ItemId>> {
    let mut users: BTreeMap<CaptionStyleId, BTreeSet<ItemId>> = BTreeMap::new();
    for item in project.items() {
        if let ItemKind::Caption(caption) = &item.kind {
            users.entry(caption.style_id).or_default().insert(item.id);
        }
    }
    users
}

/// Changes the look of `items` by `patch`: a style whose users are all listed changes in place;
/// a style also used by other captions is copied (with the patch) for the listed ones. Returns
/// the styles the captions use afterwards.
///
/// # Errors
/// See [`caption_styles_of`]; an empty patch; id generator failures.
pub(crate) fn patch_styles(
    plan: &mut EditPlan<'_>,
    items: &[ItemId],
    patch: &CaptionStylePatch,
) -> Result<Vec<CaptionStyleId>, AppError> {
    if patch.is_empty() {
        return Err(AppError::InvalidEdit {
            reason: "the style change is empty: set at least one field".to_owned(),
        });
    }
    let project = plan.project()?.clone();
    let users = style_users(&project);
    let mut chosen: BTreeMap<CaptionStyleId, Vec<ItemId>> = BTreeMap::new();
    for (item, style) in caption_styles_of(&project, items)? {
        chosen.entry(style).or_default().push(item);
    }
    let mut result = Vec::new();
    for (style_id, listed) in chosen {
        let shared = users
            .get(&style_id)
            .is_some_and(|all| all.len() > listed.len());
        if shared {
            let mut copy = find_style(&project, style_id)?.clone();
            copy.id = plan.ids().caption_style()?;
            let _undo = patch.apply(&mut copy);
            let copy_id = copy.id;
            plan.push(AddCaptionStyle {
                index: None,
                style: copy,
            });
            restyle(plan, &listed, copy_id);
            result.push(copy_id);
        } else {
            plan.push(UpdateCaptionStyle {
                style: style_id,
                patch: patch.clone(),
            });
            result.push(style_id);
        }
    }
    plan.project()?;
    Ok(result)
}

/// Gives `items` a new style made from `preset`; styles left unused are removed.
///
/// # Errors
/// See [`caption_styles_of`]; an unknown preset.
pub(crate) fn apply_preset(
    plan: &mut EditPlan<'_>,
    items: &[ItemId],
    preset: &str,
) -> Result<CaptionStyleId, AppError> {
    let before = plan.project()?.clone();
    let listed: Vec<ItemId> = caption_styles_of(&before, items)?
        .into_iter()
        .map(|(item, _)| item)
        .collect();
    let format = match listed.first() {
        Some(&first) => find_item(&before, first)?.sequence.format,
        None => before.settings.default_format,
    };
    let id = new_from_preset(plan, preset, &format)?.commit(plan);
    restyle(plan, &listed, id);
    drop_unused_styles(plan, &before)?;
    Ok(id)
}

/// Points `items` at `style`.
pub(crate) fn restyle(plan: &mut EditPlan<'_>, items: &[ItemId], style: CaptionStyleId) {
    for &item in items {
        plan.push(UpdateItem {
            item,
            patch: ItemPatch::Caption(CaptionPatch {
                words: None,
                style_id: Some(style),
            }),
            expected_revision: None,
        });
    }
}

/// Removes the styles that captions of `before` used and none uses in the draft any more.
///
/// # Errors
/// The domain refuses a queued operation.
pub(crate) fn drop_unused_styles(
    plan: &mut EditPlan<'_>,
    before: &Project,
) -> Result<(), AppError> {
    let used_before = style_users(before);
    let used_after = style_users(plan.project()?);
    for style in used_before.keys() {
        if !used_after.contains_key(style) {
            plan.push(RemoveCaptionStyle { style: *style });
        }
    }
    Ok(())
}

fn find_style(project: &Project, id: CaptionStyleId) -> Result<&CaptionStyle, AppError> {
    project
        .caption_styles
        .iter()
        .find(|style| style.id == id)
        .ok_or(AppError::Domain(DomainError::CaptionStyleNotFound {
            style: id,
        }))
}
