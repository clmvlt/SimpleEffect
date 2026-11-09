//! Pure planning of library templates in a project (prompt 25): the project's copy of a library
//! entry, the instance values a new instance starts with, and the update of a copy to a newer
//! library version. Shared by the user's commands and AI agents.

use std::collections::BTreeMap;

use se_domain::{
    ItemId, ItemKind, ItemPatch, LibraryLink, MotionTemplate, Origin, Project, ReplaceTemplate,
    TemplateId, TemplateParamValue, TemplatePatch, UpdateItem, brand_values, migrate_params,
};
use se_protocol::{LibraryContent, LibraryTemplate, ResetValue, TemplateUpdateReport};

use crate::{AppError, edit_plan::EditPlan};

/// The motion template a library entry describes, as a new project copy `id` linked to it.
///
/// # Errors
/// [`AppError::InvalidEdit`] for a caption style entry or an unusable template (a damaged
/// library file).
pub fn project_copy(
    entry: &LibraryTemplate,
    id: TemplateId,
    version: u32,
    origin: Origin,
) -> Result<MotionTemplate, AppError> {
    let LibraryContent::MotionTemplate {
        category,
        source,
        params,
        default_duration,
    } = &entry.content
    else {
        return Err(AppError::InvalidEdit {
            reason: format!(
                "library entry « {} » is a caption style, not a motion template: apply it to \
                 captions instead",
                entry.name
            ),
        });
    };
    let template = MotionTemplate {
        id,
        name: entry.name.clone(),
        description: entry.description.clone(),
        version,
        source: source.clone(),
        params: params.clone(),
        default_duration: *default_duration,
        origin,
        category: *category,
        library: Some(LibraryLink {
            template_id: entry.id,
            version: entry.version,
        }),
    };
    let problems = template.problems();
    if problems.is_empty() {
        Ok(template)
    } else {
        Err(AppError::InvalidEdit {
            reason: format!(
                "library entry « {} » is damaged ({}): import it again or save it from a project",
                entry.name,
                problems
                    .into_iter()
                    .map(|(field, rule)| format!("{field} {rule}"))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        })
    }
}

/// A copy of `entry` already in `project` (same entry, version and source), if any.
#[must_use]
pub fn existing_copy(project: &Project, entry: &LibraryTemplate) -> Option<TemplateId> {
    let LibraryContent::MotionTemplate { source, .. } = &entry.content else {
        return None;
    };
    project
        .templates
        .iter()
        .find(|template| {
            template.library
                == Some(LibraryLink {
                    template_id: entry.id,
                    version: entry.version,
                })
                && template.source == *source
        })
        .map(|template| template.id)
}

/// Queues the project's copy of `entry` in `plan` (an `AddTemplate`), or reuses an existing
/// copy; returns the project template to instantiate.
///
/// # Errors
/// See [`project_copy`]; the domain's refusals.
pub(crate) fn plan_copy(
    plan: &mut EditPlan<'_>,
    entry: &LibraryTemplate,
    origin: Origin,
) -> Result<MotionTemplate, AppError> {
    if let Some(id) = existing_copy(plan.project()?, entry)
        && let Some(template) = plan
            .project()?
            .templates
            .iter()
            .find(|template| template.id == id)
    {
        return Ok(template.clone());
    }
    let template = project_copy(entry, plan.ids().template()?, 1, origin)?;
    plan.push(se_domain::AddTemplate {
        index: None,
        template: template.clone(),
    });
    Ok(template)
}

/// The values a new instance of `template` starts with in `project`: the brand kit's values for
/// parameters bound to a role, then `explicit` values on top.
#[must_use]
pub fn instance_values(
    project: &Project,
    template: &MotionTemplate,
    explicit: BTreeMap<String, TemplateParamValue>,
) -> BTreeMap<String, TemplateParamValue> {
    let mut values = brand_values(template, project.brand_kit.as_ref());
    values.extend(explicit);
    values
}

/// Queues the update of project template `template_id` to the newer `entry` (its library
/// source): a `ReplaceTemplate` with the next project version, then the instances' values the
/// new parameters refuse reset to their defaults.
///
/// # Errors
/// [`AppError::InvalidEdit`] when the template is not a copy of `entry` or already holds its
/// version; see [`project_copy`].
pub(crate) fn plan_update(
    plan: &mut EditPlan<'_>,
    template_id: TemplateId,
    entry: &LibraryTemplate,
) -> Result<TemplateUpdateReport, AppError> {
    let project = plan.project()?;
    let current = project
        .templates
        .iter()
        .find(|template| template.id == template_id)
        .ok_or(se_domain::DomainError::TemplateNotFound {
            template: template_id,
        })?
        .clone();
    let link = current
        .library
        .filter(|link| link.template_id == entry.id)
        .ok_or_else(|| AppError::InvalidEdit {
            reason: format!(
                "template « {} » was not copied from library entry « {} »",
                current.name, entry.name
            ),
        })?;
    if !link.is_outdated_by(entry.version) {
        return Err(AppError::InvalidEdit {
            reason: format!(
                "template « {} » already holds version {} of the library's « {} »",
                current.name, link.version, entry.name
            ),
        });
    }
    let version = current.version.saturating_add(1);
    let updated = project_copy(entry, template_id, version, current.origin)?;
    let instances = instances_of(project, template_id);
    plan.push(ReplaceTemplate {
        template: updated.clone(),
    });
    let mut reset = Vec::new();
    for (item, params) in instances {
        let migration = migrate_params(&params, &updated);
        if migration.is_lossless() {
            continue;
        }
        reset.extend(migration.reset.iter().map(|param| ResetValue {
            item_id: item,
            param: param.clone(),
        }));
        plan.push(UpdateItem {
            item,
            patch: ItemPatch::MotionTemplate(TemplatePatch {
                template_id: None,
                params: migration
                    .reset
                    .into_iter()
                    .map(|name| (name, None))
                    .collect(),
            }),
            expected_revision: None,
        });
    }
    Ok(TemplateUpdateReport {
        version,
        library_version: entry.version,
        reset,
    })
}

/// Every instance of `template_id` with its values, in id order.
fn instances_of(
    project: &Project,
    template_id: TemplateId,
) -> Vec<(ItemId, BTreeMap<String, TemplateParamValue>)> {
    let mut instances: Vec<_> = project
        .items()
        .filter_map(|item| match &item.kind {
            ItemKind::MotionTemplate(instance) if instance.template_id == template_id => {
                Some((item.id, instance.params.clone()))
            }
            _ => None,
        })
        .collect();
    instances.sort_by_key(|(item, _)| *item);
    instances
}

#[cfg(test)]
mod tests {
    use se_domain::{
        Animatable, Color, LibraryTemplateId, TemplateParamSpec,
        test_support::{LIBRARY_LOWER_THIRD, example_ids, example_project},
    };

    use super::*;
    use crate::test_support::SequentialIds;

    fn entry_of(template: &MotionTemplate, version: u32) -> LibraryTemplate {
        LibraryTemplate {
            id: LIBRARY_LOWER_THIRD,
            version,
            name: template.name.clone(),
            description: template.description.clone(),
            tags: Vec::new(),
            updated_at: 0,
            content: LibraryContent::MotionTemplate {
                category: template.category,
                source: template.source.clone(),
                params: template.params.clone(),
                default_duration: template.default_duration,
            },
        }
    }

    #[test]
    fn the_same_version_is_reused_and_a_newer_one_copied() {
        let project = example_project();
        let template = &project.templates[0];
        assert_eq!(
            existing_copy(&project, &entry_of(template, 1)),
            Some(example_ids::TEMPLATE)
        );
        assert_eq!(existing_copy(&project, &entry_of(template, 2)), None);
        let ids = SequentialIds::default();
        let mut plan = EditPlan::new(&project, crate::new_ids::NewIds(&ids));
        let copy = plan_copy(&mut plan, &entry_of(template, 2), Origin::User).unwrap();
        assert_ne!(copy.id, example_ids::TEMPLATE);
        assert_eq!(copy.version, 1);
        assert_eq!(copy.library.unwrap().version, 2);
        assert_eq!(plan.project().unwrap().templates.len(), 2);
    }

    #[test]
    fn caption_styles_are_not_motion_templates() {
        let style = example_project().caption_styles[0].clone();
        let entry = LibraryTemplate {
            id: LibraryTemplateId::from_test_index(9),
            version: 1,
            name: "Style".to_owned(),
            description: String::new(),
            tags: Vec::new(),
            updated_at: 0,
            content: LibraryContent::CaptionStyle { style },
        };
        let error = project_copy(&entry, example_ids::TEMPLATE, 1, Origin::User).unwrap_err();
        assert!(error.to_string().contains("caption style"), "{error}");
    }

    #[test]
    fn instance_values_start_from_the_brand_kit() {
        let project = example_project();
        let template = &project.templates[0];
        let values = instance_values(&project, template, BTreeMap::new());
        let accent = project.brand_kit.as_ref().unwrap().colors.accent;
        assert_eq!(
            values.get("accent"),
            Some(&TemplateParamValue::Color {
                value: Animatable::constant(accent)
            })
        );
        let explicit = BTreeMap::from([(
            "accent".to_owned(),
            TemplateParamValue::Color {
                value: Animatable::constant(Color::WHITE),
            },
        )]);
        assert_eq!(
            instance_values(&project, template, explicit)["accent"],
            TemplateParamValue::Color {
                value: Animatable::constant(Color::WHITE)
            }
        );
    }

    #[test]
    fn updates_migrate_values_and_refuse_current_copies() {
        let project = example_project();
        let mut newer = project.templates[0].clone();
        // The new version narrows « slide »: the instance's keyframed 200 no longer fits.
        for param in &mut newer.params {
            if param.name == "slide" {
                param.spec = TemplateParamSpec::Number {
                    default: 50.0,
                    minimum: Some(0.0),
                    maximum: Some(100.0),
                    step: None,
                };
            }
        }
        let ids = SequentialIds::default();
        let mut plan = EditPlan::new(&project, crate::new_ids::NewIds(&ids));
        let report = plan_update(&mut plan, example_ids::TEMPLATE, &entry_of(&newer, 3)).unwrap();
        assert_eq!(report.version, 2);
        assert_eq!(report.library_version, 3);
        assert_eq!(report.reset.len(), 1);
        assert_eq!(report.reset[0].param, "slide");
        let updated = plan.project().unwrap().templates[0].clone();
        assert_eq!(updated.library.unwrap().version, 3);

        let mut plan = EditPlan::new(&project, crate::new_ids::NewIds(&ids));
        let error = plan_update(
            &mut plan,
            example_ids::TEMPLATE,
            &entry_of(&project.templates[0], 1),
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("already holds version 1"),
            "{error}"
        );
    }
}
