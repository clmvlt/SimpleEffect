// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CaptionStyleChoice } from "./CaptionStyleChoice";
import type { ProjectId } from "./ProjectId";
import type { SequenceId } from "./SequenceId";

/**
 * An SRT file to turn into captions.
 */
export type ImportSubtitlesRequest = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence (cue times are sequence times).
 */
sequenceId: SequenceId, 
/**
 * Absolute path of the `.srt` file.
 */
path: string, 
/**
 * The style (`null`: as for generation).
 */
style?: CaptionStyleChoice, 
/**
 * Name of the caption track when one is created.
 */
trackName?: string, };
