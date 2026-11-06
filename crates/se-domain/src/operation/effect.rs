//! [`Edit`], implemented by every operation payload, and the [`Effect`] it records.

use super::{Operation, TouchedEntities};
use crate::{DomainError, Project};

/// The edit of one operation payload, applied in place to a draft project.
///
/// Implementations check their preconditions, mutate `project` and record in `effect` the
/// operations that undo the edit and the entities they touched. They do not validate the whole
/// document ([`super::apply`] does) and must not change revisions. On error the draft is
/// discarded by the caller.
pub(crate) trait Edit {
    /// Applies the edit to `project`.
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError>;
}

/// What an edit did: how to undo it and which entities it touched.
#[derive(Debug, Default)]
pub(crate) struct Effect {
    /// Operations undoing the edit, in application order.
    pub(crate) inverse: Vec<Operation>,
    /// Entities added, changed or removed.
    pub(crate) touched: TouchedEntities,
}

impl Effect {
    /// Records an operation of the inverse. An edit recording several must record them in the
    /// order that undoes it (or run its changes as steps with [`run_steps`]).
    pub(crate) fn undo_with(&mut self, operation: impl Into<Operation>) {
        self.inverse.push(operation.into());
    }
}

/// Applies `steps` one after the other, then records their inverses last step first, so that
/// the inverse goes back through the same intermediate states.
///
/// Composite edits (split, ripple delete, insert gap) are built from primitive steps so that
/// their inverse is exact by construction. Each intermediate state must be valid on its own,
/// because the inverse operations are applied (and validated) one by one.
pub(crate) fn run_steps(
    project: &mut Project,
    steps: &[Operation],
    effect: &mut Effect,
) -> Result<(), DomainError> {
    let mut inverses = Vec::with_capacity(steps.len());
    for step in steps {
        let mut step_effect = Effect::default();
        step.as_edit().apply_to(project, &mut step_effect)?;
        effect.touched.merge(step_effect.touched);
        inverses.push(step_effect.inverse);
    }
    effect.inverse.extend(inverses.into_iter().rev().flatten());
    Ok(())
}
