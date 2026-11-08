// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CaptionSegmentation } from "./CaptionSegmentation";
import type { CaptionStyleChoice } from "./CaptionStyleChoice";
import type { ProjectId } from "./ProjectId";
import type { SequenceId } from "./SequenceId";
import type { TimeRange } from "./TimeRange";

/**
 * Captions to make from the transcript of a sequence range.
 */
export type GenerateCaptionsRequest = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence.
 */
sequenceId: SequenceId, 
/**
 * Where (`null`: the whole sequence). Captions of the caption track crossing it are replaced
 * (the range grows to cover them whole).
 */
range?: TimeRange, 
/**
 * The style (`null`: the style of the captions replaced, else the `minimal` preset).
 */
style?: CaptionStyleChoice, 
/**
 * Segmentation overrides.
 */
segmentation?: CaptionSegmentation, 
/**
 * Name of the caption track when one is created (`null`: « Sous-titres »).
 */
trackName?: string, };
