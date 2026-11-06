//! # se-vision
//!
//! Vision adapter: face and subject detection with ONNX Runtime, used by automatic reframing.
//!
//! It implements the `SubjectDetector` port of `se-app`.
//! Allowed workspace dependencies: `se-app`, `se-domain`; never another adapter crate.
//! Models are stored locally and downloaded only with explicit user approval.
//!
//! Implemented by roadmap prompt 26.
