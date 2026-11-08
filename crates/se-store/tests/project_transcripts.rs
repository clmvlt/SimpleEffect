//! Transcripts in a real `project.db`, through `FsProjectStore::transcripts()`: they survive a
//! reopening, a new transcription replaces the old one (rows and index), and full-text search
//! ignores accents and case.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use se_app::ports::{ProjectRepository, StoredTranscript};
use se_domain::{
    AssetId, Flicks, Transcript, TranscriptSegment, TranscriptWord,
    test_support::{example_project, millis},
};
use se_store::FsProjectRepository;

fn word(text: &str, start_ms: i64, end_ms: i64, segment: u32) -> TranscriptWord {
    TranscriptWord {
        text: text.to_owned(),
        start: millis(start_ms),
        end: millis(end_ms),
        confidence: 0.875,
        segment,
    }
}

fn stored(asset: u64, model: &str, words: Vec<TranscriptWord>) -> StoredTranscript {
    let segment_count = words.iter().map(|word| word.segment + 1).max().unwrap_or(0);
    let segments = (0..segment_count)
        .map(|index| {
            let inside: Vec<&TranscriptWord> =
                words.iter().filter(|word| word.segment == index).collect();
            TranscriptSegment {
                start: inside[0].start,
                end: inside[inside.len() - 1].end,
            }
        })
        .collect();
    StoredTranscript {
        asset_id: AssetId::from_test_index(asset),
        content_key: format!("key-{asset}"),
        created_at: 1_789_000_000_000,
        backend: "cuda".to_owned(),
        transcript: Transcript {
            language: "fr".to_owned(),
            model: model.to_owned(),
            words,
            segments,
        },
    }
}

#[tokio::test]
async fn transcripts_survive_reopening_are_replaced_and_searchable_without_accents() {
    let parent = tempfile::tempdir().unwrap();
    let repository = FsProjectRepository;
    let path = repository
        .create(parent.path(), &example_project())
        .await
        .unwrap();
    let store = repository.open(&path).await.unwrap();
    let transcripts = store.transcripts();
    let first = stored(
        1,
        "base",
        vec![
            word("Bonjour", 0, 400, 0),
            word("à", 400, 500, 0),
            word("tous.", 500, 900, 0),
            word("Abonnez-vous", 1_500, 2_200, 1),
            word("à", 2_200, 2_300, 1),
            word("la", 2_300, 2_400, 1),
            word("chaîne !", 2_400, 3_000, 1),
        ],
    );
    let other = stored(
        2,
        "base",
        vec![word("Réalisé", 0, 600, 0), word("hier.", 600, 900, 0)],
    );
    transcripts.save_transcript(&first).await.unwrap();
    transcripts.save_transcript(&other).await.unwrap();
    drop(transcripts);
    drop(store);

    let store = repository.open(&path).await.unwrap();
    let transcripts = store.transcripts();
    assert_eq!(
        transcripts.load_transcript(first.asset_id).await.unwrap(),
        Some(first.clone())
    );
    assert_eq!(
        transcripts
            .load_transcript(AssetId::from_test_index(9))
            .await
            .unwrap(),
        None
    );

    let hits = transcripts
        .search_transcripts("chaine abonn", 10)
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].text, "Abonnez-vous à la chaîne !");
    assert_eq!((hits[0].segment, hits[0].start), (1, millis(1_500)));
    let hits = transcripts.search_transcripts("REALISE", 10).await.unwrap();
    assert_eq!(hits[0].asset_id, other.asset_id);
    assert_eq!(
        transcripts
            .search_transcripts("\" OR *", 10)
            .await
            .unwrap()
            .len(),
        0
    );

    let replacement = stored(1, "large-v3-turbo", vec![word("Salut", 0, 300, 0)]);
    transcripts.save_transcript(&replacement).await.unwrap();
    let loaded = transcripts
        .load_transcript(first.asset_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(loaded.transcript.model, "large-v3-turbo");
    assert_eq!(loaded.transcript.words.len(), 1);
    assert_eq!(
        transcripts
            .search_transcripts("chaine", 10)
            .await
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        transcripts
            .search_transcripts("salut", 10)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(loaded.transcript.words[0].start, Flicks::ZERO);
    store.close().await.unwrap();
}
