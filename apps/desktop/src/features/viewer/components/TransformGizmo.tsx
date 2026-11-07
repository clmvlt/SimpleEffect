import type { ItemId, SequenceEntity, Vec2 } from '@se/protocol';
import { useState, useSyncExternalStore } from 'react';
import { useTranslation } from 'react-i18next';

import { usePlaybackRate } from '../../../shared/playback';
import { startReframing } from '../../../shared/reframe';
import { useSelectedItemIds } from '../../timeline';
import { useGizmoPointer } from '../hooks/use-gizmo-pointer';
import {
  HANDLE_POSITIONS,
  cornersOf,
  pivotOf,
  toFrame,
  type PlacedLayer,
  type ScaleHandle,
} from '../services/gizmo-geometry';
import type { ViewerEngineController } from '../services/viewer-engine-controller';
import { TextEditOverlay } from './TextEditOverlay';

/** Props of {@link TransformGizmo}. */
export interface TransformGizmoProps {
  readonly controller: ViewerEngineController;
  readonly sequence: SequenceEntity;
  /** CSS pixels per frame pixel. */
  readonly displayScale: number;
}

/** Distance of the rotation handle above the box, in CSS pixels. */
const ROTATE_OFFSET = 24;
const HANDLE = 8;
const CORNER_CURSOR = 'nwse-resize';
const SIDE_CURSOR = 'ew-resize';

const points = (list: readonly Vec2[], scale: number) =>
  list.map((point) => `${String(point.x * scale)},${String(point.y * scale)}`).join(' ');

/**
 * Transform handles over the picture: the outline of each selected layer and, for one, scale
 * handles on its corners and sides (Shift: proportional), a rotation handle above it (Shift: 15°)
 * and its anchor (drag to re-anchor without moving it). Clicking selects layers; dragging moves
 * them with snapping to the frame's centre and edges; double-clicking a text edits it in place,
 * double-clicking a clip that fills the frame starts reframing it.
 * Hidden while playing.
 */
export function TransformGizmo({ controller, sequence, displayScale }: TransformGizmoProps) {
  const { t } = useTranslation();
  const layout = useSyncExternalStore(controller.subscribeLayout, controller.getLayout);
  const selected = useSelectedItemIds();
  const playing = usePlaybackRate() !== 0;
  const [editing, setEditing] = useState<ItemId | null>(null);
  const layers: readonly PlacedLayer[] = layout?.layers ?? [];
  const pointer = useGizmoPointer({
    sequence,
    layers,
    selected,
    displayScale,
    disabled: playing || editing !== null,
  });
  const frame = sequence.format;
  const shown = playing ? [] : layers.filter((layer) => selected.includes(layer.itemId));
  const [single] = shown.length === 1 ? shown : [];
  const editedLayer = layers.find((layer) => layer.itemId === editing);
  return (
    <div
      className="absolute inset-0"
      data-viewer-gizmo=""
      onPointerDown={pointer.onPointerDown}
      onPointerMove={pointer.onPointerMove}
      onPointerUp={pointer.onPointerUp}
      onDoubleClick={(event) => {
        const text = pointer.textAt(event);
        if (text !== null) setEditing(text);
        const filling = pointer.fillingClipAt(event);
        if (filling !== null) startReframing(filling);
      }}
    >
      <svg aria-hidden className="pointer-events-none absolute inset-0 size-full overflow-visible">
        {pointer.guides.x.map((x) => (
          <line
            key={`x${String(x)}`}
            x1={x * displayScale}
            x2={x * displayScale}
            y1={0}
            y2="100%"
            className="stroke-accent"
            strokeDasharray="4 3"
          />
        ))}
        {pointer.guides.y.map((y) => (
          <line
            key={`y${String(y)}`}
            y1={y * displayScale}
            y2={y * displayScale}
            x1={0}
            x2="100%"
            className="stroke-accent"
            strokeDasharray="4 3"
          />
        ))}
        {shown.map((layer) => (
          <polygon
            key={layer.itemId}
            points={points(cornersOf(layer, frame), displayScale)}
            className="fill-none stroke-accent"
            strokeWidth={1.5}
          />
        ))}
      </svg>
      {single !== undefined && editing === null && (
        <Handles
          layer={single}
          frame={frame}
          scale={displayScale}
          label={t('editor.viewer.gizmo.handle')}
        />
      )}
      {editedLayer !== undefined && (
        <TextEditOverlay
          layer={editedLayer}
          frame={frame}
          displayScale={displayScale}
          onClose={() => {
            setEditing(null);
          }}
        />
      )}
    </div>
  );
}

/** The scale, rotation and anchor handles of one layer (`data-gizmo-handle`). */
function Handles({
  layer,
  frame,
  scale,
  label,
}: {
  readonly layer: PlacedLayer;
  readonly frame: { readonly width: number; readonly height: number };
  readonly scale: number;
  readonly label: string;
}) {
  const { box, transform } = layer;
  const at = (local: Vec2) => toFrame(transform, box, frame, local);
  const lift = ROTATE_OFFSET / (scale * Math.max(1e-6, Math.abs(transform.scale.y)));
  const rotate = at({ x: box.width / 2, y: -lift * Math.sign(transform.scale.y || 1) });
  const pivot = pivotOf(transform, frame);
  const square = (point: Vec2, handle: string, cursor: string) => (
    <div
      key={handle}
      data-gizmo-handle={handle}
      aria-label={label}
      className="absolute rounded-xs border border-accent bg-panel"
      style={{
        left: point.x * scale - HANDLE / 2,
        top: point.y * scale - HANDLE / 2,
        width: HANDLE,
        height: HANDLE,
        cursor,
      }}
    />
  );
  return (
    <>
      {(Object.keys(HANDLE_POSITIONS) as ScaleHandle[]).map((handle) => {
        const position = HANDLE_POSITIONS[handle];
        const cursor = handle.length === 2 ? CORNER_CURSOR : SIDE_CURSOR;
        return square(
          at({ x: position.x * box.width, y: position.y * box.height }),
          `scale-${handle}`,
          cursor,
        );
      })}
      <div
        data-gizmo-handle="rotate"
        aria-label={label}
        className="absolute size-3 rounded-full border border-accent bg-panel"
        style={{ left: rotate.x * scale - 6, top: rotate.y * scale - 6, cursor: 'grab' }}
      />
      <div
        data-gizmo-handle="anchor"
        aria-label={label}
        className="absolute size-3.5 rounded-full border-2 border-accent"
        style={{ left: pivot.x * scale - 7, top: pivot.y * scale - 7, cursor: 'move' }}
      />
    </>
  );
}
