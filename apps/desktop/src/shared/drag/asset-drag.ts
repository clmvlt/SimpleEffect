/**
 * Dragging assets from the media bin to another panel (the timeline). The ids travel in the
 * drag data under {@link ASSET_DRAG_TYPE}; browsers hide drag data until the drop, so the ids are
 * also kept here while the drag lasts, for previews during `dragover`.
 */

/** Drag data type of media bin assets (a JSON array of asset ids). */
export const ASSET_DRAG_TYPE = 'application/x-simpleeffect-assets';

let dragged: readonly string[] = [];

/** Starts dragging `assetIds` (call from `dragstart`). */
export function startAssetDrag(dataTransfer: DataTransfer, assetIds: readonly string[]): void {
  dragged = assetIds;
  dataTransfer.setData(ASSET_DRAG_TYPE, JSON.stringify(assetIds));
  dataTransfer.effectAllowed = 'copy';
}

/** Whether a drag carries media bin assets. */
export function isAssetDrag(dataTransfer: DataTransfer | null): boolean {
  return dataTransfer?.types.includes(ASSET_DRAG_TYPE) ?? false;
}

/** Parses the asset ids of drag data; `[]` when it is not a list of strings. */
function parseIds(data: string): string[] {
  try {
    const parsed: unknown = JSON.parse(data);
    return Array.isArray(parsed) ? parsed.filter((id): id is string => typeof id === 'string') : [];
  } catch {
    return [];
  }
}

/**
 * The dragged asset ids: from the drag data when it is readable (on drop), otherwise the ids kept
 * since `dragstart` (during `dragover`).
 */
export function draggedAssetIds(dataTransfer: DataTransfer | null): readonly string[] {
  const data = dataTransfer?.getData(ASSET_DRAG_TYPE) ?? '';
  return data === '' ? dragged : parseIds(data);
}

/** The drag ended (call from `dragend` and after a drop). */
export function endAssetDrag(): void {
  dragged = [];
}
