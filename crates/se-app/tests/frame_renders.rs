//! The frame render service against a fake renderer connection: requests go out, uploads answer
//! them in order, failures and timeouts reach the caller, and late uploads are refused.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{sync::Arc, time::Duration};

use se_app::{
    AppError, FrameRenderService, FrameRenderSpec,
    test_support::{FakeRendererLink, SequentialIds},
};
use se_domain::{Flicks, ProjectId, SequenceId};

const JPEG: [u8; 4] = [0xFF, 0xD8, 0xFF, 0xE0];

fn jpeg(tag: u8) -> Vec<u8> {
    let mut bytes = JPEG.to_vec();
    bytes.push(tag);
    bytes
}

fn service(link: &Arc<FakeRendererLink>) -> Arc<FrameRenderService> {
    Arc::new(FrameRenderService::with_timeout(
        link.clone(),
        Arc::new(SequentialIds::default()),
        Duration::from_secs(5),
    ))
}

fn spec(count: i64) -> FrameRenderSpec {
    FrameRenderSpec {
        project_id: ProjectId::from_test_index(1),
        sequence_id: SequenceId::from_test_index(2),
        times: (0..count)
            .map(|second| Flicks::new(second * Flicks::PER_SECOND).unwrap())
            .collect(),
        width: 768,
        height: 432,
    }
}

/// Waits until the fake renderer received a request, then returns its id.
async fn next_request(link: &FakeRendererLink) -> String {
    for _ in 0..100 {
        if let Some(request) = link.requests().last() {
            return request.request_id.clone();
        }
        tokio::task::yield_now().await;
    }
    panic!("no frame request reached the renderer");
}

#[tokio::test]
async fn uploads_in_any_order_answer_the_request_in_time_order() {
    let link = Arc::new(FakeRendererLink::default());
    let renders = service(&link);
    let waiting = tokio::spawn({
        let renders = renders.clone();
        async move { renders.render(spec(3)).await }
    });
    let id = next_request(&link).await;
    let request = link.requests().remove(0);
    assert_eq!(request.times.len(), 3);
    assert_eq!((request.width, request.height), (768, 432));
    renders.deliver(&id, 2, jpeg(2)).unwrap();
    renders.deliver(&id, 0, jpeg(0)).unwrap();
    assert!(matches!(
        renders.deliver(&id, 3, jpeg(3)),
        Err(AppError::InvalidRender { .. })
    ));
    assert!(matches!(
        renders.deliver(&id, 1, b"not a jpeg".to_vec()),
        Err(AppError::InvalidRender { .. })
    ));
    renders.deliver(&id, 1, jpeg(1)).unwrap();
    let frames = waiting.await.unwrap().unwrap();
    assert_eq!(frames, vec![jpeg(0), jpeg(1), jpeg(2)]);
    assert_eq!(renders.pending_count(), 0);
    assert!(matches!(
        renders.deliver(&id, 0, jpeg(0)),
        Err(AppError::RenderRequestNotFound { .. })
    ));
}

#[tokio::test]
async fn a_reported_failure_reaches_the_caller() {
    let link = Arc::new(FakeRendererLink::default());
    let renders = service(&link);
    let waiting = tokio::spawn({
        let renders = renders.clone();
        async move { renders.render(spec(2)).await }
    });
    let id = next_request(&link).await;
    renders.fail(&id, "another project is open").unwrap();
    let error = waiting.await.unwrap().unwrap_err();
    assert!(matches!(&error, AppError::RenderFailed { reason } if reason.contains("another")));
    assert!(matches!(
        renders.fail(&id, "again"),
        Err(AppError::RenderRequestNotFound { .. })
    ));
}

#[tokio::test(start_paused = true)]
async fn a_silent_renderer_times_out_and_is_forgotten() {
    let link = Arc::new(FakeRendererLink::default());
    let renders = service(&link);
    let error = renders.render(spec(1)).await.unwrap_err();
    assert!(
        matches!(&error, AppError::RendererUnavailable { reason } if reason.contains("5 s")),
        "{error}"
    );
    assert_eq!(renders.pending_count(), 0);
}

#[tokio::test]
async fn without_a_window_nothing_is_requested() {
    let link = Arc::new(FakeRendererLink::default());
    link.set_connected(false);
    let renders = service(&link);
    let error = renders.render(spec(1)).await.unwrap_err();
    assert!(matches!(error, AppError::RendererUnavailable { .. }));
    assert_eq!(link.requests(), Vec::new());
    assert_eq!(
        renders.render(spec(0)).await.unwrap(),
        Vec::<Vec<u8>>::new()
    );
}

#[tokio::test]
async fn too_many_frames_are_refused_and_a_dropped_caller_is_forgotten() {
    let link = Arc::new(FakeRendererLink::default());
    let renders = service(&link);
    assert!(matches!(
        renders.render(spec(17)).await,
        Err(AppError::InvalidRender { .. })
    ));
    let waiting = tokio::spawn({
        let renders = renders.clone();
        async move { renders.render(spec(1)).await }
    });
    let id = next_request(&link).await;
    waiting.abort();
    let _ = waiting.await;
    assert_eq!(renders.pending_count(), 0);
    assert!(matches!(
        renders.deliver(&id, 0, jpeg(0)),
        Err(AppError::RenderRequestNotFound { .. })
    ));
}
