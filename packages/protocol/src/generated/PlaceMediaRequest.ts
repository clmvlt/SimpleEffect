// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { Flicks } from "./Flicks";
import type { PlacementMode } from "./PlacementMode";
import type { ProjectId } from "./ProjectId";
import type { SequenceId } from "./SequenceId";
import type { TrackTarget } from "./TrackTarget";

/**
 * Media dropped on a sequence (`placeMedia`).
 */
export type PlaceMediaRequest = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence receiving the clips.
 */
sequenceId: SequenceId, 
/**
 * The assets, placed one after the other in this order.
 */
assetIds: Array<AssetId>, 
/**
 * Start of the first clip (≥ 0).
 */
start: Flicks, 
/**
 * Where pictures go (video assets, images).
 */
videoTrack: TrackTarget, 
/**
 * Where sound goes (audio assets, the sound of video assets): a video with sound becomes
 * two linked items.
 */
audioTrack: TrackTarget, 
/**
 * What happens to items already there.
 */
mode: PlacementMode, };
