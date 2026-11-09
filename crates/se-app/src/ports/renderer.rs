//! [`RendererLink`]: how the core reaches the renderer, which alone can composite frames of a
//! sequence and run motion template code (the engine lives there, golden rule 7; ADR-0028,
//! ADR-0035).

use se_protocol::{FrameRenderRequest, TemplateCheckRequest};

/// The connection to the renderer (implemented by se-core over its event bus).
pub trait RendererLink: Send + Sync {
    /// Whether a renderer is connected (an editor window is open).
    fn is_connected(&self) -> bool;

    /// Asks the renderer to draw and upload `request`'s frames, without waiting.
    fn request_frames(&self, request: FrameRenderRequest);

    /// Asks the renderer to check a template module, without waiting (prompt 24).
    fn request_template_check(&self, request: TemplateCheckRequest);
}
