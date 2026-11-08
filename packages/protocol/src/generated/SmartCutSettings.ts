// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { FillerList } from "./FillerList";
import type { Flicks } from "./Flicks";
import type { SilenceCutParams } from "./SilenceCutParams";

/**
 * The app's smart-cut settings (stored in `library.db`, edited in the transcript panel).
 */
export type SmartCutSettings = { 
/**
 * Filler words per language; a transcript in a language without a list uses every list.
 */
fillers: Array<FillerList>, 
/**
 * Longest filler occurrence detected (longer ones are probably mistimed words).
 */
fillerMaxDuration: Flicks, 
/**
 * Defaults of the silence removal dialog and of agents' `remove_silences`.
 */
silence: SilenceCutParams, };
