//! The transcription use cases against the in-memory ports (`MediaTestbed`): status and
//! recommendation, downloads (voice model alongside, failures, cancellation), imports and
//! removals, jobs (request, summary, settings used), automatic transcription on import, a
//! sequence's words, search and the project's settings.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::path::{Path, PathBuf};

use se_app::{
    test_support::{
        FakeModelFile, MediaTestbed, TESTBED_MODELS_DIR, fake_gpu_offer, probed_video, settle,
    },
    transcription::{MODELS, VOICE_MODEL},
};
use se_domain::{AssetId, Flicks, ProjectId, TranscriptionSettings};
use se_protocol::{
    CoreEvent, DownloadProgress, DownloadTarget, ErrorCode, PlaceMediaRequest, PlacementMode,
    ProtocolError, TrackTarget, TranscriptionBackend, TranscriptionPreferences,
};

const PARENT: &str = "/films";

fn code(error: se_app::AppError) -> ErrorCode {
    ProtocolError::from(error).code
}

fn models_dir() -> PathBuf {
    PathBuf::from(TESTBED_MODELS_DIR)
}

fn model_target(id: &str) -> DownloadTarget {
    DownloadTarget::Model {
        model_id: id.to_owned(),
    }
}

async fn started() -> MediaTestbed {
    let bed = MediaTestbed::new(Path::new(PARENT));
    bed.transcription.start().await.unwrap();
    bed
}

/// Installs catalogue model `id` (and the voice model) as if downloaded.
fn install(bed: &MediaTestbed, id: &str) {
    let model = MODELS.iter().find(|model| model.id == id).unwrap();
    for file in [model.file, VOICE_MODEL] {
        bed.model_files.put(
            &models_dir().join(file.file_name),
            FakeModelFile {
                size: file.size,
                sha256: file.sha256.to_owned(),
                is_model: true,
            },
        );
    }
}

async fn project_with_video(bed: &MediaTestbed) -> (ProjectId, AssetId) {
    let project_id = bed.create_project(Path::new(PARENT), "Film").await.unwrap();
    let path = Path::new(PARENT).join("rushes").join("interview.mp4");
    bed.files.add(path.clone(), 1, probed_video());
    let report = bed.media.import_files(project_id, &[path]).await.unwrap();
    (project_id, report.imported[0])
}

fn status_events(bed: &MediaTestbed) -> usize {
    bed.events
        .take()
        .iter()
        .filter(|event| matches!(event, CoreEvent::TranscriptionStatusChanged { .. }))
        .count()
}

#[tokio::test]
async fn the_status_lists_the_catalogue_and_recommends_by_hardware() {
    let bed = started().await;
    let status = bed.transcription.status().await.unwrap();
    assert_eq!(status.models.len(), 5);
    assert!(status.models.iter().all(|model| !model.installed));
    let recommended: Vec<&str> = status
        .models
        .iter()
        .filter(|model| model.recommended)
        .map(|model| model.id.as_str())
        .collect();
    assert_eq!(recommended, ["small"], "16 threads, no GPU");
    assert_eq!(status.backend, TranscriptionBackend::Cpu);
    assert!(status.whisper.path.is_some() && status.whisper.problem.is_none());
    assert_eq!(status.gpu_runtime, None);

    let mut environment = se_app::ports::Transcriber::environment(&bed.transcriber).await;
    environment.gpu_name = Some("NVIDIA GeForce RTX 3080".to_owned());
    environment.gpu_runtime = Some(fake_gpu_offer(false));
    bed.transcriber.set_environment(environment);
    let preferences = TranscriptionPreferences::default();
    let status = bed
        .transcription
        .update_preferences(preferences)
        .await
        .unwrap();
    let turbo = status
        .models
        .iter()
        .find(|model| model.recommended)
        .unwrap();
    assert_eq!(turbo.id, "large-v3-turbo");
    assert_eq!(status.gpu_runtime.unwrap().name, "CUDA 12.4");
}

#[tokio::test]
async fn downloading_a_first_model_also_fetches_the_voice_model_and_reports_progress() {
    let bed = started().await;
    bed.transcription
        .start_download(&model_target("base"))
        .unwrap();
    settle().await;
    let specs = bed.downloader.specs();
    let urls: Vec<&str> = specs.iter().map(|spec| spec.url.as_str()).collect();
    assert!(urls[0].ends_with("/ggml-base.bin") && urls[1].ends_with(VOICE_MODEL.file_name));
    assert_eq!(specs[0].destination, models_dir().join("ggml-base.bin"));
    let status = bed.transcription.status().await.unwrap();
    assert!(status.models[1].installed && status.models[1].download.is_none());
    assert!(status.voice_model.installed);
    assert!(status_events(&bed) >= 3, "start, progress, end");
}

#[tokio::test]
async fn failed_downloads_say_why_and_cancelled_ones_install_nothing() {
    let bed = started().await;
    install(&bed, "tiny");
    bed.downloader.fail_next();
    bed.transcription
        .start_download(&model_target("small"))
        .unwrap();
    settle().await;
    let status = bed.transcription.status().await.unwrap();
    let small = &status.models[2];
    assert!(!small.installed);
    assert!(
        matches!(&small.download, Some(DownloadProgress::Failed { message }) if message.contains("SHA-256"))
    );

    bed.downloader.hold();
    bed.transcription
        .start_download(&model_target("small"))
        .unwrap();
    settle().await;
    let status = bed.transcription.status().await.unwrap();
    assert!(matches!(
        status.models[2].download,
        Some(DownloadProgress::Downloading { .. })
    ));
    bed.transcription
        .cancel_download(&model_target("small"))
        .unwrap();
    bed.downloader.release();
    settle().await;
    let status = bed.transcription.status().await.unwrap();
    assert!(!status.models[2].installed && status.models[2].download.is_none());
    assert_eq!(
        code(
            bed.transcription
                .cancel_download(&model_target("small"))
                .unwrap_err()
        ),
        ErrorCode::TranscriptionTargetNotFound
    );
    assert_eq!(
        code(
            bed.transcription
                .start_download(&model_target("huge"))
                .unwrap_err()
        ),
        ErrorCode::TranscriptionTargetNotFound
    );
}

#[tokio::test]
async fn imports_recognise_catalogue_models_keep_custom_ones_and_refuse_other_files() {
    let bed = started().await;
    let base = MODELS[1].file;
    let downloaded = PathBuf::from("/downloads/whatever.bin");
    let custom = PathBuf::from("/downloads/french-distil.bin");
    let text = PathBuf::from("/downloads/notes.txt");
    for (path, sha256, is_model) in [
        (&downloaded, base.sha256, true),
        (&custom, "ab", true),
        (&text, "cd", false),
    ] {
        bed.model_files.put(
            path,
            FakeModelFile {
                size: 10,
                sha256: sha256.to_owned(),
                is_model,
            },
        );
    }
    let (id, _) = bed.transcription.import_model(&downloaded).await.unwrap();
    assert_eq!(id, "base");
    assert!(bed.model_files.exists(&models_dir().join("ggml-base.bin")));
    let (id, status) = bed.transcription.import_model(&custom).await.unwrap();
    assert_eq!(id, "french-distil.bin");
    assert!(
        status
            .models
            .iter()
            .any(|model| model.custom && model.id == id)
    );
    assert_eq!(
        code(bed.transcription.import_model(&text).await.unwrap_err()),
        ErrorCode::InvalidModelFile
    );

    let preferences = TranscriptionPreferences {
        model: Some("french-distil.bin".to_owned()),
        ..TranscriptionPreferences::default()
    };
    bed.transcription
        .update_preferences(preferences)
        .await
        .unwrap();
    let status = bed
        .transcription
        .remove_file(&model_target("french-distil.bin"))
        .await
        .unwrap();
    assert_eq!(
        status.preferences.model, None,
        "a removed choice falls back to automatic"
    );
    assert!(
        !bed.model_files
            .exists(&models_dir().join("french-distil.bin"))
    );
}

#[tokio::test]
async fn transcribing_stores_the_transcript_and_records_its_summary_in_the_asset() {
    let bed = started().await;
    let (project_id, asset_id) = project_with_video(&bed).await;
    assert_eq!(
        code(
            bed.transcription
                .transcribe_assets(project_id, &[asset_id])
                .await
                .unwrap_err()
        ),
        ErrorCode::NoTranscriptionModel
    );
    install(&bed, "base");
    install(&bed, "large-v3-turbo");
    bed.transcription.status().await.unwrap();
    let settings = TranscriptionSettings {
        language: Some("en".to_owned()),
        vocabulary: "  SimpleEffect, Dimzou ".to_owned(),
    };
    bed.transcription
        .set_project_settings(project_id, settings)
        .await
        .unwrap();

    let jobs = bed
        .transcription
        .transcribe_assets(project_id, &[asset_id])
        .await
        .unwrap();
    assert_eq!(jobs.len(), 1);
    settle().await;

    let request = bed.transcriber.requests().pop().unwrap();
    assert_eq!(
        request.model.id, "base",
        "the best installed model (small is not)"
    );
    assert_eq!(request.model.alignment.as_deref(), Some("base"));
    assert_eq!(request.language.as_deref(), Some("en"));
    assert_eq!(request.vocabulary.as_deref(), Some("SimpleEffect, Dimzou"));
    assert_eq!(
        request.voice_model,
        Some(models_dir().join(VOICE_MODEL.file_name))
    );
    assert!(request.audio.starts_with(&request.work_dir));
    let (source, output) = bed.extractor.extracted().pop().unwrap();
    assert!(source.ends_with("interview.mp4"));
    assert_eq!(output, request.audio);

    let transcript = bed
        .transcription
        .transcript(project_id, asset_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(transcript.words.len(), 5);
    let assets = bed.assets(project_id).await.unwrap();
    let summary = assets[0].analysis.transcript.clone().unwrap();
    assert_eq!((summary.word_count, summary.model.as_str()), (5, "base"));
    let status = bed.transcription.status().await.unwrap();
    assert_eq!(status.last_backend, Some(TranscriptionBackend::Cpu));
}

#[tokio::test]
async fn imported_media_are_transcribed_when_the_preference_is_on() {
    let bed = started().await;
    install(&bed, "tiny");
    let preferences = TranscriptionPreferences {
        auto_transcribe: true,
        voice_detection: false,
        ..TranscriptionPreferences::default()
    };
    bed.transcription
        .update_preferences(preferences)
        .await
        .unwrap();
    let (project_id, asset_id) = project_with_video(&bed).await;
    settle().await;
    let request = bed
        .transcriber
        .requests()
        .pop()
        .expect("transcribed after import");
    assert_eq!(request.voice_model, None, "voice detection is off");
    assert_eq!(
        request.language.as_deref(),
        Some("fr"),
        "the project's default"
    );
    assert!(
        bed.transcription
            .transcript(project_id, asset_id)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn a_sequence_shows_the_words_of_its_sound_clips_and_names_untranscribed_media() {
    let bed = started().await;
    install(&bed, "base");
    bed.transcription.status().await.unwrap();
    let (project_id, asset_id) = project_with_video(&bed).await;
    let session = bed.projects.project_session(project_id).await.unwrap();
    let sequence_id = session.snapshot.sequences[0].id;
    bed.projects
        .place_media(&PlaceMediaRequest {
            project_id,
            sequence_id,
            asset_ids: vec![asset_id],
            start: Flicks::from_seconds_f64(10.0).unwrap(),
            video_track: TrackTarget::New,
            audio_track: TrackTarget::New,
            mode: PlacementMode::Overwrite,
        })
        .await
        .unwrap();
    let before = bed
        .transcription
        .sequence_words(project_id, sequence_id)
        .await
        .unwrap();
    assert_eq!(before.transcript.words.len(), 0);
    assert_eq!(before.untranscribed, [asset_id]);

    bed.transcription
        .transcribe_assets(project_id, &[asset_id])
        .await
        .unwrap();
    settle().await;
    let after = bed
        .transcription
        .sequence_words(project_id, sequence_id)
        .await
        .unwrap();
    let words: Vec<&str> = after
        .transcript
        .words
        .iter()
        .map(|word| word.text.as_str())
        .collect();
    assert_eq!(words, ["Bonjour", "à", "tous.", "Ça", "va ?"]);
    assert_eq!(
        after.transcript.words[0].start,
        Flicks::from_seconds_f64(10.0).unwrap()
    );
    assert_eq!(after.untranscribed.len(), 0);

    let hits = bed
        .transcription
        .search(project_id, "bonjour TOUS")
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].text, "Bonjour à tous.");
    assert_eq!(
        bed.transcription
            .search(project_id, "  ")
            .await
            .unwrap()
            .len(),
        0
    );
}

#[tokio::test]
async fn project_settings_are_one_undoable_edit_and_refuse_malformed_languages() {
    let bed = started().await;
    let (project_id, _) = project_with_video(&bed).await;
    let settings = TranscriptionSettings {
        language: None,
        vocabulary: String::new(),
    };
    bed.transcription
        .set_project_settings(project_id, settings.clone())
        .await
        .unwrap();
    let session = bed.projects.project_session(project_id).await.unwrap();
    assert_eq!(session.snapshot.project.settings.transcription, settings);
    bed.projects.undo(project_id).await.unwrap();
    let session = bed.projects.project_session(project_id).await.unwrap();
    assert_eq!(
        session.snapshot.project.settings.transcription,
        TranscriptionSettings::default()
    );
    let refused = bed
        .transcription
        .set_project_settings(
            project_id,
            TranscriptionSettings {
                language: Some("French".to_owned()),
                vocabulary: String::new(),
            },
        )
        .await
        .unwrap_err();
    assert_eq!(code(refused), ErrorCode::EditRejected);
}

#[tokio::test]
async fn the_gpu_pack_installs_and_uninstalls_through_the_transcriber() {
    let bed = started().await;
    assert_eq!(
        code(
            bed.transcription
                .start_download(&DownloadTarget::GpuRuntime)
                .unwrap_err()
        ),
        ErrorCode::TranscriptionTargetNotFound
    );
    let mut environment = se_app::ports::Transcriber::environment(&bed.transcriber).await;
    environment.gpu_name = Some("NVIDIA GeForce RTX 3080".to_owned());
    environment.gpu_runtime = Some(fake_gpu_offer(false));
    bed.transcriber.set_environment(environment);
    bed.transcription
        .update_preferences(TranscriptionPreferences::default())
        .await
        .unwrap();
    bed.transcription
        .start_download(&DownloadTarget::GpuRuntime)
        .unwrap();
    settle().await;
    let status = bed.transcription.status().await.unwrap();
    assert!(status.gpu_runtime.as_ref().unwrap().installed);
    assert_eq!(status.backend, TranscriptionBackend::Cuda);
    let status = bed
        .transcription
        .remove_file(&DownloadTarget::GpuRuntime)
        .await
        .unwrap();
    assert!(!status.gpu_runtime.unwrap().installed);
    assert_eq!(status.backend, TranscriptionBackend::Cpu);
}
