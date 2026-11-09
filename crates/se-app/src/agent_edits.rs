//! [`AgentEditService`]: the only way AI agents change a project (ADR-0003, ADR-0029; golden
//! rules 2 and 4). MCP write tools hand it typed [`AgentEdit`]s; it plans the domain operations
//! against the current document ([`planner`]), checks that everything they touch lies in the
//! task's scope before and after ([`scope`]), and commits them as **one** transaction of actor
//! `AiTask(task)` through the project's actor, which records it in the task's changeset.
//!
//! - `request`: what agents ask for and what they get back.
//! - `planner`: one call's edits planned in order on a draft (later edits see earlier ones;
//!   `apply_batch` references resolve to items created earlier in the call).
//! - `graphics`, `timing`, `transitions`: the planners of each kind of edit. Moves, trims,
//!   splits and deletes keep linked items together, as in the timeline.
//! - `scope`: the scope check and its messages.
//! - `templates`: motion templates written by agents (prompt 24): new ones and new versions with
//!   copy on write.
//! - `cuts`, `cut_scope`: smart cuts of agents (prompt 22) and their « cut » scope rule.

mod cut_scope;
mod cuts;
mod graphics;
mod planner;
mod request;
mod scope;
mod templates;
mod timing;
mod transitions;

pub use cuts::CutOutcome;
pub use planner::EditFailure;
pub use request::{
    AgentEdit, AgentEditReport, ItemRef, NewAgentCaptions, NewAgentItem, PresetSpec, TaskChanges,
};
pub use templates::TemplateChange;

use std::{collections::BTreeSet, sync::Arc};

use se_domain::{Actor, ItemId, Operation, Transaction, apply_all};

use self::{
    planner::AgentPlanner,
    scope::{PlannedChange, check_scope},
};
use crate::{
    AppError, ProjectService, TaskService, agent_access::AgentContext, new_ids::NewIds,
    ports::IdGenerator, project_actor::AiCommit,
};

/// How many times a call is planned again when the project changed while it was planned.
const MAX_ATTEMPTS: usize = 4;

/// AI edits of open projects. Cheap to share (`Arc`).
pub struct AgentEditService {
    projects: Arc<ProjectService>,
    tasks: TaskService,
    ids: Arc<dyn IdGenerator>,
}

impl AgentEditService {
    /// A service committing through `projects`' actors and recording changesets on `tasks`.
    #[must_use]
    pub fn new(
        projects: Arc<ProjectService>,
        tasks: TaskService,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            projects,
            tasks,
            ids,
        }
    }

    /// Plans `edits` for `agent`'s task, checks its scope and commits them as one transaction
    /// labelled `label` (the tool's name): all of them, or none.
    ///
    /// # Errors
    /// The first refused edit (with its index), [`AppError::OutOfScope`], the domain's refusal,
    /// [`AppError::ProjectNotOpen`], storage errors.
    pub async fn apply(
        &self,
        agent: &AgentContext,
        label: &str,
        edits: &[AgentEdit],
    ) -> Result<AgentEditReport, EditFailure> {
        if edits.is_empty() {
            return Err(EditFailure::whole(AppError::InvalidEdit {
                reason: "nothing to do: give at least one action".to_owned(),
            }));
        }
        let handle = self.projects.handle(agent.project_id).await?;
        for _ in 0..MAX_ATTEMPTS {
            let view = handle.view().await?;
            let base = &view.project;
            let planned =
                AgentPlanner::new(base, NewIds(self.ids.as_ref()), agent).plan_all(edits)?;
            if planned.ops.is_empty() {
                return Err(EditFailure::whole(AppError::InvalidEdit {
                    reason: "these actions change nothing".to_owned(),
                }));
            }
            let applied = apply_all(base, &planned.ops).map_err(AppError::from)?;
            check_scope(
                &agent.scope,
                &PlannedChange {
                    before: base,
                    after: &applied.project,
                    touched: &applied.touched,
                    shifts_time: planned.shifts_time,
                },
            )?;
            let transaction = self.transaction(agent, label, planned.ops)?;
            let Some(commit) = handle.apply_ai_on(transaction, base.revision).await? else {
                // The user (or another task) committed meanwhile: plan again on the new state.
                continue;
            };
            self.note_changeset(agent, &commit).await;
            return Ok(report(commit, planned.created, planned.templates));
        }
        Err(EditFailure::whole(AppError::InvalidEdit {
            reason: "the project kept changing while these actions were applied: read it again \
                     and retry"
                .to_owned(),
        }))
    }

    /// What `agent`'s task changed so far (its open changeset, summarised).
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`].
    pub async fn changes(&self, agent: &AgentContext) -> Result<TaskChanges, AppError> {
        let handle = self.projects.handle(agent.project_id).await?;
        let changeset = handle.changeset(agent.task_id).await?;
        let project = handle.view().await?.project;
        let Some(changeset) = changeset else {
            return Ok(TaskChanges {
                changeset_id: None,
                edits: 0,
                created: Vec::new(),
                changed: Vec::new(),
                removed: Vec::new(),
                project,
            });
        };
        let added: BTreeSet<ItemId> = changeset
            .entries
            .iter()
            .flat_map(|entry| &entry.inverse.ops)
            .filter_map(|operation| match operation {
                Operation::RemoveItem(removal) => Some(removal.item),
                _ => None,
            })
            .collect();
        let touched: BTreeSet<ItemId> = changeset
            .entries
            .iter()
            .flat_map(|entry| entry.touched_revisions.keys().copied())
            .collect();
        let present: BTreeSet<ItemId> = project.items().map(|item| item.id).collect();
        let (mut created, mut changed, mut removed) = (Vec::new(), Vec::new(), Vec::new());
        for id in touched {
            match (present.contains(&id), added.contains(&id)) {
                (true, true) => created.push(id),
                (true, false) => changed.push(id),
                (false, _) => removed.push(id),
            }
        }
        Ok(TaskChanges {
            changeset_id: Some(changeset.id),
            edits: changeset.entries.len(),
            created,
            changed,
            removed,
            project,
        })
    }

    fn transaction(
        &self,
        agent: &AgentContext,
        label: &str,
        ops: Vec<Operation>,
    ) -> Result<Transaction, AppError> {
        Ok(Transaction {
            id: NewIds(self.ids.as_ref()).transaction()?,
            actor: Actor::AiTask {
                task_id: agent.task_id,
            },
            label: label.to_owned(),
            ops,
        })
    }

    /// Records a new changeset on its task (`taskUpdated`); a failure there is logged only: the
    /// edit is committed and its changeset saved with it.
    async fn note_changeset(&self, agent: &AgentContext, commit: &AiCommit) {
        if !commit.changeset_created {
            return;
        }
        if let Err(error) = self
            .tasks
            .attach_changeset(agent.project_id, agent.task_id, commit.changeset_id)
            .await
        {
            tracing::warn!(%error, task = %agent.task_id, "recording a task's changeset failed");
        }
    }
}

/// What a committed call reports.
fn report(
    commit: AiCommit,
    created: Vec<Vec<ItemId>>,
    templates: Vec<TemplateChange>,
) -> AgentEditReport {
    let present: BTreeSet<ItemId> = commit.project.items().map(|item| item.id).collect();
    let (changed, removed) = commit
        .touched_items
        .iter()
        .partition(|item| present.contains(item));
    AgentEditReport {
        revision: commit.revision,
        created,
        changed,
        removed,
        templates,
        project: commit.project,
    }
}
