//! `RpcRouter`: one text frame in, one response frame out.

use std::time::{Duration, Instant};

use se_protocol::{
    Command, CommandResult, ProtocolError, ResponseEnvelope, ResponseOutcome, ServerMessage,
};

use super::{
    encode::encode,
    handlers::{
        HandlerContext, ai, captions, diagnostics, editing, exports, frames, graphics, items,
        library, media, media_settings, projects, smart_cuts, sound, system, tasks, templates,
        timeline, transcription, vision,
    },
    request::{Rejection, parse_request},
};

/// Commands taking longer than this are logged at `info` (prompt 29: performance budgets,
/// `docs/performance.md`). Long-running commands by nature (imports, exports' setup, waits for
/// the renderer) show up too: the log says which command it was.
const SLOW_COMMAND: Duration = Duration::from_millis(250);

/// Logs how long a command took: every command at `debug`, slow ones at `info`. The command's
/// kind is read from the frame only when the line is written.
fn log_duration(id: u64, text: &str, elapsed: Duration) {
    let slow = elapsed >= SLOW_COMMAND;
    if !slow && !tracing::enabled!(tracing::Level::DEBUG) {
        return;
    }
    let kind = command_kind(text);
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    if slow {
        tracing::info!(id, %kind, elapsed_ms, "slow command");
    } else {
        tracing::debug!(id, %kind, elapsed_ms, "command answered");
    }
}

/// The `command.kind` of a request frame (`?` when unreadable).
fn command_kind(text: &str) -> String {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|frame| frame["command"]["kind"].as_str().map(str::to_owned))
        .unwrap_or_else(|| "?".to_owned())
}

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
                let started = Instant::now();
                let dispatched = self.dispatch(&request.command).await;
                log_duration(request.id, text, started.elapsed());
                let outcome = match dispatched {
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
            Command::SetProjectsFolder { path } => {
                projects::set_projects_folder(context, path).await
            }
            Command::ExportDiagnostics { path, client } => {
                diagnostics::export_diagnostics(context, path, client).await
            }
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
                auto_reframe,
            } => {
                let copy = editing::SequenceCopy {
                    sequence_id: *sequence_id,
                    name,
                    format: *format,
                    adaptation: *adaptation,
                    auto_reframe: *auto_reframe,
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
            Command::ReportTemplateCheck {
                request_id,
                outcome,
            } => templates::report_template_check(context, request_id, outcome.clone()),
            Command::RemoveTemplate {
                project_id,
                template_id,
            } => templates::remove_template(context, *project_id, *template_id).await,
            Command::GetTranscriptionStatus
            | Command::UpdateTranscriptionPreferences(_)
            | Command::StartTranscriptionDownload { .. }
            | Command::CancelTranscriptionDownload { .. }
            | Command::ImportTranscriptionModel { .. }
            | Command::RemoveTranscriptionFile { .. }
            | Command::TranscribeAssets { .. }
            | Command::GetTranscript { .. }
            | Command::SearchTranscripts { .. }
            | Command::SetTranscriptionSettings { .. } => {
                Self::dispatch_transcription(context, command).await
            }
            Command::GetVisionStatus => vision::get_status(context).await,
            Command::AutoReframeItems {
                project_id,
                item_ids,
                settings,
            } => vision::auto_reframe_items(context, *project_id, item_ids, *settings).await,
            Command::GetSubjectDetection {
                project_id,
                asset_id,
            } => vision::get_subject_detection(context, *project_id, *asset_id).await,
            Command::GetSmartCutSettings
            | Command::UpdateSmartCutSettings(_)
            | Command::RemoveWords { .. }
            | Command::FindFillers { .. }
            | Command::PreviewSilenceCuts { .. }
            | Command::RemoveSilences { .. } => Self::dispatch_smart_cuts(context, command).await,
            Command::GenerateCaptions(_)
            | Command::ImportSubtitles(_)
            | Command::ExportSubtitles { .. }
            | Command::EditCaptionText(_)
            | Command::MergeCaptions { .. }
            | Command::SplitCaption { .. }
            | Command::ShiftCaptions { .. }
            | Command::SetCaptionEmphasis(_)
            | Command::UpdateCaptionStyle(_)
            | Command::ApplyCaptionPreset { .. } => Self::dispatch_captions(context, command).await,
            Command::ListLibraryTemplates
            | Command::SaveTemplateToLibrary { .. }
            | Command::SaveCaptionStyleToLibrary { .. }
            | Command::UpdateLibraryTemplateInfo { .. }
            | Command::RemoveLibraryTemplate { .. }
            | Command::InsertLibraryTemplate(_)
            | Command::UpdateTemplateFromLibrary { .. }
            | Command::ExportLibraryTemplate { .. }
            | Command::ImportLibraryTemplates { .. }
            | Command::ListBrandKits
            | Command::CreateBrandKit { .. }
            | Command::SaveBrandKit { .. }
            | Command::RemoveBrandKit { .. }
            | Command::AddBrandKitLogo { .. }
            | Command::SetProjectBrandKit { .. }
            | Command::AddProjectBrandKitToLibrary { .. } => {
                Self::dispatch_library(context, command).await
            }
            Command::StartExport(_)
            | Command::CancelExport { .. }
            | Command::ListExports
            | Command::ClearFinishedExports
            | Command::ReportExportProgress { .. }
            | Command::FinishExportRender { .. }
            | Command::FailExportRender { .. }
            | Command::GetExportPreferences
            | Command::UpdateExportPreferences(_) => exports::dispatch(context, command).await,
        }
    }

    /// The template library and brand kit commands (prompt 25).
    async fn dispatch_library(
        context: &HandlerContext,
        command: &Command,
    ) -> Result<CommandResult, ProtocolError> {
        match command {
            Command::ListLibraryTemplates => library::list_templates(context).await,
            Command::SaveTemplateToLibrary {
                project_id,
                template_id,
                info,
            } => library::save_template(context, *project_id, *template_id, info).await,
            Command::SaveCaptionStyleToLibrary {
                project_id,
                style_id,
                info,
            } => library::save_caption_style(context, *project_id, *style_id, info).await,
            Command::UpdateLibraryTemplateInfo { template_id, info } => {
                library::update_info(context, *template_id, info).await
            }
            Command::RemoveLibraryTemplate { template_id } => {
                library::remove_template(context, *template_id).await
            }
            Command::InsertLibraryTemplate(request) => {
                library::insert_template(context, request).await
            }
            Command::UpdateTemplateFromLibrary {
                project_id,
                template_id,
            } => library::update_project_template(context, *project_id, *template_id).await,
            Command::ExportLibraryTemplate { template_id, path } => {
                library::export_template(context, *template_id, path).await
            }
            Command::ImportLibraryTemplates { project_id, paths } => {
                library::import_templates(context, *project_id, paths).await
            }
            Command::ListBrandKits => library::list_brand_kits(context).await,
            Command::CreateBrandKit { name, copy_of } => {
                library::create_brand_kit(context, name, *copy_of).await
            }
            Command::SaveBrandKit { kit } => library::save_brand_kit(context, kit).await,
            Command::RemoveBrandKit { kit_id } => library::remove_brand_kit(context, *kit_id).await,
            Command::AddBrandKitLogo { kit_id, path } => {
                library::add_brand_kit_logo(context, *kit_id, path).await
            }
            Command::SetProjectBrandKit { project_id, kit_id } => {
                library::set_project_brand_kit(context, *project_id, *kit_id).await
            }
            Command::AddProjectBrandKitToLibrary { project_id } => {
                library::add_project_brand_kit_to_library(context, *project_id).await
            }
            _ => Err(ProtocolError::new(
                se_protocol::ErrorCode::Internal,
                "a non-library command reached the library dispatcher",
            )),
        }
    }

    /// The caption commands (prompt 23).
    async fn dispatch_captions(
        context: &HandlerContext,
        command: &Command,
    ) -> Result<CommandResult, ProtocolError> {
        match command {
            Command::GenerateCaptions(request) => captions::generate(context, request).await,
            Command::ImportSubtitles(request) => captions::import_subtitles(context, request).await,
            Command::ExportSubtitles {
                project_id,
                sequence_id,
                path,
            } => captions::export_subtitles(context, *project_id, *sequence_id, path).await,
            Command::EditCaptionText(edit) => captions::edit_text(context, edit).await,
            Command::MergeCaptions {
                project_id,
                item_ids,
            } => captions::merge(context, *project_id, item_ids).await,
            Command::SplitCaption {
                project_id,
                item_id,
                word_index,
            } => captions::split(context, *project_id, *item_id, *word_index).await,
            Command::ShiftCaptions {
                project_id,
                item_ids,
                delta,
            } => captions::shift(context, *project_id, item_ids, *delta).await,
            Command::SetCaptionEmphasis(edit) => captions::set_emphasis(context, edit).await,
            Command::UpdateCaptionStyle(edit) => captions::update_style(context, edit).await,
            Command::ApplyCaptionPreset {
                project_id,
                item_ids,
                preset,
            } => captions::apply_preset(context, *project_id, item_ids, preset).await,
            _ => Err(ProtocolError::new(
                se_protocol::ErrorCode::Internal,
                "a non-caption command reached the caption dispatcher",
            )),
        }
    }

    /// The smart-cut commands (prompt 22).
    async fn dispatch_smart_cuts(
        context: &HandlerContext,
        command: &Command,
    ) -> Result<CommandResult, ProtocolError> {
        match command {
            Command::GetSmartCutSettings => smart_cuts::get_settings(context).await,
            Command::UpdateSmartCutSettings(settings) => {
                smart_cuts::update_settings(context, settings).await
            }
            Command::RemoveWords {
                project_id,
                sequence_id,
                ranges,
                purpose,
                all_tracks,
            } => {
                smart_cuts::remove_words(
                    context,
                    *project_id,
                    *sequence_id,
                    ranges,
                    *purpose,
                    *all_tracks,
                )
                .await
            }
            Command::FindFillers {
                project_id,
                sequence_id,
                range,
            } => smart_cuts::find_fillers(context, *project_id, *sequence_id, *range).await,
            Command::PreviewSilenceCuts {
                project_id,
                sequence_id,
                range,
                params,
                track_ids,
                all_tracks,
            } => {
                let arguments = smart_cuts::SilenceArguments {
                    project_id: *project_id,
                    sequence_id: *sequence_id,
                    range: *range,
                    params: *params,
                    track_ids: track_ids.as_deref(),
                    all_tracks: *all_tracks,
                };
                smart_cuts::preview_silences(context, &arguments).await
            }
            Command::RemoveSilences {
                project_id,
                sequence_id,
                range,
                params,
                track_ids,
                all_tracks,
            } => {
                let arguments = smart_cuts::SilenceArguments {
                    project_id: *project_id,
                    sequence_id: *sequence_id,
                    range: *range,
                    params: *params,
                    track_ids: track_ids.as_deref(),
                    all_tracks: *all_tracks,
                };
                smart_cuts::remove_silences(context, &arguments).await
            }
            other => Err(ProtocolError::new(
                se_protocol::ErrorCode::Internal,
                format!("{other:?} is not a smart-cut command; please report it"),
            )),
        }
    }

    /// The transcription commands (prompt 21).
    async fn dispatch_transcription(
        context: &HandlerContext,
        command: &Command,
    ) -> Result<CommandResult, ProtocolError> {
        match command {
            Command::GetTranscriptionStatus => transcription::get_status(context).await,
            Command::UpdateTranscriptionPreferences(preferences) => {
                transcription::update_preferences(context, preferences).await
            }
            Command::StartTranscriptionDownload { target } => {
                transcription::start_download(context, target)
            }
            Command::CancelTranscriptionDownload { target } => {
                transcription::cancel_download(context, target)
            }
            Command::ImportTranscriptionModel { path } => {
                transcription::import_model(context, path).await
            }
            Command::RemoveTranscriptionFile { target } => {
                transcription::remove_file(context, target).await
            }
            Command::TranscribeAssets {
                project_id,
                asset_ids,
            } => transcription::transcribe_assets(context, *project_id, asset_ids).await,
            Command::GetTranscript {
                project_id,
                asset_id,
            } => transcription::get_transcript(context, *project_id, *asset_id).await,
            Command::SearchTranscripts { project_id, query } => {
                transcription::search(context, *project_id, query).await
            }
            Command::SetTranscriptionSettings {
                project_id,
                settings,
            } => transcription::set_settings(context, *project_id, settings).await,
            other => Err(ProtocolError::new(
                se_protocol::ErrorCode::Internal,
                format!("{other:?} is not a transcription command; please report it"),
            )),
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

    use se_app::{
        DiagnosticsService,
        test_support::{FakeDiagnosticsFiles, FakeFontCatalog, MediaTestbed},
    };

    use crate::{app_paths::AppPaths, rpc::handlers::HandlerContext};

    /// Version `1.2.3`, folders under `data`, projects and media in memory.
    pub fn context() -> HandlerContext {
        let MediaTestbed {
            projects,
            media,
            tasks,
            renders,
            template_checks,
            transcription,
            smart_cuts,
            captions,
            library_templates,
            brand_kits,
            reframing,
            exports,
            clock,
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
            template_checks,
            transcription,
            smart_cuts,
            captions,
            library_templates,
            brand_kits,
            reframing,
            exports,
            diagnostics: DiagnosticsService::new(
                Arc::new(FakeDiagnosticsFiles::default()),
                clock,
                PathBuf::from("data").join("logs"),
                None,
            ),
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

    #[test]
    fn the_kind_of_a_command_is_read_for_duration_logs() {
        assert_eq!(
            command_kind(r#"{"type":"request","id":1,"command":{"kind":"undo","projectId":"x"}}"#),
            "undo"
        );
        assert_eq!(command_kind("not json"), "?");
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
            "getTranscriptionStatus",
            "getSmartCutSettings",
            "listLibraryTemplates",
            "listBrandKits",
            "getVisionStatus",
            "listExports",
            "clearFinishedExports",
            "getExportPreferences",
        ] {
            let request = format!(r#"{{"type":"request","id":1,"command":{{"kind":"{kind}"}}}}"#);
            assert_eq!(reply(&request).await["result"]["kind"], kind);
        }
    }

    #[tokio::test]
    async fn first_run_and_diagnostics_commands_reach_their_handlers() {
        let root = if cfg!(windows) { "C:\\" } else { "/" };
        let folder = format!("{root}Films");
        let request = |id: u64, command: serde_json::Value| {
            serde_json::json!({ "type": "request", "id": id, "command": command }).to_string()
        };
        let set = reply(&request(
            1,
            serde_json::json!({ "kind": "setProjectsFolder", "path": folder }),
        ))
        .await;
        assert_eq!(set["result"]["kind"], "setProjectsFolder", "{set}");
        assert_eq!(set["result"]["defaults"]["parentDir"], folder.as_str());
        let client = serde_json::json!({
            "appVersion": "1.2.3", "components": [], "gpu": null, "codecs": [], "locale": "fr"
        });
        let exported = reply(&request(
            2,
            serde_json::json!({
                "kind": "exportDiagnostics", "path": format!("{root}out{}diag.zip", &root[root.len() - 1..]), "client": client
            }),
        ))
        .await;
        assert_eq!(
            exported["result"]["kind"], "exportDiagnostics",
            "{exported}"
        );
        assert_eq!(exported["result"]["files"][1], "report.json");
        let relative = reply(&request(
            3,
            serde_json::json!({ "kind": "exportDiagnostics", "path": "diag.zip", "client": client }),
        ))
        .await;
        assert_eq!(relative["error"]["code"], "INVALID_REQUEST");
    }

    #[tokio::test]
    async fn reframing_commands_reach_their_handlers() {
        let project = "00000000-0009-7000-8000-000000000000";
        let reframe = reply(&format!(
            r#"{{"type":"request","id":1,"command":{{"kind":"autoReframeItems","projectId":"{project}","itemIds":[],"settings":{{"smoothing":0.5,"eyeLine":0.33,"groupFraming":true,"deadZone":0.15}}}}}}"#
        ))
        .await;
        assert_eq!(reframe["error"]["code"], "PROJECT_NOT_OPEN");
        let faces = reply(&format!(
            r#"{{"type":"request","id":2,"command":{{"kind":"getSubjectDetection","projectId":"{project}","assetId":"00000000-0003-7000-8000-000000000000"}}}}"#
        ))
        .await;
        assert_eq!(faces["error"]["code"], "PROJECT_NOT_OPEN");
    }

    #[tokio::test]
    async fn caption_commands_reach_their_handlers() {
        let project = "00000000-0009-7000-8000-000000000000";
        let sequence = "00000000-0001-7000-8000-000000000000";
        let generate = reply(&format!(
            r#"{{"type":"request","id":1,"command":{{"kind":"generateCaptions","projectId":"{project}","sequenceId":"{sequence}"}}}}"#
        ))
        .await;
        assert_eq!(generate["error"]["code"], "PROJECT_NOT_OPEN");
        let export = reply(&format!(
            r#"{{"type":"request","id":2,"command":{{"kind":"exportSubtitles","projectId":"{project}","sequenceId":"{sequence}","path":"film.srt"}}}}"#
        ))
        .await;
        assert_eq!(export["error"]["code"], "INVALID_REQUEST");
    }

    #[tokio::test]
    async fn export_commands_reach_their_handlers() {
        let export = "00000000-0001-7000-8000-000000000000";
        let cancel = reply(&format!(
            r#"{{"type":"request","id":1,"command":{{"kind":"cancelExport","exportId":"{export}"}}}}"#
        ))
        .await;
        assert_eq!(cancel["error"]["code"], "EXPORT_NOT_FOUND");
        let progress = reply(&format!(
            r#"{{"type":"request","id":2,"command":{{"kind":"reportExportProgress","exportId":"{export}","done":1,"total":2}}}}"#
        ))
        .await;
        assert_eq!(progress["error"]["code"], "EXPORT_NOT_FOUND");
        let start = reply(
            r#"{"type":"request","id":3,"command":{"kind":"startExport","projectId":"00000000-0009-7000-8000-000000000000","sequenceId":"00000000-0001-7000-8000-000000000000","settings":{"container":"wav","video":null,"audio":{"codec":"pcm","bitsPerSecond":1536000,"encoder":"webCodecs"}},"path":"son.wav","presetName":"WAV"}}"#,
        )
        .await;
        assert_eq!(start["error"]["code"], "INVALID_EXPORT");
    }

    #[tokio::test]
    async fn late_frame_render_failures_are_refused_with_their_code() {
        let reply = reply(
            r#"{"type":"request","id":1,"command":{"kind":"reportFrameRenderFailure","requestId":"gone","message":"another project is open"}}"#,
        )
        .await;
        assert_eq!(reply["error"]["code"], "RENDER_REQUEST_NOT_FOUND");
    }

    #[tokio::test]
    async fn template_commands_reach_their_handlers() {
        let late = reply(
            r#"{"type":"request","id":1,"command":{"kind":"reportTemplateCheck","requestId":"gone","outcome":{"kind":"failed","problems":[]}}}"#,
        )
        .await;
        assert_eq!(late["error"]["code"], "TEMPLATE_CHECK_NOT_FOUND");
        let remove = reply(
            r#"{"type":"request","id":1,"command":{"kind":"removeTemplate","projectId":"00000000-0009-7000-8000-000000000000","templateId":"00000000-0001-7000-8000-000000000000"}}"#,
        )
        .await;
        assert_eq!(remove["error"]["code"], "PROJECT_NOT_OPEN");
    }
}
