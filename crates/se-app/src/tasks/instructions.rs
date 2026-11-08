//! The system instructions of a run: provider-agnostic markdown embedded at compile time
//! (`crates/se-app/assets/agent-instructions/`, in file order), then the task's scope and a short
//! summary of the project, so the agent knows where it stands before its first tool call.

use std::fmt::Write as _;

use se_domain::{AssetKind, Project, Sequence, TrackKind, format_clock};
use se_protocol::Task;

/// The instruction files, in order.
const GUIDES: [&str; 3] = [
    include_str!("../../assets/agent-instructions/00-role.md"),
    include_str!("../../assets/agent-instructions/01-time-and-scope.md"),
    include_str!("../../assets/agent-instructions/02-reading-the-project.md"),
];

/// The instructions added to the CLI's system prompt for `task` on `project` (`None` when the
/// project could not be read: the agent then discovers it with its tools).
pub(super) fn compose(task: &Task, project: Option<&Project>) -> String {
    let mut text = GUIDES
        .iter()
        .map(|guide| guide.trim_end())
        .collect::<Vec<_>>()
        .join("\n\n");
    text.push_str("\n\n# Task scope\n\n");
    write_scope(&mut text, task, project);
    if let Some(project) = project {
        text.push_str("\n# Project\n\n");
        write_project(&mut text, task, project);
    }
    text
}

fn write_scope(text: &mut String, task: &Task, project: Option<&Project>) {
    let sequence = project.and_then(|project| {
        project
            .sequences
            .iter()
            .find(|sequence| sequence.id == task.scope.sequence_id)
    });
    // Writing to a String cannot fail.
    match sequence {
        Some(sequence) => {
            let _ = writeln!(
                text,
                "- Sequence: « {} » (`{}`)",
                sequence.name, sequence.id
            );
        }
        None => {
            let _ = writeln!(text, "- Sequence: `{}`", task.scope.sequence_id);
        }
    }
    match task.scope.range {
        Some(range) => {
            let _ = writeln!(
                text,
                "- Time range: {} to {}; change nothing outside it.",
                format_clock(range.start()),
                format_clock(range.end())
            );
        }
        None => text.push_str("- Time range: the whole sequence.\n"),
    }
    match &task.scope.tracks {
        Some(tracks) => {
            let names: Vec<String> = tracks
                .iter()
                .map(|id| {
                    let name = sequence
                        .and_then(|sequence| sequence.tracks.iter().find(|track| track.id == *id))
                        .map(|track| format!("{} ", track.name))
                        .unwrap_or_default();
                    format!("{name}(`{id}`)")
                })
                .collect();
            let _ = writeln!(
                text,
                "- Tracks: {}; change no other track.",
                names.join(", ")
            );
        }
        None => text.push_str("- Tracks: every track of the sequence.\n"),
    }
}

fn write_project(text: &mut String, task: &Task, project: &Project) {
    let _ = writeln!(
        text,
        "- Name: « {} » (revision {})",
        project.name, project.revision
    );
    for sequence in &project.sequences {
        let mark = if sequence.id == task.scope.sequence_id {
            " — the task's sequence"
        } else {
            ""
        };
        let _ = writeln!(
            text,
            "- Sequence « {} » (`{}`): {}×{}, {:.3} fps, lasts {}{mark}",
            sequence.name,
            sequence.id,
            sequence.format.width,
            sequence.format.height,
            sequence.format.frame_rate.as_f64(),
            format_clock(sequence.content_end()),
        );
    }
    if let Some(sequence) = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == task.scope.sequence_id)
    {
        let _ = writeln!(
            text,
            "- Tracks of the task's sequence: {}",
            tracks_line(sequence)
        );
    }
    let count = |kind: AssetKind| {
        project
            .assets
            .iter()
            .filter(|asset| asset.kind == kind)
            .count()
    };
    let _ = writeln!(
        text,
        "- Media: {} ({} video, {} audio, {} image)",
        project.assets.len(),
        count(AssetKind::Video),
        count(AssetKind::Audio),
        count(AssetKind::Image)
    );
}

/// `V1 (video, 3 items), A1 (audio, 2 items, muted)`, bottom video track first.
fn tracks_line(sequence: &Sequence) -> String {
    let lines: Vec<String> = sequence
        .tracks
        .iter()
        .map(|track| {
            let kind = match track.kind {
                TrackKind::Video => "video",
                TrackKind::Audio => "audio",
            };
            let mut flags = String::new();
            for (on, flag) in [
                (track.muted, ", muted"),
                (track.hidden, ", hidden"),
                (track.locked, ", locked"),
            ] {
                if on {
                    flags.push_str(flag);
                }
            }
            format!(
                "{} ({kind}, {} items{flags})",
                track.name,
                track.items.len()
            )
        })
        .collect();
    if lines.is_empty() {
        "none".to_owned()
    } else {
        lines.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use se_domain::{Flicks, ProjectId, TimeRange, TrackId, test_support::example_project};
    use se_protocol::{ProviderId, TaskScope, TaskStatus};

    use super::*;

    fn task(project: &Project, tracks: Option<Vec<TrackId>>) -> Task {
        Task {
            id: se_domain::TaskId::from_test_index(1),
            project_id: ProjectId::from_test_index(2),
            prompt: "Tighten".to_owned(),
            scope: TaskScope {
                sequence_id: project.sequences[0].id,
                range: Some(TimeRange::new(Flicks::one_second(), Flicks::one_second()).unwrap()),
                tracks,
            },
            provider: ProviderId("fake".to_owned()),
            model: None,
            effort: None,
            status: TaskStatus::Queued,
            agent_session_id: None,
            created_at: 0,
            updated_at: 0,
            runs: Vec::new(),
        }
    }

    #[test]
    fn instructions_describe_the_scope_and_the_project() {
        let project = example_project();
        let track = &project.sequences[0].tracks[0];
        let text = compose(&task(&project, Some(vec![track.id])), Some(&project));
        assert!(text.starts_with("# Role"));
        assert!(text.contains("# Time and scope"));
        assert!(text.contains("# Reading the project"));
        assert!(text.contains("00:00:01.000 to 00:00:02.000"));
        assert!(text.contains(&format!("{} (`{}`)", track.name, track.id)));
        assert!(text.contains("« Main edit »"));
        assert!(text.contains("the task's sequence"));
        assert!(text.contains("- Media: 3 (1 video, 1 audio, 1 image)"));
        assert!(text.contains("V1 (video, 2 items)"));
    }

    #[test]
    fn instructions_without_the_project_still_name_the_scope() {
        let project = example_project();
        let text = compose(&task(&project, None), None);
        assert!(text.contains(&format!("- Sequence: `{}`", project.sequences[0].id)));
        assert!(text.contains("every track"));
        assert!(!text.contains("# Project"));
    }
}
