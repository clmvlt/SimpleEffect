//! JSON shapes of commands.

use super::*;

#[test]
fn commands_are_tagged_by_camel_case_kind() {
    assert_eq!(
        serde_json::to_string(&Command::GetCoreInfo).unwrap(),
        r#"{"kind":"getCoreInfo"}"#
    );
    let parsed: Command = serde_json::from_str(r#"{"kind":"ping"}"#).unwrap();
    assert_eq!(parsed, Command::Ping);
}

#[test]
fn struct_command_fields_are_camel_case() {
    let undo = Command::Undo {
        project_id: ProjectId::from_test_index(1),
    };
    assert_eq!(
        serde_json::to_string(&undo).unwrap(),
        r#"{"kind":"undo","projectId":"00000000-0001-7000-8000-000000000000"}"#
    );
}

#[test]
fn track_and_marker_patches_may_omit_unchanged_fields() {
    let json = r#"{"kind":"updateTrack","projectId":"00000000-0001-7000-8000-000000000000","trackId":"00000000-0002-7000-8000-000000000000","patch":{"muted":true}}"#;
    let Command::UpdateTrack { patch, .. } = serde_json::from_str(json).unwrap() else {
        panic!("updateTrack must parse as Command::UpdateTrack");
    };
    assert_eq!(
        patch,
        TrackPatch {
            muted: Some(true),
            ..TrackPatch::default()
        }
    );
    let json = r#"{"kind":"updateMarker","projectId":"00000000-0001-7000-8000-000000000000","markerId":"00000000-0003-7000-8000-000000000000","patch":{"label":"Intro"}}"#;
    let Command::UpdateMarker { patch, .. } = serde_json::from_str(json).unwrap() else {
        panic!("updateMarker must parse as Command::UpdateMarker");
    };
    assert_eq!(patch.label.as_deref(), Some("Intro"));
    assert_eq!(patch.time, None);
}

#[test]
fn sequence_versions_carry_their_format_and_adaptation() {
    let json = r#"{"kind":"duplicateSequence","projectId":"00000000-0001-7000-8000-000000000000","sequenceId":"00000000-0002-7000-8000-000000000000","name":"Short","format":{"width":1080,"height":1920,"frameRate":{"numerator":25,"denominator":1},"sampleRate":48000},"adaptation":{"clipReframe":"fill","fitTextWidths":true}}"#;
    let Command::DuplicateSequence {
        format, adaptation, ..
    } = serde_json::from_str(json).unwrap()
    else {
        panic!("duplicateSequence must parse as Command::DuplicateSequence");
    };
    assert_eq!(format.map(|format| format.height), Some(1920));
    assert_eq!(
        adaptation,
        FormatAdaptation {
            clip_reframe: se_domain::ClipReframePolicy::Fill,
            fit_text_widths: true,
        }
    );
    // Prompt 26: automatic reframing is optional.
    let reframed = json.replace(
        r#""fitTextWidths":true}}"#,
        r#""fitTextWidths":true},"autoReframe":{"smoothing":0.5,"eyeLine":0.33,"groupFraming":true,"deadZone":0.15}}"#,
    );
    let Command::DuplicateSequence { auto_reframe, .. } = serde_json::from_str(&reframed).unwrap()
    else {
        panic!("duplicateSequence must parse as Command::DuplicateSequence");
    };
    assert_eq!(auto_reframe, Some(se_domain::TrackingSettings::default()));
}

#[test]
fn reframing_commands_carry_items_and_settings() {
    let json = r#"{"kind":"autoReframeItems","projectId":"00000000-0001-7000-8000-000000000000","itemIds":["00000000-0003-7000-8000-000000000000"],"settings":{"smoothing":0.8,"eyeLine":0.33,"groupFraming":false,"deadZone":0.0}}"#;
    let Command::AutoReframeItems {
        item_ids, settings, ..
    } = serde_json::from_str(json).unwrap()
    else {
        panic!("autoReframeItems must parse as Command::AutoReframeItems");
    };
    assert_eq!(item_ids.len(), 1);
    assert!(!settings.group_framing);
    assert_eq!(
        serde_json::to_string(&Command::GetVisionStatus).unwrap(),
        r#"{"kind":"getVisionStatus"}"#
    );
    let result = crate::CommandResult::AutoReframeItems(crate::AutoReframeReport::default());
    assert_eq!(
        serde_json::to_string(&result).unwrap(),
        r#"{"kind":"autoReframeItems","appliedItemIds":[],"waitingAssetIds":[],"skipped":[]}"#
    );
}

#[test]
fn item_updates_may_omit_their_animations() {
    let json =
        r#"{"kind":"updateItems","projectId":"00000000-0001-7000-8000-000000000000","updates":[]}"#;
    let command: Command = serde_json::from_str(json).unwrap();
    let Command::UpdateItems { animations, .. } = &command else {
        panic!("updateItems must parse as Command::UpdateItems");
    };
    assert_eq!(animations, &None);
    assert_eq!(serde_json::to_string(&command).unwrap(), json);
}

#[test]
fn create_project_carries_its_request_next_to_the_kind() {
    let json = r#"{"kind":"createProject","name":"Film","parentDir":"/p","format":{"width":1920,"height":1080,"frameRate":{"numerator":25,"denominator":1},"sampleRate":48000}}"#;
    let Command::CreateProject(request) = serde_json::from_str(json).unwrap() else {
        panic!("createProject must parse as Command::CreateProject");
    };
    assert_eq!(request.name, "Film");
    assert_eq!(request.parent_dir, "/p");
    assert_eq!(request.format.width, 1920);
}

#[test]
fn task_commands_name_their_project_and_task() {
    let json = r#"{"kind":"followUpTask","projectId":"00000000-0001-7000-8000-000000000000","taskId":"00000000-0002-7000-8000-000000000000","prompt":"Shorter"}"#;
    let Command::FollowUpTask { prompt, .. } = serde_json::from_str(json).unwrap() else {
        panic!("followUpTask must parse as Command::FollowUpTask");
    };
    assert_eq!(prompt, "Shorter");
    let json = r#"{"kind":"updateAiSettings","provider":"claude-code","executablePath":null,"defaultModel":"opus","defaultEffort":"high","maxConcurrentTasks":3}"#;
    let Command::UpdateAiSettings(settings) = serde_json::from_str(json).unwrap() else {
        panic!("updateAiSettings must parse as Command::UpdateAiSettings");
    };
    assert_eq!(settings.provider.as_str(), "claude-code");
    assert_eq!(settings.default_effort, Some(crate::EffortLevel::High));
}

#[test]
fn frame_render_failures_name_their_request() {
    let json = r#"{"kind":"reportFrameRenderFailure","requestId":"r1","message":"another project is open"}"#;
    let command: Command = serde_json::from_str(json).unwrap();
    assert_eq!(
        command,
        Command::ReportFrameRenderFailure {
            request_id: "r1".to_owned(),
            message: "another project is open".to_owned(),
        }
    );
}

#[test]
fn smart_cut_commands_carry_ranges_and_silence_params() {
    let json = r#"{"kind":"removeWords","projectId":"00000000-0001-7000-8000-000000000000","sequenceId":"00000000-0002-7000-8000-000000000000","ranges":[{"start":705600000,"duration":352800000}],"purpose":"fillers","allTracks":false}"#;
    let Command::RemoveWords {
        ranges, purpose, ..
    } = serde_json::from_str(json).unwrap()
    else {
        panic!("removeWords must parse as Command::RemoveWords");
    };
    assert_eq!(ranges.len(), 1);
    assert_eq!(purpose, crate::WordCutPurpose::Fillers);
    let json = r#"{"kind":"previewSilenceCuts","projectId":"00000000-0001-7000-8000-000000000000","sequenceId":"00000000-0002-7000-8000-000000000000","range":null,"params":{"thresholdDbfs":-45.0,"minSilence":352800000,"padding":0},"trackIds":null,"allTracks":true}"#;
    let Command::PreviewSilenceCuts {
        params, all_tracks, ..
    } = serde_json::from_str(json).unwrap()
    else {
        panic!("previewSilenceCuts must parse as Command::PreviewSilenceCuts");
    };
    assert!((params.threshold_dbfs + 45.0).abs() < f64::EPSILON);
    assert!(all_tracks);
}

#[test]
fn diagnostics_carry_what_the_window_knows() {
    let json = r#"{"kind":"exportDiagnostics","path":"/tmp/d.zip","client":{"appVersion":"0.1.0","components":[{"name":"Electron","version":"45.2.3"}],"gpu":"ANGLE","codecs":[{"codec":"H.264","decode":true,"encode":false}],"locale":"fr"}}"#;
    let Command::ExportDiagnostics { path, client } = serde_json::from_str(json).unwrap() else {
        panic!("exportDiagnostics must parse as Command::ExportDiagnostics");
    };
    assert_eq!(path, "/tmp/d.zip");
    assert_eq!(client.components[0].name, "Electron");
    assert!(client.codecs[0].decode && !client.codecs[0].encode);
    let folder: Command =
        serde_json::from_str(r#"{"kind":"setProjectsFolder","path":"/films"}"#).unwrap();
    assert_eq!(
        folder,
        Command::SetProjectsFolder {
            path: "/films".to_owned()
        }
    );
}
