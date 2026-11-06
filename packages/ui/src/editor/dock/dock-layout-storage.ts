/**
 * Where a {@link Dock} saves its panel sizes: any object with the `getItem`/`setItem` subset of the
 * Web Storage API (the app passes a namespaced `localStorage` wrapper; tests pass a memory map).
 */
export type DockLayoutStorage = Pick<Storage, 'getItem' | 'setItem'>;

/** Panel sizes of one dock: panel id → percentage of the dock (0–100). */
export type DockLayout = Readonly<Record<string, number>>;

/** Storage key of a layout. Versioned so a future format change can ignore old entries. */
export function dockLayoutKey(layoutId: string): string {
  return `dock-layout.v1.${layoutId}`;
}

function isDockLayout(value: unknown): value is DockLayout {
  return (
    typeof value === 'object' &&
    value !== null &&
    !Array.isArray(value) &&
    Object.values(value).every((size) => typeof size === 'number' && Number.isFinite(size))
  );
}

/**
 * The saved layout of `layoutId`, or `undefined` when there is none or it is unreadable (a corrupt
 * entry must never break the editor: the dock then starts from its default sizes).
 */
export function loadDockLayout(
  storage: DockLayoutStorage,
  layoutId: string,
): DockLayout | undefined {
  const saved = storage.getItem(dockLayoutKey(layoutId));
  if (saved === null) {
    return undefined;
  }
  try {
    const parsed: unknown = JSON.parse(saved);
    return isDockLayout(parsed) ? parsed : undefined;
  } catch {
    return undefined;
  }
}

/** Saves the layout of `layoutId`. */
export function saveDockLayout(
  storage: DockLayoutStorage,
  layoutId: string,
  layout: DockLayout,
): void {
  storage.setItem(dockLayoutKey(layoutId), JSON.stringify(layout));
}
