//! `RpcRouter`: one text frame in, one response frame out.

use se_protocol::{
    Command, CommandResult, ProtocolError, ResponseEnvelope, ResponseOutcome, ServerMessage,
};

use super::{
    encode::encode,
    handlers::{
        HandlerContext, ai, editing, frames, graphics, items, media, media_settings, projects,
        sound, system, tasks, timeline,
    },
    request::{Rejection, parse_request},
};

/// Parses requests, dispatches each command to its handler and encodes the response.
///
/// Handlers are async; `connection.rs` runs each request in its own task, so a slow command
/// (opening a large project) never delays events or other requests. Requests of one client may
/// therefore complete in any order: a client that needs ordering waits for the previous answer.
pub struct RpcRouter {
    context: HandlerContext,
}

impl RpcRouter {
    /// A router whose handlers read `context`.
    pub fn new(context: HandlerContext) -> Self {
        Self { context }
    }

    /// Handles one text frame received on `/rpc` and returns the JSON response frame.
    pub async fn handle_text(&self, text: &str) -> String {
        let response = match parse_request(text) {
            Ok(request) => {
                tracing::debug!(id = request.id, command = ?request.command, "request");
                let outcome = match self.dispatch(&request.command).await {
                    Ok(result) => ResponseOutcome::Result(result),
                    Err(error) => {
                        tracing::info!(id = request.id, code = ?error.code, message = %error.message, "command failed");
                        ResponseOutcome::Error(error)
                    }
                };
                ResponseEnvelope {
                    id: Some(request.id),
                    outcome,
                }
            }
            Err(Rejection { id, error }) => {
                tracing::warn!(?id, code = ?error.code, message = %error.message, "request rejected");
                ResponseEnvelope {
                    id,
                    outcome: ResponseOutcome::Error(error),
                }
            }
        };
        encode(&ServerMessage::Response(response))
    }

    /// The one place mapping commands to handlers: add a match arm per new command.
    #[allow(
        clippy::too_many_lines,
        reason = "one exhaustive match arm per command, so the compiler points at new commands"
    )]
    async fn dispatch(&self, command: &Command) -> Result<CommandResult, ProtocolError> {
        let context = &self.context;
        match command {
            Command::Ping => Ok(system::ping()),
            Command::GetCoreInfo => Ok(system::get_core_info(context)),
            Command::GetNewProjectDefaults => projects::get_new_project_defaults(context).await,
            Command::CreateProject(request) => projects::create_project(context, request).await,
            Command::OpenProject { path } => projects::open_project(context, path).await,
            Command::CloseProject { project_id } => {
                projects::close_project(context, *project_id).await
            }
            Command::ListRecentProjects => projects::list_recent_projects(context).await,
            Command::RenameProject { path, name } => {
                projects::rename_project(context, path, name).await
            }
            Command::DuplicateProject { path, name } => {
                projects::duplicate_project(context, path, name).await
            }
            Command::TrashProject { path } => projects::trash_project(context, path).await,
            Command::GetProjectSnapshot { project_id } => {
                editing::get_project_snapshot(context, *project_id).await
            }
            Command::CreateSequence {
                project_id,
                name,
                format,
            } => editing::create_sequence(context, *project_id, name, *format).await,
            Command::RenameSequence {
                project_id,
                sequence_id,
                name,
            } => editing::rename_sequence(context, *project_id, *sequence_id, name).await,
            Command::SetSequenceFormat {
                project_id,
                sequence_id,
                format,
                adaptation,
            } => {
                editing::set_sequence_format(
                    context,
                    *project_id,
                    *sequence_id,
                    *format,
                    *adaptation,
                )
                .await
            }
            Command::DuplicateSequence {
                project_id,
                sequence_id,
                name,
                format,
                adaptation,
            } => {
                let copy = editing::SequenceCopy {
                    sequence_id: *sequence_id,
                    name,
                    format: *format,
                    adaptation: *adaptation,
                };
                editing::duplicate_sequence(context, *project_id, copy).await
            }
            Command::RemoveSequence {
                project_id,
                sequence_id,
            } => editing::remove_sequence(context, *project_id, *sequence_id).await,
            Command::Undo { project_id } => editing::undo(context, *project_id).await,
            Command::Redo { project_id } => editing::redo(context, *project_id).await,
            Command::AddTrack {
                project_id,
                sequence_id,
                track_kind,
                index,
            } => timeline::add_track(context, *project_id, *sequence_id, *track_kind, *index).await,
            Command::RemoveTrack {
                project_id,
                track_id,
            } => timeline::remove_track(context, *project_id, *track_id).await,
            Command::UpdateTrack {
                project_id,
                track_id,
                patch,
            } => timeline::update_track(context, *project_id, *track_id, patch).await,
            Command::ReorderTracks {
                project_id,
                sequence_id,
                order,
            } => timeline::reorder_tracks(context, *project_id, *sequence_id, order).await,
            Command::AddMarker {
                project_id,
                sequence_id,
                time,
                label,
                color,
            } => {
                timeline::add_marker(context, *project_id, *sequence_id, *time, label, *color).await
            }
            Command::UpdateMarker {
                project_id,
                marker_id,
                patch,
            } => timeline::update_marker(context, *project_id, *marker_id, patch).await,
            Command::RemoveMarker {
                project_id,
                marker_id,
            } => timeline::remove_marker(context, *project_id, *marker_id).await,
            Command::PlaceMedia(request) => items::place_media(context, request).await,
            Command::SetItemAnimations {
                project_id,
                animations,
            } => graphics::set_item_animations(context, *project_id, animations).await,
            Command::AddGraphicItem(request) => graphics::add_graphic_item(context, request).await,
            Command::ApplyAnimationPreset {
                project_id,
                item_ids,
                preset,
                phase,
                duration,
                start,
            } => {
                let request = graphics::PresetRequest {
                    item_ids,
                    preset,
                    phase: *phase,
                    duration: *duration,
                    start: *start,
                };
                graphics::apply_animation_preset(context, *project_id, request).await
            }
            Command::ListAnimationPresets => graphics::list_animation_presets(),
            Command::ListFonts => Ok(graphics::list_fonts(context).await),
            Command::MoveItems {
                project_id,
                moves,
                mode,
            } => items::move_items(context, *project_id, moves, *mode).await,
            Command::DuplicateItems {
                project_id,
                copies,
                mode,
            } => items::duplicate_items(context, *project_id, copies, *mode).await,
            Command::PasteItems {
                project_id,
                items: pasted,
                mode,
            } => items::paste_items(context, *project_id, pasted, *mode).await,
            Command::TrimItems {
                project_id,
                trims,
                ripple,
            } => items::trim_items(context, *project_id, trims, *ripple).await,
            Command::SplitItems {
                project_id,
                item_ids,
                at,
            } => items::split_items(context, *project_id, item_ids, *at).await,
            Command::DeleteItems {
                project_id,
                item_ids,
                ripple,
            } => items::delete_items(context, *project_id, item_ids, *ripple).await,
            Command::LinkItems {
                project_id,
                item_ids,
            } => items::link_items(context, *project_id, item_ids).await,
            Command::UnlinkItems {
                project_id,
                item_ids,
            } => items::unlink_items(context, *project_id, item_ids).await,
            Command::ResizeGap {
                project_id,
                track_id,
                start,
                end,
                duration,
            } => {
                items::resize_gap(context, *project_id, *track_id, (*start, *end), *duration).await
            }
            Command::UpdateItems {
                project_id,
                updates,
                animations,
            } => {
                let animations = animations.as_deref().unwrap_or_default();
                sound::update_items(context, *project_id, updates, animations).await
            }
            Command::SetItemAnimation {
                project_id,
                item_id,
                animation,
            } => sound::set_item_animation(context, *project_id, *item_id, animation).await,
            Command::AddTransitions {
                project_id,
                transitions,
            } => sound::add_transitions(context, *project_id, transitions).await,
            Command::UpdateTransition {
                project_id,
                transition_id,
                patch,
            } => sound::update_transition(context, *project_id, *transition_id, patch).await,
            Command::RemoveTransitions {
                project_id,
                transition_ids,
            } => sound::remove_transitions(context, *project_id, transition_ids).await,
            Command::ImportMedia(request) => media::import_media(context, request).await,
            Command::RelinkAsset(request) => media::relink_asset(context, request).await,
            Command::RemoveAsset {
                project_id,
                asset_id,
            } => media::remove_asset(context, *project_id, *asset_id).await,
            Command::ListJobs { project_id } => media::list_jobs(context, *project_id),
            Command::CancelJob { job_id } => media::cancel_job(context, *job_id),
            Command::ReportDecodability(report) => {
                media::report_decodability(context, report).await
            }
            Command::ReportDecoderCapabilities(capabilities) => {
                media::report_decoder_capabilities(context, *capabilities).await
            }
            Command::GenerateDerivedMedia(request) => {
                media::generate_derived_media(context, request).await
            }
            Command::GetMediaSettings => media_settings::get_media_settings(context).await,
            Command::UpdateMediaSettings(settings) => {
                media_settings::update_media_settings(context, settings).await
            }
            Command::GetMediaToolsStatus => media_settings::get_media_tools_status(context).await,
            Command::GetAudioSilences {
                project_id,
                asset_id,
            } => media::get_audio_silences(context, *project_id, *asset_id).await,
            Command::GetProjectCache { project_id } => {
                media_settings::get_project_cache(context, *project_id).await
            }
            Command::ClearProjectCache { project_id } => {
                media_settings::clear_project_cache(context, *project_id).await
            }
            Command::SubmitTask(request) => tasks::submit_task(context, request).await,
            Command::CancelTask {
                project_id,
                task_id,
            } => tasks::cancel_task(context, *project_id, *task_id).await,
            Command::RetryTask {
                project_id,
                task_id,
            } => tasks::retry_task(context, *project_id, *task_id).await,
            Command::FollowUpTask {
                project_id,
                task_id,
                prompt,
                scope,
            } => tasks::follow_up_task(context, *project_id, *task_id, prompt, scope.clone()).await,
            Command::ListTasks { project_id } => tasks::list_tasks(context, *project_id).await,
            Command::RevertTask {
                project_id,
                task_id,
            } => tasks::revert_task(context, *project_id, *task_id).await,
            Command::PreviewTaskRevert {
                project_id,
                task_id,
            } => tasks::preview_task_revert(context, *project_id, *task_id).await,
            Command::GetTaskChanges {
                project_id,
                task_id,
            } => tasks::get_task_changes(context, *project_id, *task_id).await,
            Command::GetTaskEvents {
                project_id,
                task_id,
            } => tasks::get_task_events(context, *project_id, *task_id).await,
            Command::GetAiStatus => Ok(ai::get_ai_status(context).await),
            Command::RefreshAiStatus => Ok(ai::refresh_ai_status(context).await),
            Command::GetAiSettings => Ok(ai::get_ai_settings(context)),
            Command::UpdateAiSettings(settings) => ai::update_ai_settings(context, settings).await,
            Command::ReportFrameRenderFailure {
                request_id,
                message,
            } => frames::report_failure(context, request_id, message),
        }
    }
}

/// A handler context over in-memory ports, for unit tests.
#[cfg(test)]
pub mod testing {
    use std::{
        path::{Path, PathBuf},
        sync::Arc,
    };

    use se_app::test_support::{FakeFontCatalog, MediaTestbed};

    use crate::{app_paths::AppPaths, rpc::handlers::HandlerContext};

    /// Version `1.2.3`, folders under `data`, projects and media in memory.
    pub fn context() -> HandlerContext {
        let MediaTestbed {
            projects,
            media,
            tasks,
            renders,
            ..
        } = MediaTestbed::new(Path::new("/projects"));
        HandlerContext {
            version: "1.2.3",
            paths: AppPaths {
                data_dir: PathBuf::from("data"),
                logs_dir: PathBuf::from("data").join("logs"),
            },
            projects,
            media,
            tasks: Arc::new(tasks),
            fonts: Arc::new(FakeFontCatalog),
            renders,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    async fn reply(text: &str) -> Value {
        let router = RpcRouter::new(testing::context());
        serde_json::from_str(&router.handle_text(text).await).unwrap()
    }

    #[tokio::test]
    async fn answers_ping_with_a_ping_result() {
        let router = RpcRouter::new(testing::context());
        assert_eq!(
            router
                .handle_text(r#"{"type":"request","id":7,"command":{"kind":"ping"}}"#)
                .await,
            r#"{"type":"response","id":7,"result":{"kind":"ping"}}"#
        );
    }

    #[tokio::test]
    async fn answers_get_core_info_with_the_context() {
        let reply = reply(r#"{"type":"request","id":8,"command":{"kind":"getCoreInfo"}}"#).await;
        assert_eq!(reply["id"], 8);
        assert_eq!(reply["result"]["kind"], "getCoreInfo");
        assert_eq!(reply["result"]["version"], "1.2.3");
        assert_eq!(reply["result"]["appDataDir"], "data");
    }

    #[tokio::test]
    async fn answers_malformed_json_with_invalid_request_and_a_null_id() {
        let reply = reply("{oops").await;
        assert_eq!(reply["type"], "response");
        assert_eq!(reply["id"], Value::Null);
        assert_eq!(reply["error"]["code"], "INVALID_REQUEST");
    }

    #[tokio::test]
    async fn answers_an_unknown_command_with_unknown_command_and_its_id() {
        let reply = reply(r#"{"type":"request","id":3,"command":{"kind":"explode"}}"#).await;
        assert_eq!(reply["id"], 3);
        assert_eq!(reply["error"]["code"], "UNKNOWN_COMMAND");
        assert_eq!(reply["error"]["details"]["kind"], "explode");
    }

    #[tokio::test]
    async fn use_case_errors_keep_their_code_and_message() {
        let not_open = reply(
            r#"{"type":"request","id":4,"command":{"kind":"undo","projectId":"00000000-0001-7000-8000-000000000000"}}"#,
        )
        .await;
        assert_eq!(not_open["error"]["code"], "PROJECT_NOT_OPEN");
        assert!(
            not_open["error"]["message"]
                .as_str()
                .unwrap()
                .contains("openProject")
        );
        let relative = reply(
            r#"{"type":"request","id":5,"command":{"kind":"openProject","path":"a.seproj"}}"#,
        )
        .await;
        assert_eq!(relative["error"]["code"], "INVALID_REQUEST");
    }

    #[tokio::test]
    async fn media_commands_reach_the_media_service() {
        let project = "00000000-0001-7000-8000-000000000000";
        let jobs = reply(&format!(
            r#"{{"type":"request","id":6,"command":{{"kind":"listJobs","projectId":"{project}"}}}}"#
        ))
        .await;
        assert_eq!(jobs["error"]["code"], "PROJECT_NOT_OPEN");
        let relative = reply(&format!(
            r#"{{"type":"request","id":7,"command":{{"kind":"importMedia","projectId":"{project}","paths":["clip.mp4"]}}}}"#
        ))
        .await;
        assert_eq!(relative["error"]["code"], "INVALID_REQUEST");
        let job = "00000000-0002-7000-8000-000000000000";
        let cancel = reply(&format!(
            r#"{{"type":"request","id":8,"command":{{"kind":"cancelJob","jobId":"{job}"}}}}"#
        ))
        .await;
        assert_eq!(cancel["error"]["code"], "JOB_NOT_FOUND");
        let asset = "00000000-0003-7000-8000-000000000000";
        let proxy = reply(&format!(
            r#"{{"type":"request","id":9,"command":{{"kind":"generateDerivedMedia","projectId":"{project}","assetId":"{asset}","derived":"proxy"}}}}"#
        ))
        .await;
        assert_eq!(proxy["error"]["code"], "PROJECT_NOT_OPEN");
        let capabilities = reply(
            r#"{"type":"request","id":10,"command":{"kind":"reportDecoderCapabilities","hevcMain10":true,"h264High":true,"aac":true,"flac":true}}"#,
        )
        .await;
        assert_eq!(capabilities["result"]["kind"], "reportDecoderCapabilities");
    }

    #[tokio::test]
    async fn sound_and_transition_commands_reach_the_project_service() {
        let project = "00000000-0001-7000-8000-000000000000";
        let item = "00000000-0002-7000-8000-000000000000";
        for command in [
            format!(
                r#"{{"kind":"updateItems","projectId":"{project}","updates":[{{"itemId":"{item}","patch":{{"kind":"mediaClip","fadeIn":0}}}}]}}"#
            ),
            format!(
                r#"{{"kind":"setItemAnimation","projectId":"{project}","itemId":"{item}","animation":{{"property":"volumeDb","value":{{"kind":"static","value":-6.0}}}}}}"#
            ),
            format!(
                r#"{{"kind":"addTransitions","projectId":"{project}","transitions":[{{"trackId":"{item}","leftItemId":"{item}","rightItemId":"{item}","duration":1,"alignment":"center"}}]}}"#
            ),
            format!(
                r#"{{"kind":"updateTransition","projectId":"{project}","transitionId":"{item}","patch":{{"alignment":"end"}}}}"#
            ),
            format!(
                r#"{{"kind":"removeTransitions","projectId":"{project}","transitionIds":["{item}"]}}"#
            ),
            format!(r#"{{"kind":"getAudioSilences","projectId":"{project}","assetId":"{item}"}}"#),
        ] {
            let answer = reply(&format!(
                r#"{{"type":"request","id":1,"command":{command}}}"#
            ))
            .await;
            assert_eq!(
                answer["error"]["code"], "PROJECT_NOT_OPEN",
                "{command}: {answer}"
            );
        }
    }

    #[tokio::test]
    async fn media_settings_are_read_and_validated() {
        let settings =
            reply(r#"{"type":"request","id":1,"command":{"kind":"getMediaSettings"}}"#).await;
        assert_eq!(settings["result"]["settings"]["mezzanineQuality"], 14);
        let invalid = reply(
            r#"{"type":"request","id":2,"command":{"kind":"updateMediaSettings","mezzanineQuality":99,"proxyResolution":"720p","autoProxies":true,"toolsDir":null}}"#,
        )
        .await;
        assert_eq!(invalid["error"]["code"], "INVALID_REQUEST");
        let cache = reply(
            r#"{"type":"request","id":3,"command":{"kind":"clearProjectCache","projectId":"00000000-0001-7000-8000-000000000000"}}"#,
        )
        .await;
        assert_eq!(cache["error"]["code"], "PROJECT_NOT_OPEN");
    }

    #[tokio::test]
    async fn track_and_marker_commands_reach_the_project_service() {
        let project = "00000000-0001-7000-8000-000000000000";
        let track = reply(&format!(
            r#"{{"type":"request","id":1,"command":{{"kind":"updateTrack","projectId":"{project}","trackId":"00000000-0002-7000-8000-000000000000","patch":{{"muted":true}}}}}}"#
        ))
        .await;
        assert_eq!(track["error"]["code"], "PROJECT_NOT_OPEN");
        let marker = reply(&format!(
            r#"{{"type":"request","id":2,"command":{{"kind":"addMarker","projectId":"{project}","sequenceId":"00000000-0003-7000-8000-000000000000","time":0,"label":"","color":null}}}}"#
        ))
        .await;
        assert_eq!(marker["error"]["code"], "PROJECT_NOT_OPEN");
    }

    #[tokio::test]
    async fn item_commands_reach_the_project_service() {
        let project = "00000000-0001-7000-8000-000000000000";
        let item = "00000000-0004-7000-8000-000000000000";
        let delete = reply(&format!(
            r#"{{"type":"request","id":1,"command":{{"kind":"deleteItems","projectId":"{project}","itemIds":["{item}"],"ripple":true}}}}"#
        ))
        .await;
        assert_eq!(delete["error"]["code"], "PROJECT_NOT_OPEN");
        let moved = reply(&format!(
            r#"{{"type":"request","id":2,"command":{{"kind":"moveItems","projectId":"{project}","moves":[{{"itemId":"{item}","trackId":"00000000-0002-7000-8000-000000000000","start":0}}],"mode":"insert"}}}}"#
        ))
        .await;
        assert_eq!(moved["error"]["code"], "PROJECT_NOT_OPEN");
        let gap = reply(&format!(
            r#"{{"type":"request","id":3,"command":{{"kind":"resizeGap","projectId":"{project}","trackId":"00000000-0002-7000-8000-000000000000","start":10,"end":5,"duration":0}}}}"#
        ))
        .await;
        assert_eq!(gap["error"]["code"], "INVALID_REQUEST");
    }

    #[tokio::test]
    async fn every_result_has_the_kind_of_its_command() {
        for kind in [
            "ping",
            "getCoreInfo",
            "getNewProjectDefaults",
            "listRecentProjects",
            "getMediaSettings",
            "getMediaToolsStatus",
            "listAnimationPresets",
            "listFonts",
            "getAiStatus",
            "refreshAiStatus",
            "getAiSettings",
        ] {
            let request = format!(r#"{{"type":"request","id":1,"command":{{"kind":"{kind}"}}}}"#);
            assert_eq!(reply(&request).await["result"]["kind"], kind);
        }
    }

    #[tokio::test]
    async fn late_frame_render_failures_are_refused_with_their_code() {
        let reply = reply(
            r#"{"type":"request","id":1,"command":{"kind":"reportFrameRenderFailure","requestId":"gone","message":"another project is open"}}"#,
        )
        .await;
        assert_eq!(reply["error"]["code"], "RENDER_REQUEST_NOT_FOUND");
    }
}
