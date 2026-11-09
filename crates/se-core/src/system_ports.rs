//! Production implementations of the small se-app ports: wall clock, UUIDv7 ids, task tokens,
//! and the event bus as an event sink and as the link to the renderer.

use std::time::{SystemTime, UNIX_EPOCH};

use se_app::ports::{Clock, EventSink, IdGenerator, RendererLink, TokenSource};
use se_domain::Uuid;
use se_protocol::{CoreEvent, FrameRenderRequest, TemplateCheckRequest};

use crate::rpc::EventBus;

/// The system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_millis(&self) -> i64 {
        // A clock set before 1970 reads as the epoch rather than failing.
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| {
                i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
            })
    }
}

/// Random, time-ordered UUIDv7 ids.
#[derive(Debug, Clone, Copy, Default)]
pub struct UuidV7Ids;

impl IdGenerator for UuidV7Ids {
    fn new_uuid_v7(&self) -> Uuid {
        Uuid::now_v7()
    }
}

/// Number of random bytes in a task token (256 bits).
const TASK_TOKEN_BYTES: usize = 32;

/// Task tokens from the thread-local CSPRNG, as lowercase hex.
#[derive(Debug, Clone, Copy, Default)]
pub struct RandomTokens;

impl TokenSource for RandomTokens {
    fn new_token(&self) -> String {
        hex::encode(rand::random::<[u8; TASK_TOKEN_BYTES]>())
    }
}

impl EventSink for EventBus {
    fn publish(&self, event: CoreEvent) {
        EventBus::publish(self, event);
    }
}

/// The renderer is reached through the `/rpc` sockets: connected while one is open.
impl RendererLink for EventBus {
    fn is_connected(&self) -> bool {
        self.connection_count() > 0
    }

    fn request_frames(&self, request: FrameRenderRequest) {
        EventBus::publish(self, CoreEvent::RenderFramesRequested(request));
    }

    fn request_template_check(&self, request: TemplateCheckRequest) {
        EventBus::publish(self, CoreEvent::TemplateCheckRequested(request));
    }
}

#[cfg(test)]
mod tests {
    use se_domain::ItemId;

    use super::*;

    #[test]
    fn ids_are_valid_uuid_v7() {
        let uuid = UuidV7Ids.new_uuid_v7();
        assert!(ItemId::from_uuid(uuid).is_ok());
        assert_ne!(uuid, UuidV7Ids.new_uuid_v7());
    }

    #[test]
    fn task_tokens_are_256_random_bits() {
        let token = RandomTokens.new_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|character| character.is_ascii_hexdigit()));
        assert_ne!(token, RandomTokens.new_token());
    }

    #[test]
    fn the_renderer_is_connected_while_a_socket_listens() {
        let bus = EventBus::new();
        assert!(!bus.is_connected());
        let mut socket = bus.subscribe();
        assert!(bus.is_connected());
        bus.request_frames(FrameRenderRequest {
            request_id: "r".to_owned(),
            project_id: se_domain::ProjectId::from_test_index(1),
            sequence_id: se_domain::SequenceId::from_test_index(1),
            times: Vec::new(),
            width: 2,
            height: 2,
        });
        assert!(matches!(
            socket.try_recv(),
            Ok(CoreEvent::RenderFramesRequested(_))
        ));
    }

    #[test]
    fn the_clock_reads_after_2025() {
        assert!(SystemClock.now_unix_millis() > 1_735_689_600_000);
    }
}
