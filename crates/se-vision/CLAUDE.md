# se-vision

## Purpose
Vision adapter: face and subject detection with ONNX Runtime, used by automatic reframing. It implements the `SubjectDetector` port of `se-app`; turning detections into keyframes is a use case, not this crate's job.

Currently an empty skeleton, implemented by prompt 26.

## Public API
None yet. Planned: a `SubjectDetector` implementation.

## Rules and invariants
- Models are stored locally and downloaded only after explicit user approval.
- Inference is CPU/GPU heavy: run it in `spawn_blocking` or a dedicated thread pool.

## Allowed dependencies
`se-app`, `se-domain`, ONNX Runtime bindings. Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-vision`
- Post-processing (boxes, tracking) is unit-tested on fixtures without a model.
