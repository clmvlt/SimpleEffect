//! [`AgentPlanner`]: the domain operations of one agent call, planned edit by edit against a
//! draft of the document (an [`EditPlan`]), so each edit sees the previous ones and its refusal
//! names it.

use se_domain::{
    AddMarker, ApplyAnimationPreset, Flicks, ItemId, ItemKind, Marker, Operation, Origin, Project,
    SetKeyframes, TemplateItem, Transform2D, UpdateItem, animation_presets,
};

use super::{
    AgentEdit, ItemRef, LibraryInstance, NewAgentItem, PresetSpec,
    graphics::plan_new_item,
    templates::{TemplateChange, plan_new_template, plan_template_version},
    timing::{plan_delete, plan_gap, plan_move, plan_ripple_delete, plan_split, plan_trim},
    transitions::plan_transition,
};
use crate::{
    AppError,
    agent_access::AgentContext,
    captions::{
        edits as caption_edits,
        generate::{CaptionPlacement, plan_generation},
        styles as caption_styles,
    },
    edit_plan::EditPlan,
    new_ids::NewIds,
    template_library::{instance_values, plan_copy},
};

/// Why planning failed: the error, and the edit (0-based) that caused it when known.
#[derive(Debug, Clone, PartialEq)]
pub struct EditFailure {
    /// Index of the failing edit in the call; `None` when the call as a whole was refused.
    pub edit: Option<usize>,
    /// What went wrong.
    pub error: AppError,
}

impl EditFailure {
    /// A failure of the call as a whole.
    #[must_use]
    pub const fn whole(error: AppError) -> Self {
        Self { edit: None, error }
    }
}

impl From<AppError> for EditFailure {
    fn from(error: AppError) -> Self {
        Self::whole(error)
    }
}

/// The planned operations of a call.
pub(super) struct PlannedOps {
    /// The operations, in order (one transaction).
    pub(super) ops: Vec<Operation>,
    /// For each edit, the items it created.
    pub(super) created: Vec<Vec<ItemId>>,
    /// Whether an edit removes or opens time.
    pub(super) shifts_time: bool,
    /// Templates created or given a new version, in edit order.
    pub(super) templates: Vec<TemplateChange>,
}

/// Plans the edits of one agent call.
pub(super) struct AgentPlanner<'a> {
    pub(super) plan: EditPlan<'a>,
    pub(super) agent: &'a AgentContext,
    created: Vec<Vec<ItemId>>,
    shifts_time: bool,
    templates: Vec<TemplateChange>,
}

impl<'a> AgentPlanner<'a> {
    /// A planner over a copy of `project` for the run `agent`.
    pub(super) fn new(project: &Project, ids: NewIds<'a>, agent: &'a AgentContext) -> Self {
        Self {
            plan: EditPlan::new(project, ids),
            agent,
            created: Vec::new(),
            shifts_time: false,
            templates: Vec::new(),
        }
    }

    /// Plans every edit, in order.
    ///
    /// # Errors
    /// The first refused edit, with its index.
    pub(super) fn plan_all(mut self, edits: &[AgentEdit]) -> Result<PlannedOps, EditFailure> {
        for (index, edit) in edits.iter().enumerate() {
            let created = self.add(edit).map_err(|error| EditFailure {
                edit: Some(index),
                error,
            })?;
            self.created.push(created);
        }
        let ops = self.plan.finish()?;
        Ok(PlannedOps {
            ops,
            created: self.created,
            shifts_time: self.shifts_time,
            templates: self.templates,
        })
    }

    /// Plans the project's copy of a library template and its new instance.
    fn add_library_instance(&mut self, instance: &LibraryInstance) -> Result<ItemId, AppError> {
        let origin = Origin::AiTask {
            task_id: self.agent.task_id,
        };
        let template = plan_copy(&mut self.plan, &instance.entry, origin)?;
        let params = instance_values(self.plan.project()?, &template, instance.params.clone());
        let new = NewAgentItem {
            kind: ItemKind::MotionTemplate(TemplateItem {
                template_id: template.id,
                params,
                transform: Transform2D::default(),
            }),
            range: instance.range,
            track: instance.track,
            presets: instance.presets.clone(),
        };
        plan_new_item(self, &new)
    }

    /// Plans one edit and applies it to the draft (so its refusal surfaces now); returns the
    /// items it created.
    fn add(&mut self, edit: &AgentEdit) -> Result<Vec<ItemId>, AppError> {
        let created = match edit {
            AgentEdit::AddItem(new) => vec![plan_new_item(self, new)?],
            AgentEdit::UpdateItem { .. }
            | AgentEdit::SetProperty { .. }
            | AgentEdit::ApplyPreset { .. } => {
                self.add_content_change(edit)?;
                Vec::new()
            }
            AgentEdit::DeleteItems { items } => {
                plan_delete(self, items)?;
                Vec::new()
            }
            AgentEdit::MoveItem {
                item,
                track,
                start,
                expected_revision,
            } => {
                plan_move(self, *item, *track, *start, *expected_revision)?;
                Vec::new()
            }
            AgentEdit::TrimItem {
                item,
                edge,
                to,
                expected_revision,
            } => {
                plan_trim(self, *item, *edge, *to, *expected_revision)?;
                Vec::new()
            }
            AgentEdit::Split { items, at } => plan_split(self, items.as_deref(), *at)?,
            AgentEdit::RippleDelete { range, tracks } => {
                self.shifts_time = true;
                plan_ripple_delete(self, *range, tracks.as_deref())?;
                Vec::new()
            }
            AgentEdit::InsertGap {
                at,
                duration,
                tracks,
            } => {
                self.shifts_time = true;
                plan_gap(self, *at, *duration, tracks.as_deref())?;
                Vec::new()
            }
            AgentEdit::AddTransition {
                left,
                right,
                duration,
                alignment,
            } => {
                plan_transition(self, *left, *right, *duration, *alignment)?;
                Vec::new()
            }
            AgentEdit::AddCaptions(_)
            | AgentEdit::EditCaptionText { .. }
            | AgentEdit::SetCaptionEmphasis { .. }
            | AgentEdit::UpdateCaptionStyle { .. }
            | AgentEdit::ApplyCaptionPreset { .. } => self.add_caption_edit(edit)?,
            AgentEdit::CreateTemplate { definition, source } => {
                let change = plan_new_template(self, definition, source)?;
                self.templates.push(change);
                Vec::new()
            }
            AgentEdit::UpdateTemplate {
                template,
                definition,
                source,
            } => {
                let change = plan_template_version(self, *template, definition, source)?;
                self.templates.push(change);
                Vec::new()
            }
            AgentEdit::InsertLibraryTemplate(instance) => {
                vec![self.add_library_instance(instance)?]
            }
            AgentEdit::AddMarker { time, label, color } => {
                let marker = Marker {
                    id: self.plan.ids().marker()?,
                    time: *time,
                    label: label.clone(),
                    color: *color,
                };
                self.plan.push(AddMarker {
                    sequence: self.agent.sequence_id(),
                    index: None,
                    marker,
                });
                Vec::new()
            }
        };
        self.plan.project()?;
        Ok(created)
    }

    /// Plans a caption edit; returns the captions it created.
    fn add_caption_edit(&mut self, edit: &AgentEdit) -> Result<Vec<ItemId>, AppError> {
        match edit {
            AgentEdit::AddCaptions(new) => {
                let placement = CaptionPlacement {
                    sequence_id: self.agent.sequence_id(),
                    range: new.range,
                    style: new.style.as_ref(),
                    track_name: None,
                    origin: Origin::AiTask {
                        task_id: self.agent.task_id,
                    },
                };
                let placed = plan_generation(
                    &mut self.plan,
                    &placement,
                    &new.words,
                    new.segmentation.as_ref(),
                )?;
                return Ok(placed.item_ids);
            }
            AgentEdit::EditCaptionText { item, text } => {
                let item = self.resolve(*item)?;
                caption_edits::plan_text(&mut self.plan, item, text)?;
            }
            AgentEdit::SetCaptionEmphasis {
                item,
                words,
                emphasis,
            } => {
                let item = self.resolve(*item)?;
                caption_edits::plan_emphasis(&mut self.plan, item, words, *emphasis)?;
            }
            AgentEdit::UpdateCaptionStyle { items, patch } => {
                let items = self.resolve_all(items)?;
                caption_styles::patch_styles(&mut self.plan, &items, patch)?;
            }
            AgentEdit::ApplyCaptionPreset { items, preset } => {
                let items = self.resolve_all(items)?;
                caption_styles::apply_preset(&mut self.plan, &items, preset)?;
            }
            _ => {}
        }
        Ok(Vec::new())
    }

    /// The ids of several references.
    fn resolve_all(&self, items: &[ItemRef]) -> Result<Vec<ItemId>, AppError> {
        items.iter().map(|item| self.resolve(*item)).collect()
    }

    /// Plans a change of an item's content: a patch, a property or a preset.
    fn add_content_change(&mut self, edit: &AgentEdit) -> Result<(), AppError> {
        match edit {
            AgentEdit::UpdateItem {
                item,
                patch,
                expected_revision,
            } => {
                let item = self.resolve(*item)?;
                self.plan.push(UpdateItem {
                    item,
                    patch: patch.clone(),
                    expected_revision: *expected_revision,
                });
            }
            AgentEdit::SetProperty {
                item,
                animation,
                expected_revision,
            } => {
                let item = self.resolve(*item)?;
                self.plan.push(SetKeyframes {
                    item,
                    animation: animation.clone(),
                    expected_revision: *expected_revision,
                });
            }
            AgentEdit::ApplyPreset {
                item,
                preset,
                start,
                expected_revision,
            } => {
                let item = self.resolve(*item)?;
                self.push_preset(item, preset, *start, *expected_revision)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// The id an [`ItemRef`] stands for.
    ///
    /// # Errors
    /// [`AppError::InvalidEdit`] for a reference to an edit that comes later or created nothing.
    pub(super) fn resolve(&self, item: ItemRef) -> Result<ItemId, AppError> {
        match item {
            ItemRef::Id(id) => Ok(id),
            ItemRef::Created { edit } => self
                .created
                .get(edit)
                .and_then(|created| created.first().copied())
                .ok_or_else(|| AppError::InvalidEdit {
                    reason: format!(
                        "action {} creates no item before this one: refer only to earlier \
                         actions that add or split items",
                        edit + 1
                    ),
                }),
        }
    }

    /// Queues an animation preset on `item` (its default length when none is given).
    pub(super) fn push_preset(
        &mut self,
        item: ItemId,
        preset: &PresetSpec,
        start: Option<Flicks>,
        expected_revision: Option<u64>,
    ) -> Result<(), AppError> {
        let duration = match preset.duration {
            Some(duration) => duration,
            None => default_duration(&preset.preset)?,
        };
        self.plan.push(ApplyAnimationPreset {
            item,
            preset: preset.preset.clone(),
            phase: preset.phase,
            duration,
            start,
            expected_revision,
        });
        Ok(())
    }
}

/// The default length of preset `id` (an unknown preset is refused later by the domain, with
/// the list of presets: its length does not matter then).
fn default_duration(id: &str) -> Result<Flicks, AppError> {
    let library = animation_presets()?;
    Ok(library.find(id).map_or_else(
        Flicks::one_second,
        se_domain::AnimationPreset::default_duration,
    ))
}
