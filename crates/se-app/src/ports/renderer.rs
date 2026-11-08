//! [`RendererLink`]: how the core reaches the renderer, which alone can composite frames of a
//! sequence (the engine lives there, golden rule 7; ADR-0028).

use se_protocol::FrameRenderRequest;

/// The connection to the renderer (implemented by se-core over its event bus).
pub trait RendererLink: Send + Sync {
    /// Whether a renderer is connected (an editor window is open).
    fn is_connected(&self) -> bool;

    /// Asks the renderer to draw and upload `request`'s frames, without waiting.
    fn request_frames(&self, request: FrameRenderRequest);
}
