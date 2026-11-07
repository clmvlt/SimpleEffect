/**
 * @vitest-environment jsdom
 */
import type { SequenceEntity } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { SequenceTabs } from './SequenceTabs';

const format = {
  width: 1920,
  height: 1080,
  frameRate: { numerator: 25, denominator: 1 },
  sampleRate: 48_000,
};
const sequence = (id: string, name: string): SequenceEntity => ({
  id,
  name,
  format,
  trackIds: [],
  markerIds: [],
});

function renderTabs(sequences: SequenceEntity[]) {
  const handlers = {
    onSelect: vi.fn(),
    onCreate: vi.fn(),
    onAction: vi.fn(),
    onRename: vi.fn(),
  };
  render(
    <I18nextProvider i18n={createI18n()}>
      <TooltipProvider>
        <SequenceTabs sequences={sequences} activeId={sequences[0]?.id ?? ''} {...handlers} />
      </TooltipProvider>
    </I18nextProvider>,
  );
  return handlers;
}

describe('sequence tabs', () => {
  it('shows a lone sequence too, with its menu', async () => {
    const user = userEvent.setup();
    const handlers = renderTabs([sequence('a', 'Montage')]);
    expect(screen.getByRole('tab', { name: 'Montage' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Actions de la séquence' }));
    expect(screen.getByRole('menuitem', { name: 'Supprimer…' })).toHaveAttribute(
      'aria-disabled',
      'true',
    );
    await user.click(screen.getByRole('menuitem', { name: 'Créer une version…' }));
    expect(handlers.onAction).toHaveBeenCalledWith('version', 'a');
  });

  it('opens the actions of any tab from its context menu', async () => {
    const user = userEvent.setup();
    const handlers = renderTabs([sequence('a', 'Montage'), sequence('b', 'Short')]);
    fireEvent.contextMenu(screen.getByRole('tab', { name: 'Short' }));
    await user.click(await screen.findByRole('menuitem', { name: 'Format de la séquence…' }));
    expect(handlers.onAction).toHaveBeenCalledWith('format', 'b');
    fireEvent.contextMenu(screen.getByRole('tab', { name: 'Short' }));
    await user.click(await screen.findByRole('menuitem', { name: 'Supprimer…' }));
    expect(handlers.onAction).toHaveBeenCalledWith('remove', 'b');
  });

  it('renames a tab in place on double click', async () => {
    const user = userEvent.setup();
    const handlers = renderTabs([sequence('a', 'Montage'), sequence('b', 'Short')]);
    await user.dblClick(screen.getByRole('tab', { name: 'Short' }));
    const field = screen.getByRole('textbox', { name: 'Nom de la séquence' });
    await user.clear(field);
    await user.type(field, 'Short 9:16{Enter}');
    expect(handlers.onRename).toHaveBeenCalledWith('b', 'Short 9:16');
    await user.dblClick(screen.getByRole('tab', { name: 'Montage' }));
    await user.keyboard('{Escape}');
    expect(handlers.onRename).toHaveBeenCalledTimes(1);
  });
});
