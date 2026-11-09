//! # se-vision
//!
//! Vision adapter (prompt 26, ADR-0037): finds faces in sampled pictures with the YuNet model
//! (OpenCV Zoo, MIT, embedded in the binary) over ONNX Runtime, loaded at runtime from
//! `resources/bin/<platform>/onnxruntime/` (`pnpm fetch-binaries`). It implements the
//! `SubjectDetector` port of `se-app`; turning faces into reframing keyframes is the domain's job
//! (`se_domain::tracking`).
//!
//! - [`YuNetDetector`]: the port, one ONNX Runtime session reused on the blocking pool.
//! - [`RuntimeLocator`]: where the ONNX Runtime library is looked for.
//! - [`yunet`]: the model, its input tensor and the decoding of its outputs (pure).
//! - [`platform`]: library names, execution providers, unsupported systems.
//!
//! Allowed workspace dependencies: `se-app`, `se-domain`, `se-protocol`; never another adapter
//! crate. No network: the model is in the binary and pictures stay in memory.

mod detector;
pub mod platform;
mod runtime;
pub mod yunet;

pub use detector::YuNetDetector;
pub use runtime::{LoadedRuntime, RuntimeLocator};
