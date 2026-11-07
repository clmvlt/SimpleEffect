import type { Asset, Flicks, ItemId, PlacementMode } from '@se/protocol';
import { useRef, type DragEvent } from 'react';

import { draggedAssetIds, endAssetDrag, isAssetDrag } from '../../../shared/drag';
import { useShortcutRegistry } from '../../../shared/shortcuts';
import { snapToFrame } from '../../../shared/time';
import { dropGhosts, dropTargets, type DropTargets } from '../services/drop-placement';
import type { GesturePreview, PreviewGhost } from '../services/preview';
import { concernedTrackIds, shiftedPlacements } from '../services/ripple-preview';
import { rowAt } from '../services/track-layout';
import { xToTime } from '../services/viewport';
import { clearGesturePreview, setGesturePreview } from '../store/gesture-store';
import { selectItems } from '../store/selection-store';
import { snapEdges, type GestureContext } from './gestures/gesture-context';

/** Where a drop lands, as computed during `dragover` and sent on `drop`. */
interface DropPlan {
  readonly assets: readonly Asset[];
  readonly start: Flicks;
  readonly targets: DropTargets;
  readonly mode: PlacementMode;
  readonly preview: GesturePreview;
}

/** What follows the drop point on the tracks receiving existing parts, drawn pushed (insert). */
function insertPreview(context: GestureContext, ghosts: readonly PreviewGhost[], start: Flicks) {
  const { tracks, itemsByTrack } = context.model;
  const seeds = new Set(
    ghosts.flatMap((ghost) => (ghost.row.kind === 'track' ? [ghost.row.trackId] : [])),
  );
  const end = Math.max(...ghosts.map((ghost) => ghost.start + ghost.duration));
  const concerned = concernedTrackIds(tracks, itemsByTrack, seeds, start);
  return shiftedPlacements(itemsByTrack, concerned, start, (end - start) as Flicks);
}

/**
 * Dropping media bin assets on the lanes: a ghost of the clips follows the pointer (frame
 * aligned, snapping their edges) on the track under it — below the last track, a new one — and
 * the drop sends one `placeMedia` command (Ctrl/⌘: insert, pushing what follows).
 */
export function useMediaDrop(context: () => GestureContext) {
  const { platform } = useShortcutRegistry();
  const plan = useRef<DropPlan | null>(null);

  const planDrop = (event: DragEvent<HTMLElement>): DropPlan | null => {
    const current = context();
    const { model, layout, viewport } = current;
    const assets = draggedAssetIds(event.dataTransfer)
      .map((id) => model.assets[id])
      .filter((asset): asset is Asset => asset !== undefined);
    if (assets.length === 0) return null;
    const bounds = event.currentTarget.getBoundingClientRect();
    const [x, y] = [event.clientX - bounds.left, event.clientY - bounds.top];
    const rate = model.sequence.format.frameRate;
    const pointed = snapToFrame(xToTime(x, viewport), rate, 'nearest');
    const row = rowAt(layout, y);
    const targets = dropTargets(model.tracks, y >= layout.height ? null : (row?.track ?? null));
    const unsnapped = dropGhosts(assets, pointed, targets);
    const blockEnd = Math.max(...unsnapped.map((ghost) => ghost.start + ghost.duration));
    const snap = snapEdges(current, [pointed, blockEnd as Flicks], []);
    const start = Math.max(0, pointed + (snap?.delta ?? 0)) as Flicks;
    const ghosts = dropGhosts(assets, start, targets);
    const insert = platform === 'apple' ? event.metaKey : event.ctrlKey;
    const placements = insert ? insertPreview(current, ghosts, start) : new Map();
    const snapTime = snap?.point ?? null;
    const mode = insert ? 'insert' : 'overwrite';
    return { assets, start, targets, mode, preview: { placements, ghosts, snapTime } };
  };

  const onDragOver = (event: DragEvent<HTMLDivElement>) => {
    if (!isAssetDrag(event.dataTransfer)) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = 'copy';
    plan.current = planDrop(event);
    if (plan.current === null) clearGesturePreview();
    else setGesturePreview(plan.current.preview);
  };

  const onDragLeave = (event: DragEvent<HTMLDivElement>) => {
    const next = event.relatedTarget;
    if (next instanceof Node && event.currentTarget.contains(next)) return;
    plan.current = null;
    clearGesturePreview();
  };

  const onDrop = async (event: DragEvent<HTMLDivElement>) => {
    if (!isAssetDrag(event.dataTransfer)) return;
    event.preventDefault();
    const drop = planDrop(event) ?? plan.current;
    plan.current = null;
    endAssetDrag();
    if (drop === null) {
      clearGesturePreview();
      return;
    }
    const { model, commands } = context();
    const itemIds: ItemId[] | null = await commands.placeMedia({
      sequenceId: model.sequence.id,
      assetIds: drop.assets.map((asset) => asset.id),
      start: drop.start,
      videoTrack: drop.targets.video,
      audioTrack: drop.targets.audio,
      mode: drop.mode,
    });
    if (itemIds !== null) selectItems(itemIds);
    clearGesturePreview();
  };

  return {
    onDragOver,
    onDragLeave,
    onDrop: (event: DragEvent<HTMLDivElement>) => {
      void onDrop(event);
    },
  };
}
