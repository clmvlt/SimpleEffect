import type { Flicks, TrackId } from '@se/protocol';
import { useRef, type DragEvent } from 'react';

import {
  draggedTemplate,
  endTemplateDrag,
  isTemplateDrag,
  type DraggedTemplate,
} from '../../../shared/drag';
import { snapToFrame } from '../../../shared/time';
import { rowAt } from '../services/track-layout';
import { xToTime } from '../services/viewport';
import { clearGesturePreview, setGesturePreview } from '../store/gesture-store';
import { selectItems } from '../store/selection-store';
import { snapEdges, type GestureContext } from './gestures/gesture-context';

/** Where a template drop lands. */
interface TemplateDropPlan {
  readonly template: DraggedTemplate;
  readonly start: Flicks;
  /** The video track pointed at; `null` lets the core choose (a free track, or a new one). */
  readonly trackId: TrackId | null;
}

/**
 * Dropping a motion template of the template browser on the lanes (prompt 25): a ghost of the
 * instance follows the pointer (frame aligned, snapping its edges) on the video track under it,
 * and the drop places the instance there — on another free track when that one is busy, through
 * the core's usual choice — then selects it. One command: `addGraphicItem` for a project
 * template, `insertLibraryTemplate` for a library entry (copied into the project).
 */
export function useTemplateDrop(context: () => GestureContext) {
  const plan = useRef<TemplateDropPlan | null>(null);

  const planDrop = (event: DragEvent<HTMLElement>): TemplateDropPlan | null => {
    const template = draggedTemplate(event.dataTransfer);
    if (template === null) return null;
    const current = context();
    const { model, layout, viewport } = current;
    const bounds = event.currentTarget.getBoundingClientRect();
    const [x, y] = [event.clientX - bounds.left, event.clientY - bounds.top];
    const pointed = snapToFrame(xToTime(x, viewport), model.sequence.format.frameRate, 'nearest');
    const end = (pointed + template.duration) as Flicks;
    const snap = snapEdges(current, [pointed, end], []);
    const start = Math.max(0, pointed + (snap?.delta ?? 0)) as Flicks;
    const row = rowAt(layout, y);
    const trackId = row?.track.kind === 'video' ? row.track.id : null;
    setGesturePreview({
      placements: new Map(),
      ghosts:
        trackId === null
          ? []
          : [
              {
                key: 'template-drop',
                row: { kind: 'track', trackId },
                start,
                duration: template.duration,
                source: { kind: 'media', label: template.name, tone: 'graphic' },
              },
            ],
      snapTime: snap?.point ?? null,
    });
    return { template, start, trackId };
  };

  const onDragOver = (event: DragEvent<HTMLDivElement>) => {
    if (!isTemplateDrag(event.dataTransfer)) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = 'copy';
    plan.current = planDrop(event);
    if (plan.current === null) clearGesturePreview();
  };

  const onDragLeave = (event: DragEvent<HTMLDivElement>) => {
    const next = event.relatedTarget;
    if (next instanceof Node && event.currentTarget.contains(next)) return;
    plan.current = null;
    clearGesturePreview();
  };

  const onDrop = async (event: DragEvent<HTMLDivElement>) => {
    if (!isTemplateDrag(event.dataTransfer)) return;
    event.preventDefault();
    const drop = planDrop(event) ?? plan.current;
    plan.current = null;
    endTemplateDrag();
    if (drop === null) {
      clearGesturePreview();
      return;
    }
    const { model, commands } = context();
    const itemId = await commands.insertTemplate(
      model.sequence.id,
      drop.template,
      drop.start,
      drop.trackId,
    );
    if (itemId !== null) selectItems([itemId]);
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
