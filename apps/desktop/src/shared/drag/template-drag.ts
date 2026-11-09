/**
 * Dragging a motion template from the template browser to the timeline (prompt 25). The
 * template travels in the drag data under {@link TEMPLATE_DRAG_TYPE}; browsers hide drag data
 * until the drop, so it is also kept here while the drag lasts, for the timeline's ghost during
 * `dragover`.
 */

import type { Flicks } from '@se/protocol';

/** Drag data type of templates (a JSON {@link DraggedTemplate}). */
export const TEMPLATE_DRAG_TYPE = 'application/x-simpleeffect-template';

/** The template being dragged: a project template or a library entry. */
export interface DraggedTemplate {
  readonly source: 'project' | 'library';
  /** The project template id, or the library entry id. */
  readonly id: string;
  /** Its name (the ghost's label). */
  readonly name: string;
  /** Length of the instance a drop makes. */
  readonly duration: Flicks;
}

let dragged: DraggedTemplate | null = null;

/** Starts dragging `template` (call from `dragstart`). */
export function startTemplateDrag(dataTransfer: DataTransfer, template: DraggedTemplate): void {
  dragged = template;
  dataTransfer.setData(TEMPLATE_DRAG_TYPE, JSON.stringify(template));
  dataTransfer.effectAllowed = 'copy';
}

/** Whether a drag carries a template. */
export function isTemplateDrag(dataTransfer: DataTransfer | null): boolean {
  return dataTransfer?.types.includes(TEMPLATE_DRAG_TYPE) ?? false;
}

/** Reads drag data as a template, `null` when it is not one. */
function parseTemplate(data: string): DraggedTemplate | null {
  try {
    const parsed: unknown = JSON.parse(data);
    if (typeof parsed !== 'object' || parsed === null) return null;
    const { source, id, name, duration } = parsed as Record<string, unknown>;
    if (
      (source === 'project' || source === 'library') &&
      typeof id === 'string' &&
      typeof name === 'string' &&
      typeof duration === 'number' &&
      duration > 0
    ) {
      return { source, id, name, duration: duration as Flicks };
    }
    return null;
  } catch {
    return null;
  }
}

/**
 * The dragged template: from the drag data when it is readable (on drop), otherwise the one kept
 * since `dragstart` (during `dragover`).
 */
export function draggedTemplate(dataTransfer: DataTransfer | null): DraggedTemplate | null {
  const data = dataTransfer?.getData(TEMPLATE_DRAG_TYPE) ?? '';
  return data === '' ? dragged : parseTemplate(data);
}

/** The drag ended (call from `dragend` and after a drop). */
export function endTemplateDrag(): void {
  dragged = null;
}
