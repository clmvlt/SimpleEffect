//! # se-protocol
//!
//! Data transfer objects exchanged between `se-core` and its clients:
//! - the startup handshake line printed by `se-core` on stdout ([`CoreHandshake`]);
//! - the JSON messages of the `/rpc` WebSocket ([`ClientMessage`], [`ServerMessage`]).
//!
//! Allowed workspace dependencies: `se-domain` (see `docs/architecture.md` §3 and §6).
//! DTOs only: no logic beyond (de)serialisation.
//!
//! The types are hand-written for now. Roadmap prompt 02 derives `ts_rs::TS` on them and
//! generates `@se/protocol` from this crate (ADR-0010).

mod handshake;
mod rpc;

pub use handshake::CoreHandshake;
pub use rpc::{
    ClientMessage, Command, CommandResult, ErrorCode, ProtocolError, RequestId, Response,
    ResponseOutcome, ServerMessage,
};
