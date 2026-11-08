//! Where transcriptions stand and transcribing on an agent's behalf (ADR-0034), against the
//! in-memory ports: not started, started then awaited (the wait gives up at its timeout while
//! the job keeps running), transcribed, never started twice, and the errors of a missing model.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{path::Path, time::Duration};

use se_app::{
    test_support::{FakeModelFile, MediaTestbed, TESTBED_MODELS_DIR, probed_video},
    transcription::{MODELS, TranscriptionState, VOICE_MODEL},
};
use se_domain::{AssetId, ProjectId};
use se_protocol::{ErrorCode, ProtocolError};

const PARENT: &str = "/films";

async fn started() -> MediaTestbed {
    let bed = MediaTestbed::new(Path::new(PARENT));
    bed.transcription.start().await.unwrap();
    bed
}

/// Installs catalogue model `id` (and the voice model) as if downloaded.
async fn install(bed: &MediaTestbed, id: &str) {
    let model = MODELS.iter().find(|model| model.id == id).unwrap();
    for file in [model.file, VOICE_MODEL] {
        bed.model_files.put(
            &Path::new(TESTBED_MODELS_DIR).join(file.file_name),
            FakeModelFile {
                size: file.size,
                sha256: file.sha256.to_owned(),
                is_model: true,
            },
        );
    }
    bed.transcription.status().await.unwrap();
}

async fn project_with_video(bed: &MediaTestbed) -> (ProjectId, AssetId) {
    let project_id = bed.create_project(Path::new(PARENT), "Film").await.unwrap();
    let path = Path::new(PARENT).join("rushes").join("interview.mp4");
    bed.files.add(path.clone(), 1, probed_video());
    let report = bed.media.import_files(project_id, &[path]).await.unwrap();
    (project_id, report.imported[0])
}

#[tokio::test(start_paused = true)]
async fn an_agent_starts_a_transcription_and_waits_for_it() {
    let bed = started().await;
    let (project_id, asset_id) = project_with_video(&bed).await;
    let states = bed
        .transcription
        .transcription_states(project_id, &[asset_id])
        .await
        .unwrap();
    assert_eq!(states[0].state, TranscriptionState::NotStarted);

    install(&bed, "base").await;
    bed.transcriber.hold();
    let round = bed
        .transcription
        .transcribe_and_wait(project_id, &[asset_id], Duration::from_secs(5))
        .await
        .unwrap();
    assert_eq!(round.started, [asset_id]);
    assert!(!round.finished(), "the wait gave up at its timeout");
    assert!(matches!(
        round.assets[0].state,
        TranscriptionState::Running { .. }
    ));

    let again = bed
        .transcription
        .transcribe_and_wait(project_id, &[asset_id], Duration::ZERO)
        .await
        .unwrap();
    assert!(
        again.started.is_empty(),
        "a running transcription is not started twice"
    );

    bed.transcriber.release();
    let states = bed
        .transcription
        .wait_for_transcriptions(project_id, &[asset_id], Duration::from_secs(5))
        .await
        .unwrap();
    assert_eq!(
        states[0].state,
        TranscriptionState::Transcribed { word_count: 5 }
    );
    assert_eq!(bed.transcriber.requests().len(), 1);
}

#[tokio::test]
async fn without_a_model_nothing_starts_and_the_error_says_so() {
    let bed = started().await;
    let (project_id, asset_id) = project_with_video(&bed).await;
    let error = bed
        .transcription
        .transcribe_and_wait(project_id, &[asset_id], Duration::ZERO)
        .await
        .unwrap_err();
    assert_eq!(
        ProtocolError::from(error).code,
        ErrorCode::NoTranscriptionModel
    );
    let unknown = AssetId::from_test_index(999);
    let error = bed
        .transcription
        .transcription_states(project_id, &[unknown])
        .await
        .unwrap_err();
    assert_eq!(ProtocolError::from(error).code, ErrorCode::AssetNotFound);
}
