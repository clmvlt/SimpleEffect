import type { PointerEvent as ReactPointerEvent } from 'react';

/** One pointer movement during a drag, relative to the previous movement and to the start. */
export interface PointerDragMove {
  /** Horizontal movement since the previous move event, in CSS pixels. */
  readonly deltaX: number;
  /** Vertical movement since the previous move event, in CSS pixels. */
  readonly deltaY: number;
  /** Pointer position in the viewport (for widgets that map positions, not deltas). */
  readonly clientX: number;
  readonly clientY: number;
  /** Modifiers held during this movement (they may change mid-drag). */
  readonly shiftKey: boolean;
  readonly altKey: boolean;
}

/** Callbacks of {@link usePointerDrag}. */
export interface PointerDragOptions {
  /**
   * Called on the primary-button press, before any movement. Return `false` to ignore this
   * press (e.g. the widget is disabled or the press landed on a child control).
   */
  onStart?: (event: ReactPointerEvent) => boolean | undefined;
  /** Called for every movement once the pointer has travelled `threshold` pixels. */
  onMove: (move: PointerDragMove) => void;
  /** Called once when the pointer is released or the drag is cancelled. */
  onEnd?: (result: { readonly moved: boolean }) => void;
  /** Distance in pixels before a press becomes a drag, so clicks do not jitter values (default 2). */
  threshold?: number;
}

/** Event handlers to spread on the element that starts the drag. */
export interface PointerDragHandlers {
  onPointerDown: (event: ReactPointerEvent) => void;
}

/**
 * Pointer dragging with pointer capture, shared by scrubbable fields and curve handles. Movement is
 * reported as per-event deltas so modifier keys (fast, fine) can change during one gesture, and
 * `onEnd` fires exactly once per press so callers can commit a gesture as one undo step.
 */
export function usePointerDrag({
  onStart,
  onMove,
  onEnd,
  threshold = 2,
}: PointerDragOptions): PointerDragHandlers {
  const onPointerDown = (event: ReactPointerEvent) => {
    if (event.button !== 0 || onStart?.(event) === false) {
      return;
    }
    const target = event.currentTarget;
    const pointerId = event.pointerId;
    const startX = event.clientX;
    const startY = event.clientY;
    let lastX = startX;
    let lastY = startY;
    let moved = false;

    target.setPointerCapture(pointerId);

    const handleMove = (moveEvent: Event) => {
      if (!(moveEvent instanceof PointerEvent) || moveEvent.pointerId !== pointerId) {
        return;
      }
      if (
        !moved &&
        Math.hypot(moveEvent.clientX - startX, moveEvent.clientY - startY) < threshold
      ) {
        return;
      }
      moved = true;
      onMove({
        deltaX: moveEvent.clientX - lastX,
        deltaY: moveEvent.clientY - lastY,
        clientX: moveEvent.clientX,
        clientY: moveEvent.clientY,
        shiftKey: moveEvent.shiftKey,
        altKey: moveEvent.altKey,
      });
      lastX = moveEvent.clientX;
      lastY = moveEvent.clientY;
    };

    const handleEnd = (endEvent: Event) => {
      if (endEvent instanceof PointerEvent && endEvent.pointerId !== pointerId) {
        return;
      }
      target.removeEventListener('pointermove', handleMove);
      target.removeEventListener('pointerup', handleEnd);
      target.removeEventListener('pointercancel', handleEnd);
      if (target.hasPointerCapture(pointerId)) {
        target.releasePointerCapture(pointerId);
      }
      onEnd?.({ moved });
    };

    target.addEventListener('pointermove', handleMove);
    target.addEventListener('pointerup', handleEnd);
    target.addEventListener('pointercancel', handleEnd);
  };

  return { onPointerDown };
}
