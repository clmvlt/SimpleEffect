import type { KeyboardPlatform } from '@se/ui';

import { matchesShortcut, parseShortcut, type Shortcut } from './shortcut';
import {
  actionKeys,
  SHORTCUT_ACTIONS,
  type ShortcutActionId,
  type ShortcutActionTable,
  type ShortcutContext,
} from './shortcut-actions';

interface Handler {
  readonly action: ShortcutActionId;
  readonly run: () => void;
}

/** Whether keys typed in `target` belong to it (text fields keep their own undo and arrows). */
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
 * The app's keyboard shortcuts: named actions (`SHORTCUT_ACTIONS`) with default keys adapted to
 * the OS, handlers bound by components while mounted, and the active context (the region last
 * clicked or focused). A key press runs the latest handler of the action it matches, if that
 * action is global or belongs to the active context; never inside text fields.
 */
export class ShortcutRegistry {
  readonly platform: KeyboardPlatform;
  readonly actions: ShortcutActionTable;
  readonly #shortcuts: ReadonlyMap<ShortcutActionId, readonly Shortcut[]>;
  readonly #handlers: Handler[] = [];
  readonly #listeners = new Set<() => void>();
  readonly #heldKeys = new Set<string>();
  #activeContext: ShortcutContext | null = null;

  constructor(platform: KeyboardPlatform, actions: ShortcutActionTable = SHORTCUT_ACTIONS) {
    this.platform = platform;
    this.actions = actions;
    this.#shortcuts = new Map(
      (Object.keys(actions) as ShortcutActionId[]).map((id) => [
        id,
        actionKeys(actions[id], platform).map(parseShortcut),
      ]),
    );
  }

  /** The parsed combinations of `action` on this platform (first = the one to show). */
  shortcutsOf(action: ShortcutActionId): readonly Shortcut[] {
    return this.#shortcuts.get(action) ?? [];
  }

  /** Binds `run` to `action`; the latest binding of an action wins. Returns its removal. */
  bind(action: ShortcutActionId, run: () => void): () => void {
    const handler = { action, run };
    this.#handlers.push(handler);
    return () => {
      const index = this.#handlers.indexOf(handler);
      if (index !== -1) this.#handlers.splice(index, 1);
    };
  }

  /** The region whose shortcuts are live (`null`: only global ones). */
  get activeContext(): ShortcutContext | null {
    return this.#activeContext;
  }

  /** Makes `context` the live region and tells subscribers when it changes. */
  setActiveContext(context: ShortcutContext | null): void {
    if (context === this.#activeContext) return;
    this.#activeContext = context;
    for (const listener of this.#listeners) listener();
  }

  /** Calls `listener` when the active context changes; returns the unsubscription. */
  subscribe(listener: () => void): () => void {
    this.#listeners.add(listener);
    return () => {
      this.#listeners.delete(listener);
    };
  }

  /**
   * Whether `key` (as `KeyboardEvent.key`, case-insensitive) is held down now: actions combining
   * keys read it (K held while pressing J or L steps one frame).
   */
  isKeyHeld(key: string): boolean {
    return this.#heldKeys.has(key.toLowerCase());
  }

  /** A key was released. */
  handleKeyUp(event: Pick<KeyboardEvent, 'key'>): void {
    this.#heldKeys.delete(event.key.toLowerCase());
  }

  /** The window lost the focus: no key is held anymore (their release will not be seen). */
  releaseKeys(): void {
    this.#heldKeys.clear();
  }

  /** Runs the handler of the action matching `event`, if any; returns whether one ran. */
  handleKeyDown(event: KeyboardEvent): boolean {
    this.#heldKeys.add(event.key.toLowerCase());
    if (event.defaultPrevented || isEditable(event.target)) {
      return false;
    }
    const handler = this.#handlers.findLast(
      ({ action }) => this.#isLive(action) && this.#matches(action, event),
    );
    if (handler === undefined) {
      return false;
    }
    event.preventDefault();
    if (event.repeat && this.actions[handler.action].repeats !== true) {
      return false;
    }
    handler.run();
    return true;
  }

  #isLive(action: ShortcutActionId): boolean {
    const { contexts } = this.actions[action];
    return (
      contexts.includes('global') ||
      (this.#activeContext !== null && contexts.includes(this.#activeContext))
    );
  }

  #matches(action: ShortcutActionId, event: KeyboardEvent): boolean {
    return this.shortcutsOf(action).some((shortcut) =>
      matchesShortcut(event, shortcut, this.platform),
    );
  }
}
