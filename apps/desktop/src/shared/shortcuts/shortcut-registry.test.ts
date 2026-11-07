/** @vitest-environment jsdom */
import { describe, expect, it, vi } from 'vitest';

import { matchesShortcut, parseShortcut } from './shortcut';
import { ShortcutRegistry } from './shortcut-registry';

function keyDown(init: KeyboardEventInit, target: EventTarget = document.body): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { ...init, cancelable: true, bubbles: true });
  Object.defineProperty(event, 'target', { value: target });
  return event;
}

describe('parseShortcut', () => {
  it('reads modifiers and the key', () => {
    expect(parseShortcut('Mod+Shift+Z')).toEqual({ key: 'z', mod: true, shift: true, alt: false });
  });

  it('refuses combinations without a key or with unknown modifiers', () => {
    expect(() => parseShortcut('mod+shift')).toThrow(/invalid shortcut/);
    expect(() => parseShortcut('hyper+z')).toThrow(/invalid shortcut/);
  });
});

describe('matchesShortcut', () => {
  const undo = parseShortcut('mod+z');

  it('uses ⌘ on Apple systems and Ctrl elsewhere', () => {
    expect(matchesShortcut(keyDown({ key: 'z', metaKey: true }), undo, 'apple')).toBe(true);
    expect(matchesShortcut(keyDown({ key: 'z', ctrlKey: true }), undo, 'apple')).toBe(false);
    expect(matchesShortcut(keyDown({ key: 'z', ctrlKey: true }), undo, 'other')).toBe(true);
  });

  it('requires the exact modifiers', () => {
    expect(
      matchesShortcut(keyDown({ key: 'Z', ctrlKey: true, shiftKey: true }), undo, 'other'),
    ).toBe(false);
    expect(
      matchesShortcut(
        keyDown({ key: 'Z', ctrlKey: true, shiftKey: true }),
        parseShortcut('mod+shift+z'),
        'other',
      ),
    ).toBe(true);
  });
});

describe('ShortcutRegistry', () => {
  it('runs the latest matching binding and prevents the default action', () => {
    const registry = new ShortcutRegistry('other');
    const first = vi.fn();
    const second = vi.fn();
    registry.register({ keys: 'mod+z', run: first });
    const unregister = registry.register({ keys: 'mod+z', run: second });
    const event = keyDown({ key: 'z', ctrlKey: true });
    expect(registry.handleKeyDown(event)).toBe(true);
    expect(event.defaultPrevented).toBe(true);
    expect(second).toHaveBeenCalledOnce();
    unregister();
    registry.handleKeyDown(keyDown({ key: 'z', ctrlKey: true }));
    expect(first).toHaveBeenCalledOnce();
  });

  it('leaves key presses in text fields alone', () => {
    const registry = new ShortcutRegistry('other');
    const run = vi.fn();
    registry.register({ keys: 'mod+z', run });
    const input = document.createElement('input');
    expect(registry.handleKeyDown(keyDown({ key: 'z', ctrlKey: true }, input))).toBe(false);
    expect(run).not.toHaveBeenCalled();
  });
});
