# 0010 — Typed protocol with Rust as the source of truth

- Status: Accepted
- Date: 2026-11-06

## Context
The renderer (TypeScript) and se-core (Rust) exchange commands, results, events and the full document model. Hand-written duplicate types drift and confuse AI maintainers.

## Decision
- Domain types (`se-domain`) and protocol types (`se-protocol`) derive `ts_rs::TS`. `pnpm codegen` exports them to `packages/protocol/src/generated/` (committed, never hand-edited).
- Transport is a single WebSocket at `/rpc` with JSON messages: `request` / `response` (correlated by id) and `event` (server push). `Command`, `CommandResult`, `Event`, `ProtocolError` are internally tagged enums (`kind`).
- `@se/protocol` exposes a typed `CoreClient`: `send<C extends Command>(command): Promise<ResultFor<C>>`, `subscribe(eventKind, handler)`, automatic reconnect with snapshot resync.
- Errors have stable codes and actionable messages.
- A CI check (`pnpm codegen --check`) fails if generated files are stale.

## Consequences
- One definition per type; compile errors on both sides when the contract changes.
- Large integers: flicks are serialized as JSON numbers and stay exact in JS up to ±2^53 (see ADR-0004).

## Alternatives considered
- **specta / tauri-specta**: tied to Tauri for command typing; we do not use Tauri. ts-rs is simpler for plain types.
- **JSON Schema / OpenAPI first**: an extra source of truth and tooling. Rejected.
- **gRPC / protobuf**: heavier tooling, poor fit for WebSocket push in the renderer. Rejected.
