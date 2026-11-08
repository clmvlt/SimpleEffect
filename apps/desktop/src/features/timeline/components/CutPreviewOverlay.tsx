import type { Flicks, SequenceId } from '@se/protocol';

import { spanBox } from '../services/item-geometry';
import type { Viewport } from '../services/viewport';
import { useCutPreview } from '../store/cut-preview-store';

/** Props of {@link CutPreviewOverlay}. */
export interface CutPreviewOverlayProps {
  readonly sequenceId: SequenceId;
  readonly viewport: Viewport;
}

/**
 * Cuts previewed by the silence removal panel (prompt 22): hatched danger bands over every lane,
 * with dashed edges, before anything changes. Clicks go through to the lanes.
 */
export function CutPreviewOverlay({ sequenceId, viewport }: CutPreviewOverlayProps) {
  const ranges = useCutPreview(sequenceId);
  return ranges.map((range) => {
    const box = spanBox(
      { start: range.start, end: (range.start + range.duration) as Flicks },
      viewport,
    );
    if (box.left + box.width < 0 || box.left > viewport.width) return null;
    return (
      <div
        key={range.start}
        aria-hidden
        data-cut-preview
        className="pointer-events-none absolute inset-y-0 z-10 border-x border-dashed border-danger bg-danger-muted/50"
        style={{ left: box.left, width: Math.max(box.width, 1) }}
      />
    );
  });
}
