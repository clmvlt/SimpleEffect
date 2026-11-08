//! [`RunQueue`]: which runs wait (first in, first out, all projects together) and which run, under
//! a limit of runs at once. Bookkeeping only: the service starts and finishes the runs.

use std::collections::{HashMap, VecDeque};

use se_domain::{ProjectId, TaskId};
use tokio::sync::{oneshot, watch};

/// A run to start: the latest run of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RunKey {
    pub project_id: ProjectId,
    pub task_id: TaskId,
}

/// A running run, as the queue sees it.
pub(super) struct Running {
    project_id: ProjectId,
    /// Set to `true` to ask the run to stop.
    cancel: watch::Sender<bool>,
    /// Resolves (with an error) once the run's task is gone.
    finished: Option<oneshot::Receiver<()>>,
}

/// Waiting and running runs.
pub(super) struct RunQueue {
    pending: VecDeque<RunKey>,
    running: HashMap<TaskId, Running>,
    limit: usize,
}

impl RunQueue {
    /// An empty queue running at most `limit` runs at once.
    pub fn new(limit: usize) -> Self {
        Self {
            pending: VecDeque::new(),
            running: HashMap::new(),
            limit: limit.max(1),
        }
    }

    /// Changes the limit (running runs above a lowered limit go on).
    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit.max(1);
    }

    /// Queues a run behind the others.
    pub fn push(&mut self, key: RunKey) {
        self.pending.push_back(key);
    }

    /// Removes a waiting run; returns whether it was waiting.
    pub fn remove_pending(&mut self, task_id: TaskId) -> bool {
        let before = self.pending.len();
        self.pending.retain(|key| key.task_id != task_id);
        self.pending.len() != before
    }

    /// The oldest waiting run, taken out of the queue, when a slot is free.
    pub fn next_to_start(&mut self) -> Option<RunKey> {
        if self.running.len() >= self.limit {
            return None;
        }
        self.pending.pop_front()
    }

    /// Records that `key` runs; returns the receiver of its cancel flag and the sender to drop
    /// once the run's task is gone.
    pub fn mark_running(&mut self, key: RunKey) -> (watch::Receiver<bool>, oneshot::Sender<()>) {
        let (cancel, cancelled) = watch::channel(false);
        let (gone, finished) = oneshot::channel();
        self.running.insert(
            key.task_id,
            Running {
                project_id: key.project_id,
                cancel,
                finished: Some(finished),
            },
        );
        (cancelled, gone)
    }

    /// Forgets a run that ended.
    pub fn finish(&mut self, task_id: TaskId) {
        self.running.remove(&task_id);
    }

    /// Asks a running run to stop; returns whether it was running.
    pub fn request_cancel(&self, task_id: TaskId) -> bool {
        self.running.get(&task_id).is_some_and(|running| {
            running.cancel.send_replace(true);
            true
        })
    }

    /// Removes the project's waiting runs (returned) and asks its running ones to stop (their
    /// end signals returned, to wait for).
    pub fn cancel_project(
        &mut self,
        project_id: ProjectId,
    ) -> (Vec<TaskId>, Vec<oneshot::Receiver<()>>) {
        let mut removed = Vec::new();
        self.pending.retain(|key| {
            let keep = key.project_id != project_id;
            if !keep {
                removed.push(key.task_id);
            }
            keep
        });
        let mut ending = Vec::new();
        for running in self.running.values_mut() {
            if running.project_id == project_id {
                running.cancel.send_replace(true);
                ending.extend(running.finished.take());
            }
        }
        (removed, ending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(task: u64) -> RunKey {
        RunKey {
            project_id: ProjectId::from_test_index(task % 2),
            task_id: TaskId::from_test_index(task),
        }
    }

    #[test]
    fn runs_start_in_order_up_to_the_limit() {
        let mut queue = RunQueue::new(2);
        for task in 1..=3 {
            queue.push(key(task));
        }
        let first = queue.next_to_start().unwrap();
        drop(queue.mark_running(first));
        let second = queue.next_to_start().unwrap();
        drop(queue.mark_running(second));
        assert_eq!((first, second), (key(1), key(2)));
        assert_eq!(queue.next_to_start(), None);
        queue.finish(first.task_id);
        assert_eq!(queue.next_to_start(), Some(key(3)));
    }

    #[test]
    fn a_raised_limit_frees_slots_and_a_lowered_one_waits() {
        let mut queue = RunQueue::new(1);
        queue.push(key(1));
        queue.push(key(2));
        let first = queue.next_to_start().unwrap();
        drop(queue.mark_running(first));
        assert_eq!(queue.next_to_start(), None);
        queue.set_limit(2);
        assert_eq!(queue.next_to_start(), Some(key(2)));
        queue.set_limit(0);
        queue.push(key(3));
        assert_eq!(queue.next_to_start(), None);
    }

    #[test]
    fn cancelling_a_project_removes_its_waiting_runs_and_flags_its_running_ones() {
        let mut queue = RunQueue::new(1);
        queue.push(key(2));
        queue.push(key(3));
        queue.push(key(4));
        let running = queue.next_to_start().unwrap();
        let (cancelled, _gone) = queue.mark_running(running);
        let (removed, ending) = queue.cancel_project(ProjectId::from_test_index(0));
        assert_eq!(removed, [TaskId::from_test_index(4)]);
        assert_eq!(ending.len(), 1);
        assert!(*cancelled.borrow());
        assert!(queue.remove_pending(TaskId::from_test_index(3)));
        assert!(!queue.remove_pending(TaskId::from_test_index(3)));
    }
}
