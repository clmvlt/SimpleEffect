import type { KeyboardPlatform } from '@se/ui';

import { matchesShortcut, parseShortcut, type Shortcut } from './shortcut';

/** A shortcut and what it does. */
export interface ShortcutBinding {
  /** The combination, e.g. `mod+z` (see `parseShortcut`). */
  readonly keys: string;
  /** Runs when the combination is pressed outside text fields. */
  readonly run: () => void;
}

interface RegisteredBinding {
  readonly shortcut: Shortcut;
  readonly run: () => void;
}

/** Whether keys typed in `target` belong to it (text fields keep their own undo). */
function isEditable(target: EventTarget | null): boolean {
  if (typeof HTMLElement === 'undefined' || !(target instanceof HTMLElement)) {
    return false;
  }
  return (
    target.isContentEditable ||
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement
  );
}

/**
 * The app's global keyboard shortcuts. Minimal for now (prompt 07: undo/redo); prompt 10 adds
 * scopes (timeline, viewer) and user-visible listings.
 */
export class ShortcutRegistry {
  readonly #platform: KeyboardPlatform;
  readonly #bindings: RegisteredBinding[] = [];

  constructor(platform: KeyboardPlatform) {
    this.#platform = platform;
  }

  /** Adds a binding; the latest one wins when two share a combination. Returns its removal. */
  register(binding: ShortcutBinding): () => void {
    const registered = { shortcut: parseShortcut(binding.keys), run: binding.run };
    this.#bindings.push(registered);
    return () => {
      const index = this.#bindings.indexOf(registered);
      if (index !== -1) this.#bindings.splice(index, 1);
    };
  }

  /** Runs the binding matching `event`, if any; returns whether one ran. */
  handleKeyDown(event: KeyboardEvent): boolean {
    if (event.defaultPrevented || isEditable(event.target)) {
      return false;
    }
    const binding = this.#bindings.findLast(({ shortcut }) =>
      matchesShortcut(event, shortcut, this.#platform),
    );
    if (binding === undefined) {
      return false;
    }
    event.preventDefault();
    binding.run();
    return true;
  }
}
