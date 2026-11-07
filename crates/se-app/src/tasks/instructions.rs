//! The system instructions of a run: provider-agnostic markdown embedded at compile time
//! (`crates/se-app/assets/agent-instructions/`), composed with the task's scope.
//! TODO(prompt-18): add the MCP tool, time format and scope guides with the tools.

use std::fmt::Write as _;

use se_protocol::Task;

/// The agent's role and limits.
const ROLE: &str = include_str!("../../assets/agent-instructions/role.md");

/// The instructions added to the CLI's system prompt for `task`.
pub(super) fn compose(task: &Task) -> String {
    let mut text = String::from(ROLE.trim_end());
    text.push_str("\n\n# Task scope\n\n");
    // Writing to a String cannot fail.
    let _ = writeln!(text, "- Sequence: `{}`", task.scope.sequence_id);
    match task.scope.range {
        Some(range) => {
            let _ = writeln!(
                text,
                "- Time range: {:.3} s to {:.3} s; change nothing outside it.",
                range.start().to_seconds_f64(),
                range.end().to_seconds_f64()
            );
        }
        None => text.push_str("- Time range: the whole sequence.\n"),
    }
    match &task.scope.tracks {
        Some(tracks) => {
            let ids: Vec<String> = tracks.iter().map(|track| format!("`{track}`")).collect();
            let _ = writeln!(text, "- Tracks: {}; change no other track.", ids.join(", "));
        }
        None => text.push_str("- Tracks: every track of the sequence.\n"),
    }
    text
}

#[cfg(test)]
mod tests {
    use se_domain::{Flicks, ProjectId, SequenceId, TaskId, TimeRange, TrackId};
    use se_protocol::{ProviderId, TaskScope, TaskStatus};

    use super::*;

    #[test]
    fn instructions_describe_the_scope() {
        let task = Task {
            id: TaskId::from_test_index(1),
            project_id: ProjectId::from_test_index(2),
            prompt: "Tighten".to_owned(),
            scope: TaskScope {
                sequence_id: SequenceId::from_test_index(3),
                range: Some(TimeRange::new(Flicks::one_second(), Flicks::one_second()).unwrap()),
                tracks: Some(vec![TrackId::from_test_index(4)]),
            },
            provider: ProviderId("fake".to_owned()),
            model: None,
            effort: None,
            status: TaskStatus::Queued,
            agent_session_id: None,
            created_at: 0,
            updated_at: 0,
            runs: Vec::new(),
        };
        let text = compose(&task);
        assert!(text.starts_with("# Role"));
        assert!(text.contains("1.000 s to 2.000 s"));
        assert!(text.contains(&TrackId::from_test_index(4).to_string()));
    }
}
