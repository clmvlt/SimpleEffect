//! How AI agents reach the project (ADR-0003, ADR-0028): each run of a task gets a fresh bearer
//! token for se-core's `/mcp` endpoint, bound to an [`AgentContext`] (the task, its run and its
//! scope). The [`TaskTokenRegistry`] mints the token when the run starts and revokes it when the
//! run ends; the MCP server resolves the token of every request to its context and refuses
//! unknown or revoked ones.

use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use se_domain::{ProjectId, SequenceId, TaskId};
use se_protocol::TaskScope;

use crate::ports::TokenSource;

/// Who is calling an MCP tool: one run of one task, and what it may change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentContext {
    /// The task's project.
    pub project_id: ProjectId,
    /// The task.
    pub task_id: TaskId,
    /// Index of the run in the task's runs (0 for the first, then each retry or follow-up), as
    /// in its event log.
    pub run: u32,
    /// What the task may change (writes are checked against it, prompt 19).
    pub scope: TaskScope,
}

impl AgentContext {
    /// The sequence the task works on (the default sequence of read tools).
    #[must_use]
    pub const fn sequence_id(&self) -> SequenceId {
        self.scope.sequence_id
    }
}

/// A run's bearer token. Never logged: `Debug` redacts it.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TaskToken(String);

impl TaskToken {
    /// The secret, for the agent's MCP configuration only.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for TaskToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TaskToken(<redacted>)")
    }
}

/// The tokens of running agents and the context each one stands for. Shared by the task service
/// (mint, revoke) and the MCP server (resolve).
pub struct TaskTokenRegistry {
    source: Arc<dyn TokenSource>,
    active: Mutex<HashMap<String, AgentContext>>,
}

impl fmt::Debug for TaskTokenRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TaskTokenRegistry")
            .field("active", &self.active_count())
            .finish_non_exhaustive()
    }
}

impl TaskTokenRegistry {
    /// An empty registry whose secrets come from `source`.
    #[must_use]
    pub fn new(source: Arc<dyn TokenSource>) -> Self {
        Self {
            source,
            active: Mutex::new(HashMap::new()),
        }
    }

    /// A new token standing for `context` until [`TaskTokenRegistry::revoke`].
    pub fn mint(&self, context: AgentContext) -> TaskToken {
        let mut active = self.lock();
        loop {
            let secret = self.source.new_token();
            // 256 random bits never collide in practice; a broken source must not hand one run
            // another run's access.
            if let std::collections::hash_map::Entry::Vacant(slot) = active.entry(secret.clone()) {
                slot.insert(context);
                return TaskToken(secret);
            }
        }
    }

    /// The context of a token presented by an agent; `None` when unknown or revoked.
    ///
    /// The lookup hashes the candidate with the map's random keys, so its timing tells nothing
    /// useful about the stored secrets.
    #[must_use]
    pub fn resolve(&self, candidate: &str) -> Option<AgentContext> {
        self.lock().get(candidate).cloned()
    }

    /// Ends `token`'s access (its run is over). Revoking twice does nothing more.
    pub fn revoke(&self, token: &TaskToken) {
        self.lock().remove(&token.0);
    }

    /// How many tokens are valid now.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.lock().len()
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, AgentContext>> {
        // Entries are inserted and removed whole: a panic elsewhere cannot leave one half-made.
        self.active.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// What runs need to reach the MCP server: its URL and the token registry.
#[derive(Clone)]
pub struct McpAccess {
    /// `http://127.0.0.1:<port>/mcp`.
    pub url: String,
    /// Tokens of running agents.
    pub tokens: Arc<TaskTokenRegistry>,
}

impl fmt::Debug for McpAccess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("McpAccess")
            .field("url", &self.url)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    /// Gives the same secret twice, then distinct ones.
    struct RepeatingSource(AtomicUsize);

    impl TokenSource for RepeatingSource {
        fn new_token(&self) -> String {
            let call = self.0.fetch_add(1, Ordering::SeqCst);
            format!("secret-{}", call.saturating_sub(1))
        }
    }

    fn context(task: u64) -> AgentContext {
        AgentContext {
            project_id: ProjectId::from_test_index(1),
            task_id: TaskId::from_test_index(task),
            run: 1,
            scope: TaskScope {
                sequence_id: SequenceId::from_test_index(2),
                range: None,
                tracks: None,
            },
        }
    }

    #[test]
    fn tokens_resolve_until_revoked() {
        let registry = TaskTokenRegistry::new(Arc::new(RepeatingSource(AtomicUsize::new(1))));
        let token = registry.mint(context(3));
        assert_eq!(registry.resolve(token.as_str()), Some(context(3)));
        assert_eq!(registry.resolve("guess"), None);
        registry.revoke(&token);
        assert_eq!(registry.resolve(token.as_str()), None);
        registry.revoke(&token);
        assert_eq!(registry.active_count(), 0);
    }

    #[test]
    fn a_repeated_secret_never_shares_access() {
        let registry = TaskTokenRegistry::new(Arc::new(RepeatingSource(AtomicUsize::new(0))));
        let first = registry.mint(context(3));
        let second = registry.mint(context(4));
        assert_ne!(first, second);
        assert_eq!(registry.resolve(second.as_str()), Some(context(4)));
        assert_eq!(registry.active_count(), 2);
    }

    #[test]
    fn tokens_are_redacted_in_logs() {
        let registry = TaskTokenRegistry::new(Arc::new(RepeatingSource(AtomicUsize::new(5))));
        let token = registry.mint(context(3));
        assert!(!format!("{token:?}").contains(token.as_str()));
    }
}
