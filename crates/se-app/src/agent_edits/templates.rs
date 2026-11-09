//! Motion templates written by agents (prompt 24, ADR-0035): a new template, or a new version of
//! one with copy on write — in place when every instance lies in the task's scope, otherwise a
//! copy that only the instances of the scope switch to. Values a new version refuses fall back to
//! its defaults ([`migrate_params`]).

use std::collections::BTreeMap;

use se_domain::{
    AddTemplate, DomainError, ItemId, ItemKind, ItemPatch, MotionTemplate, Origin, ReplaceTemplate,
    TemplateDefinition, TemplateId, TemplatePatch, UpdateItem, migrate_params,
};

use super::{planner::AgentPlanner, scope::item_in_scope};
use crate::{AppError, template_checks::definition_problem};

/// What a template edit did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateChange {
    /// The template now holding the code (a new id for a new template or a copy).
    pub template_id: TemplateId,
    /// Its version.
    pub version: u32,
    /// The template copied when some of its instances lie outside the task's scope (they keep
    /// it); `None` when the template was created or changed in place.
    pub copied_from: Option<TemplateId>,
    /// Instances now rendering this version, in id order.
    pub instances: Vec<ItemId>,
    /// Instances whose values fell back to defaults, with the parameter names.
    pub reset: Vec<(ItemId, Vec<String>)>,
}

/// Queues a new template made of `definition` and `source`.
///
/// # Errors
/// [`AppError::TemplateInvalid`] when the template as a whole is unusable.
pub(super) fn plan_new_template(
    planner: &mut AgentPlanner<'_>,
    definition: &TemplateDefinition,
    source: &str,
) -> Result<TemplateChange, AppError> {
    let id = planner.plan.ids().template()?;
    let template = build(planner, definition, source, id, 1)?;
    planner.plan.push(AddTemplate {
        index: None,
        template,
    });
    Ok(TemplateChange {
        template_id: id,
        version: 1,
        copied_from: None,
        instances: Vec::new(),
        reset: Vec::new(),
    })
}

/// Queues a new version of template `template_id` (see the module documentation).
///
/// # Errors
/// An unknown template, [`AppError::TemplateInvalid`].
pub(super) fn plan_template_version(
    planner: &mut AgentPlanner<'_>,
    template_id: TemplateId,
    definition: &TemplateDefinition,
    source: &str,
) -> Result<TemplateChange, AppError> {
    let project = planner.plan.project()?;
    let current = project
        .templates
        .iter()
        .find(|template| template.id == template_id)
        .ok_or(DomainError::TemplateNotFound {
            template: template_id,
        })?;
    let version = current.version.saturating_add(1);
    let instances = instances_of(planner, template_id)?;
    let in_place = instances.iter().all(|instance| instance.in_scope);
    let id = if in_place {
        template_id
    } else {
        planner.plan.ids().template()?
    };
    let template = build(planner, definition, source, id, version)?;
    let mut change = TemplateChange {
        template_id: id,
        version,
        copied_from: (!in_place).then_some(template_id),
        instances: Vec::new(),
        reset: Vec::new(),
    };
    // The template first (items switch to it), then the values it refuses: validation runs at
    // the end of the transaction.
    if in_place {
        planner.plan.push(ReplaceTemplate {
            template: template.clone(),
        });
    } else {
        planner.plan.push(AddTemplate {
            index: None,
            template: template.clone(),
        });
    }
    for instance in instances.iter().filter(|instance| instance.in_scope) {
        let migration = migrate_params(&instance.params, &template);
        change.instances.push(instance.item);
        if migration.is_lossless() && in_place {
            continue;
        }
        let params: BTreeMap<String, Option<_>> = migration
            .reset
            .iter()
            .map(|name| (name.clone(), None))
            .collect();
        if !migration.is_lossless() {
            change.reset.push((instance.item, migration.reset));
        }
        planner.plan.push(UpdateItem {
            item: instance.item,
            patch: ItemPatch::MotionTemplate(TemplatePatch {
                template_id: (!in_place).then_some(id),
                params,
            }),
            expected_revision: None,
        });
    }
    Ok(change)
}

/// An item using a template.
struct Instance {
    item: ItemId,
    params: BTreeMap<String, se_domain::TemplateParamValue>,
    in_scope: bool,
}

fn instances_of(
    planner: &mut AgentPlanner<'_>,
    template_id: TemplateId,
) -> Result<Vec<Instance>, AppError> {
    let scope = &planner.agent.scope;
    let project = planner.plan.project()?;
    let mut instances = Vec::new();
    for sequence in &project.sequences {
        for track in &sequence.tracks {
            for item in &track.items {
                if let ItemKind::MotionTemplate(instance) = &item.kind
                    && instance.template_id == template_id
                {
                    instances.push(Instance {
                        item: item.id,
                        params: instance.params.clone(),
                        in_scope: item_in_scope(scope, sequence, track, item),
                    });
                }
            }
        }
    }
    instances.sort_by_key(|instance| instance.item);
    Ok(instances)
}

fn build(
    planner: &AgentPlanner<'_>,
    definition: &TemplateDefinition,
    source: &str,
    id: TemplateId,
    version: u32,
) -> Result<MotionTemplate, AppError> {
    let origin = Origin::AiTask {
        task_id: planner.agent.task_id,
    };
    definition
        .clone()
        .into_template(id, source.to_owned(), version, origin)
        .map_err(|problems| AppError::TemplateInvalid {
            problems: problems
                .into_iter()
                .map(|problem| definition_problem(format!("{}: {}", problem.path, problem.message)))
                .collect(),
        })
}
