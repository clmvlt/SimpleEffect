import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { Dock, DockHandle, DockPanel } from './dock';
import {
  dockLayoutKey,
  loadDockLayout,
  saveDockLayout,
  type DockLayoutStorage,
} from './dock-layout-storage';

/** In-memory storage standing in for the app's localStorage adapter. */
function createMemoryStorage(): DockLayoutStorage & { readonly entries: Map<string, string> } {
  const entries = new Map<string, string>();
  return {
    entries,
    getItem: (key) => entries.get(key) ?? null,
    setItem: (key, value) => {
      entries.set(key, value);
    },
  };
}

function TwoPanels({ storage }: { storage?: DockLayoutStorage }) {
  return (
    <Dock layoutId="test.layout" storage={storage}>
      <DockPanel id="left" defaultSize="30%">
        Gauche
      </DockPanel>
      <DockHandle label="Redimensionner" />
      <DockPanel id="right">Droite</DockPanel>
    </Dock>
  );
}

/**
 * Size of the first panel as a percentage, read from the separator that controls it. (Resizing by
 * pointer or keyboard needs element rectangles, which jsdom lacks: it is checked in the app.)
 */
function separatorValue(): number {
  return Number(
    screen.getByRole('separator', { name: 'Redimensionner' }).getAttribute('aria-valuenow'),
  );
}

describe('Dock', () => {
  it('renders its panels around a named, keyboard-focusable handle', async () => {
    const user = userEvent.setup();
    render(<TwoPanels />);
    expect(screen.getByText('Gauche')).toBeInTheDocument();
    expect(screen.getByText('Droite')).toBeInTheDocument();
    await user.tab();
    expect(screen.getByRole('separator', { name: 'Redimensionner' })).toHaveFocus();
  });

  it('starts from the layout saved under its layout id', () => {
    const storage = createMemoryStorage();
    saveDockLayout(storage, 'test.layout', { left: 42, right: 58 });
    render(<TwoPanels storage={storage} />);
    expect(separatorValue()).toBeCloseTo(42);
  });

  it('ignores layouts saved under another layout id', () => {
    const storage = createMemoryStorage();
    saveDockLayout(storage, 'other.layout', { left: 42, right: 58 });
    render(<TwoPanels storage={storage} />);
    expect(separatorValue()).not.toBeCloseTo(42);
  });
});

describe('dock layout storage', () => {
  it('round-trips a layout under a versioned key', () => {
    const storage = createMemoryStorage();
    saveDockLayout(storage, 'editor.main', { media: 20, center: 80 });
    expect([...storage.entries.keys()]).toEqual([dockLayoutKey('editor.main')]);
    expect(loadDockLayout(storage, 'editor.main')).toEqual({ media: 20, center: 80 });
  });

  it('ignores missing, corrupt and malformed entries', () => {
    const storage = createMemoryStorage();
    expect(loadDockLayout(storage, 'absent')).toBeUndefined();
    storage.setItem(dockLayoutKey('corrupt'), '{not json');
    expect(loadDockLayout(storage, 'corrupt')).toBeUndefined();
    storage.setItem(dockLayoutKey('malformed'), JSON.stringify({ media: 'wide' }));
    expect(loadDockLayout(storage, 'malformed')).toBeUndefined();
    storage.setItem(dockLayoutKey('array'), JSON.stringify([20, 80]));
    expect(loadDockLayout(storage, 'array')).toBeUndefined();
  });
});
