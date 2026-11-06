import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { detectKeyboardPlatform } from '../../lib/platform';
import { KeyboardShortcut } from './keyboard-shortcut';
import { formatShortcutKeys } from './shortcut-format';

describe('formatShortcutKeys', () => {
  it('uses glyphs on Apple systems', () => {
    expect(formatShortcutKeys(['Mod', 'Alt', 'Shift', 'Z'], 'apple')).toEqual(['⌘', '⌥', '⇧', 'Z']);
    expect(formatShortcutKeys(['Mod', 'Enter'], 'apple')).toEqual(['⌘', '↩']);
  });

  it('spells modifiers elsewhere, with localised names', () => {
    expect(formatShortcutKeys(['Mod', 'Shift', 'Z'], 'other')).toEqual(['Ctrl', 'Shift', 'Z']);
    expect(
      formatShortcutKeys(['Mod', 'Shift', 'Delete'], 'other', { Shift: 'Maj', Delete: 'Suppr' }),
    ).toEqual(['Ctrl', 'Maj', 'Suppr']);
  });

  it('draws arrows the same everywhere', () => {
    expect(formatShortcutKeys(['Shift', 'ArrowLeft'], 'apple')).toEqual(['⇧', '←']);
    expect(formatShortcutKeys(['Shift', 'ArrowLeft'], 'other')).toEqual(['Shift', '←']);
  });
});

describe('detectKeyboardPlatform', () => {
  it('recognises Apple platforms from navigator hints', () => {
    expect(detectKeyboardPlatform({ platform: 'MacIntel', userAgent: '' })).toBe('apple');
    expect(detectKeyboardPlatform({ platform: 'Win32', userAgent: '' })).toBe('other');
    expect(
      detectKeyboardPlatform({ platform: '', userAgent: 'Mozilla/5.0 (iPad; CPU OS 18_0)' }),
    ).toBe('apple');
  });
});

describe('KeyboardShortcut', () => {
  it('renders one key cap per key', () => {
    const { container } = render(<KeyboardShortcut keys={['Mod', 'K']} platform="other" />);
    const caps = container.querySelectorAll('kbd kbd');
    expect(Array.from(caps, (cap) => cap.textContent)).toEqual(['Ctrl', 'K']);
  });
});
