import type { ComponentProps } from 'react';

import { detectKeyboardPlatform, type KeyboardPlatform } from '../../lib/platform';
import { Kbd } from '../../primitives/kbd';
import { formatShortcutKeys, type ShortcutKey, type ShortcutKeyLabels } from './shortcut-format';

/** Props of {@link KeyboardShortcut}. */
export interface KeyboardShortcutProps extends Omit<ComponentProps<'kbd'>, 'children'> {
  /** Platform-neutral keys: `['Mod', 'Shift', 'Z']` (`Mod` = ⌘ on Apple systems, Ctrl elsewhere). */
  keys: readonly ShortcutKey[];
  /** Forces a platform (stories, tests); detected from the browser by default. */
  platform?: KeyboardPlatform | undefined;
  /** Localised names of spelled-out keys on non-Apple systems (`{ Shift: 'Maj' }`). */
  keyLabels?: ShortcutKeyLabels | undefined;
}

/**
 * A keyboard shortcut written the way the user's OS writes it: ⌘ ⌥ ⇧ glyphs on macOS, `Ctrl`,
 * `Alt`, `Shift` elsewhere. Declare shortcuts once with neutral names and show them in menus,
 * tooltips and help screens.
 */
export function KeyboardShortcut({
  keys,
  platform = detectKeyboardPlatform(),
  keyLabels,
  ...props
}: KeyboardShortcutProps) {
  return <Kbd keys={formatShortcutKeys(keys, platform, keyLabels)} {...props} />;
}
