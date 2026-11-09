//! Motion template operations (prompt 24, ADR-0035): [`AddTemplate`], [`ReplaceTemplate`],
//! [`RemoveTemplate`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Edit, Effect,
    checks::{ensure_unused, insertion_index},
    lookup::all_items,
};
use crate::{DomainError, ItemKind, MotionTemplate, Project, TemplateId};

/// Index of the template `id`.
fn find_template(project: &Project, id: TemplateId) -> Result<usize, DomainError> {
    project
        .templates
        .iter()
        .position(|template| template.id == id)
        .ok_or(DomainError::TemplateNotFound { template: id })
}

/// Adds a motion template. Inverse: [`RemoveTemplate`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AddTemplate {
    /// Position in the project's templates (`null` appends).
    pub index: Option<usize>,
    /// The template, with an unused id.
    pub template: MotionTemplate,
}

impl Edit for AddTemplate {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let id = self.template.id;
        ensure_unused(find_template(project, id).is_ok(), "template", &id)?;
        let index = insertion_index("template", self.index, project.templates.len())?;
        project.templates.insert(index, self.template.clone());
        effect.touched.templates.insert(id);
        effect.undo_with(RemoveTemplate { template: id });
        Ok(())
    }
}

/// Replaces a template's definition (a new version of its source; every item using it renders
/// the new code). Items whose values no longer fit must be updated in the same transaction
/// (validation runs at its end). Inverse: [`ReplaceTemplate`] with the previous definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceTemplate {
    /// The new definition; its id names the template replaced.
    pub template: MotionTemplate,
}

impl Edit for ReplaceTemplate {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let id = self.template.id;
        let index = find_template(project, id)?;
        let previous = std::mem::replace(&mut project.templates[index], self.template.clone());
        effect.touched.templates.insert(id);
        effect.undo_with(Self { template: previous });
        Ok(())
    }
}

/// Removes a template no item uses. Inverse: [`AddTemplate`] at the same position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RemoveTemplate {
    /// The template removed.
    pub template: TemplateId,
}

impl Edit for RemoveTemplate {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let index = find_template(project, self.template)?;
        let user = all_items(project).find(|item| {
            matches!(&item.kind, ItemKind::MotionTemplate(instance) if instance.template_id == self.template)
        });
        if let Some(user) = user {
            return Err(DomainError::TemplateInUse {
                template: self.template,
                item: user.id,
            });
        }
        let template = project.templates.remove(index);
        effect.touched.templates.insert(self.template);
        effect.undo_with(AddTemplate {
            index: Some(index),
            template,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DomainErrorCode,
        operation::testing::{apply_err, apply_ok, example},
        test_support::example_ids,
    };

    fn other_template(index: u64) -> MotionTemplate {
        MotionTemplate {
            id: TemplateId::from_test_index(index),
            name: "Progress bar".to_owned(),
            ..example().templates[0].clone()
        }
    }

    #[test]
    fn adds_a_template_at_its_place() {
        let project = example();
        let applied = apply_ok(
            &project,
            AddTemplate {
                index: Some(0),
                template: other_template(9),
            },
        );
        assert_eq!(applied.project.templates[0].name, "Progress bar");
        assert!(
            applied
                .touched
                .templates
                .contains(&TemplateId::from_test_index(9))
        );
    }

    #[test]
    fn refuses_a_template_id_in_use() {
        let error = apply_err(
            &example(),
            AddTemplate {
                index: None,
                template: other_template(1),
            },
        );
        assert_eq!(error.code(), DomainErrorCode::IdAlreadyUsed);
    }

    #[test]
    fn replaces_a_definition_and_back() {
        let project = example();
        let mut next = project.templates[0].clone();
        next.version += 1;
        next.source.push_str("\n// v2\n");
        let applied = apply_ok(&project, ReplaceTemplate { template: next });
        assert_eq!(
            applied.project.templates[0].version,
            project.templates[0].version + 1
        );
        let unknown = apply_err(
            &project,
            ReplaceTemplate {
                template: other_template(9),
            },
        );
        assert_eq!(unknown.code(), DomainErrorCode::TemplateNotFound);
    }

    #[test]
    fn removes_only_unused_templates() {
        let project = example();
        let used = apply_err(
            &project,
            RemoveTemplate {
                template: example_ids::TEMPLATE,
            },
        );
        assert_eq!(used.code(), DomainErrorCode::TemplateInUse);
        let with_spare = apply_ok(
            &project,
            AddTemplate {
                index: None,
                template: other_template(9),
            },
        )
        .project;
        let removed = apply_ok(
            &with_spare,
            RemoveTemplate {
                template: TemplateId::from_test_index(9),
            },
        );
        assert_eq!(removed.project.templates.len(), 1);
    }
}
