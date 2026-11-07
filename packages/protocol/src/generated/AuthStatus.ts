// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Whether the provider's CLI has a logged-in session. The app never reads, stores or forwards
 * credentials: it only asks the CLI.
 */
export type AuthStatus = { "kind": "loggedIn", 
/**
 * The kind of account when the CLI tells it (`max`, `pro`, `api key`…), never an
 * identity (no e-mail, no organisation).
 */
accountKind: string | null, } | { "kind": "loggedOut" } | { "kind": "unknown", 
/**
 * Why.
 */
reason: string, };
