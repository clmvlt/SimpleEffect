import { formatShortcutKeys, type ShortcutKey, type ShortcutKeyLabels } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { shortcutDisplayKeys } from './shortcut';
import type { ShortcutActionId } from './shortcut-actions';
import { useShortcutRegistry } from './shortcut-context';

/** Localised names of spelled-out keys (non-Apple keyboards) and of Home/End. */
export function useShortcutKeyLabels(): ShortcutKeyLabels & { Home: string; End: string } {
  const { t } = useTranslation();
  return {
    Ctrl: t('shortcuts.keys.ctrl'),
    Alt: t('shortcuts.keys.alt'),
    Shift: t('shortcuts.keys.shift'),
    Enter: t('shortcuts.keys.enter'),
    Backspace: t('shortcuts.keys.backspace'),
    Delete: t('shortcuts.keys.delete'),
    Escape: t('shortcuts.keys.escape'),
    Space: t('shortcuts.keys.space'),
    Tab: t('shortcuts.keys.tab'),
    Home: t('shortcuts.keys.home'),
    End: t('shortcuts.keys.end'),
  };
}

/**
 * Platform-neutral keys of every combination of `action`, Home/End already localised (for
 * `KeyboardShortcut keys=…`).
 */
export function useShortcutDisplayKeys(action: ShortcutActionId): ShortcutKey[][] {
  const registry = useShortcutRegistry();
  const labels = useShortcutKeyLabels();
  const localised = (key: ShortcutKey): ShortcutKey => {
    if (key === 'Home') return labels.Home;
    if (key === 'End') return labels.End;
    return key;
  };
  return registry
    .shortcutsOf(action)
    .map((shortcut) => shortcutDisplayKeys(shortcut).map(localised));
}

/**
 * The key caps of the first combination of `action`, written the way this OS writes them, for
 * `Tooltip shortcut=…`.
 */
export function useShortcutHint(action: ShortcutActionId): string[] | undefined {
  const registry = useShortcutRegistry();
  const labels = useShortcutKeyLabels();
  const [first] = useShortcutDisplayKeys(action);
  return first === undefined ? undefined : formatShortcutKeys(first, registry.platform, labels);
}
