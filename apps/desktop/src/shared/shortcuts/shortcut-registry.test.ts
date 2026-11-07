/** @vitest-environment jsdom */
import { describe, expect, it, vi } from 'vitest';

import { matchesShortcut, parseShortcut, shortcutDisplayKeys } from './shortcut';
import { SHORTCUT_ACTIONS } from './shortcut-actions';
import { findShortcutConflicts } from './shortcut-conflicts';
import { ShortcutRegistry } from './shortcut-registry';

function keyDown(init: KeyboardEventInit, target: EventTarget = document.body): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { ...init, cancelable: true, bubbles: true });
  Object.defineProperty(event, 'target', { value: target });
  return event;
}

describe('parseShortcut', () => {
  it('reads modifiers, the key and aliases', () => {
    expect(parseShortcut('Mod+Shift+Z')).toEqual({ key: 'z', mod: true, shift: true, alt: false });
    expect(parseShortcut('plus').key).toBe('+');
    expect(parseShortcut('space').key).toBe(' ');
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

  it('requires the exact modifiers, except Shift on symbols', () => {
    const shifted = keyDown({ key: 'Z', ctrlKey: true, shiftKey: true });
    expect(matchesShortcut(shifted, undo, 'other')).toBe(false);
    expect(matchesShortcut(shifted, parseShortcut('mod+shift+z'), 'other')).toBe(true);
    const question = keyDown({ key: '?', shiftKey: true });
    expect(matchesShortcut(question, parseShortcut('?'), 'other')).toBe(true);
  });

  it('reads letters from the physical key with ⌥ on Apple systems', () => {
    const optionX = keyDown({ key: '≈', code: 'KeyX', altKey: true });
    expect(matchesShortcut(optionX, parseShortcut('alt+x'), 'apple')).toBe(true);
    expect(matchesShortcut(optionX, parseShortcut('alt+x'), 'other')).toBe(false);
  });
});

describe('shortcutDisplayKeys', () => {
  it('lists modifiers first, then the key cap', () => {
    expect(shortcutDisplayKeys(parseShortcut('mod+shift+z'))).toEqual(['Mod', 'Shift', 'Z']);
    expect(shortcutDisplayKeys(parseShortcut('shift+arrowleft'))).toEqual(['Shift', 'ArrowLeft']);
    expect(shortcutDisplayKeys(parseShortcut('home'))).toEqual(['Home']);
  });
});

describe('findShortcutConflicts', () => {
  it('finds none in the default table, on any platform', () => {
    expect(findShortcutConflicts(SHORTCUT_ACTIONS, 'other')).toEqual([]);
    expect(findShortcutConflicts(SHORTCUT_ACTIONS, 'apple')).toEqual([]);
  });

  it('reports actions sharing keys in overlapping contexts only', () => {
    const conflicts = findShortcutConflicts(
      {
        save: { contexts: ['global'], keys: ['mod+s'] },
        split: { contexts: ['timeline'], keys: ['Mod+S'] },
        play: { contexts: ['viewer'], keys: ['k'] },
        cut: { contexts: ['timeline'], keys: ['k'] },
      },
      'other',
    );
    expect(conflicts).toEqual([{ keys: 'mod+s', actions: ['save', 'split'] }]);
  });
});

describe('ShortcutRegistry', () => {
  it('runs the latest handler of the matching action and prevents the default action', () => {
    const registry = new ShortcutRegistry('other');
    const first = vi.fn();
    const second = vi.fn();
    registry.bind('undo', first);
    const unbind = registry.bind('undo', second);
    const event = keyDown({ key: 'z', ctrlKey: true });
    expect(registry.handleKeyDown(event)).toBe(true);
    expect(event.defaultPrevented).toBe(true);
    expect(second).toHaveBeenCalledOnce();
    unbind();
    registry.handleKeyDown(keyDown({ key: 'z', ctrlKey: true }));
    expect(first).toHaveBeenCalledOnce();
  });

  it('adapts default keys to the platform', () => {
    const redo = vi.fn();
    const windows = new ShortcutRegistry('other');
    windows.bind('redo', redo);
    windows.handleKeyDown(keyDown({ key: 'y', ctrlKey: true }));
    expect(redo).toHaveBeenCalledOnce();
    const mac = new ShortcutRegistry('apple');
    mac.bind('redo', redo);
    expect(mac.handleKeyDown(keyDown({ key: 'y', metaKey: true }))).toBe(false);
  });

  it('runs context actions only while their region is active', () => {
    const registry = new ShortcutRegistry('other');
    const markIn = vi.fn();
    registry.bind('markIn', markIn);
    expect(registry.handleKeyDown(keyDown({ key: 'i' }))).toBe(false);
    const listener = vi.fn();
    registry.subscribe(listener);
    registry.setActiveContext('timeline');
    expect(listener).toHaveBeenCalledOnce();
    expect(registry.handleKeyDown(keyDown({ key: 'i' }))).toBe(true);
    registry.setActiveContext('viewer');
    expect(registry.handleKeyDown(keyDown({ key: 'i' }))).toBe(true);
    expect(markIn).toHaveBeenCalledTimes(2);
  });

  it('ignores held keys unless the action repeats', () => {
    const registry = new ShortcutRegistry('other');
    registry.setActiveContext('timeline');
    const addMarker = vi.fn();
    const nextFrame = vi.fn();
    registry.bind('addMarker', addMarker);
    registry.bind('nextFrame', nextFrame);
    registry.handleKeyDown(keyDown({ key: 'm', repeat: true }));
    registry.handleKeyDown(keyDown({ key: 'ArrowRight', repeat: true }));
    expect(addMarker).not.toHaveBeenCalled();
    expect(nextFrame).toHaveBeenCalledOnce();
  });

  it('leaves key presses in text fields alone', () => {
    const registry = new ShortcutRegistry('other');
    const run = vi.fn();
    registry.bind('undo', run);
    const input = document.createElement('input');
    expect(registry.handleKeyDown(keyDown({ key: 'z', ctrlKey: true }, input))).toBe(false);
    expect(run).not.toHaveBeenCalled();
  });

  it('knows which keys are held, until released or the window blurs', () => {
    const registry = new ShortcutRegistry('other');
    registry.setActiveContext('viewer');
    const step = vi.fn();
    registry.bind('shuttleForward', () => {
      if (registry.isKeyHeld('k')) step();
    });
    registry.handleKeyDown(keyDown({ key: 'K' }));
    registry.handleKeyDown(keyDown({ key: 'l' }));
    expect(step).toHaveBeenCalledOnce();
    registry.handleKeyUp({ key: 'k' });
    expect(registry.isKeyHeld('k')).toBe(false);
    registry.handleKeyDown(keyDown({ key: 'k' }));
    registry.releaseKeys();
    expect(registry.isKeyHeld('k')).toBe(false);
  });
});
