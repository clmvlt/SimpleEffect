import type { Flicks, ItemEntity, ItemId, PlacementMode } from '@se/protocol';

import { showSidePanelTab } from '../../../shared/side-panel';
import { frameDuration } from '../../../shared/time';
import { revealAsset } from '../../media-bin';
import { clipboardOf, pastedItems } from '../services/clipboard';
import { isGapOf } from '../services/gaps';
import {
  canLink,
  canUnlink,
  duplicatePlacements,
  itemsToSplit,
  nudgePlacements,
} from '../services/item-edits';
import { clipboardFor, setClipboard } from '../store/clipboard-store';
import { clearSelection, selectionStore, selectItems } from '../store/selection-store';
import type { TimelineCommands } from './use-timeline-commands';
import type { TimelineModel } from './use-timeline-model';

/** The item edits shared by the keyboard shortcuts and the context menu, each one command. */
export interface EditActions {
  /** Cuts `itemIds` (default: the selected items, or everything unlocked) crossing `time`. */
  readonly splitAt: (time: Flicks, itemIds?: readonly ItemId[]) => void;
  /** Cuts at the playhead (`S`, Ctrl/⌘+K). */
  readonly splitAtPlayhead: () => void;
  /** Deletes the selected items (with `ripple`, closing their time) or closes the selected gap. */
  readonly deleteSelection: (ripple: boolean) => void;
  readonly copySelection: () => void;
  readonly cutSelection: () => void;
  /** Pastes the clipboard at the playhead on the active track. */
  readonly paste: (mode: PlacementMode) => void;
  /** Moves the selected items by `frames` frames (`,` `.`, ten with Maj). */
  readonly nudge: (frames: number) => void;
  /** Copies the selected items right after themselves, pushing what follows. */
  readonly duplicateSelection: () => void;
  readonly linkSelection: () => void;
  readonly unlinkSelection: () => void;
  /** The selected items of the sequence (stale ids skipped). */
  readonly selectedItems: () => ItemEntity[];
  readonly revealInMediaBin: (assetId: string) => void;
  /** Opens the inspector on the selection (« Propriétés »). */
  readonly showProperties: () => void;
}

/** Selects `ids` once a command returned them (`null`: it failed, its toast is shown). */
function selectResult(ids: readonly ItemId[] | null): void {
  if (ids !== null) selectItems(ids);
}

/** The edit actions on the active sequence of `model`. */
export function useEditActions(
  model: TimelineModel,
  commands: TimelineCommands,
  playheadTime: () => Flicks,
): EditActions {
  const selectedItems = () => {
    const selected = new Set(selectionStore.getState().itemIds);
    return model.items.filter((item) => selected.has(item.id));
  };
  const ids = (items: readonly ItemEntity[]) => items.map((item) => item.id);
  const lockedTracks = new Set(model.tracks.filter((track) => track.locked).map((t) => t.id));

  const splitAt = (time: Flicks, itemIds?: readonly ItemId[]) => {
    const selected = new Set(ids(selectedItems()));
    const targets = itemIds ?? itemsToSplit(model.items, selected, lockedTracks, time);
    if (targets.length > 0) void commands.splitItems(targets, time);
  };

  const deleteSelection = (ripple: boolean) => {
    const { gap } = selectionStore.getState();
    if (gap !== null) {
      if (!isGapOf(model.itemsByTrack.get(gap.trackId) ?? [], gap)) return;
      void commands.resizeGap(gap, 0 as Flicks).then((done) => {
        if (done !== null) clearSelection();
      });
      return;
    }
    const items = selectedItems();
    if (items.length === 0) return;
    void commands.deleteItems(ids(items), ripple).then((done) => {
      if (done !== null) clearSelection();
    });
  };

  const copySelection = () => {
    const clipboard = clipboardOf(model.projectId, selectedItems(), model.tracks);
    if (clipboard !== null) setClipboard(clipboard);
  };

  return {
    splitAt,
    splitAtPlayhead: () => {
      splitAt(playheadTime());
    },
    deleteSelection,
    copySelection,
    cutSelection: () => {
      copySelection();
      deleteSelection(false);
    },
    paste: (mode) => {
      const clipboard = clipboardFor(model.projectId);
      if (clipboard === null) return;
      const { activeTrackId } = selectionStore.getState();
      const active = model.tracks.find((track) => track.id === activeTrackId) ?? null;
      const items = pastedItems(clipboard, model.tracks, playheadTime(), active);
      if (items.length > 0) void commands.pasteItems(items, mode).then(selectResult);
    },
    nudge: (frames) => {
      const frame = frameDuration(model.sequence.format.frameRate);
      const placements = nudgePlacements(selectedItems(), (frames * frame) as Flicks);
      if (placements !== null) void commands.moveItems(placements, 'overwrite');
    },
    duplicateSelection: () => {
      const placements = duplicatePlacements(selectedItems());
      if (placements.length > 0) {
        void commands.duplicateItems(placements, 'insert').then(selectResult);
      }
    },
    linkSelection: () => {
      const items = selectedItems();
      if (canLink(items)) void commands.linkItems(ids(items));
    },
    unlinkSelection: () => {
      const items = selectedItems();
      if (canUnlink(items)) void commands.unlinkItems(ids(items));
    },
    selectedItems,
    revealInMediaBin: revealAsset,
    showProperties: () => {
      showSidePanelTab('inspector');
    },
  };
}
