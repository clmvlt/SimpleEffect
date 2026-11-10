// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CodecSupport } from "./CodecSupport";
import type { ComponentVersion } from "./ComponentVersion";

/**
 * What the window knows about itself, added to a diagnostics archive (`exportDiagnostics`).
 */
export type ClientDiagnostics = { 
/**
 * Version of the app (`package.json`).
 */
appVersion: string, 
/**
 * Runtime components and their versions (Electron, Chromium, Node.js, the engine…).
 */
components: Array<ComponentVersion>, 
/**
 * The graphics card as WebGL names it, when the window can tell.
 */
gpu: string | null, 
/**
 * What WebCodecs can decode and encode on this machine.
 */
codecs: Array<CodecSupport>, 
/**
 * The interface language (`fr`).
 */
locale: string, };
