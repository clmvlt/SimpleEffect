import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Tabs, TabsContent, TabsList, TabsTrigger } from './tabs';

function renderTabs(onValueChange = vi.fn()) {
  render(
    <Tabs defaultValue="video" onValueChange={onValueChange}>
      <TabsList variant="underline" size="xs" aria-label="Propriétés">
        <TabsTrigger value="video">Vidéo</TabsTrigger>
        <TabsTrigger value="audio">Audio</TabsTrigger>
        <TabsTrigger value="effects" disabled>
          Effets
        </TabsTrigger>
        <TabsTrigger value="captions">Sous-titres</TabsTrigger>
      </TabsList>
      <TabsContent value="video">Panneau vidéo</TabsContent>
      <TabsContent value="audio">Panneau audio</TabsContent>
      <TabsContent value="effects">Panneau effets</TabsContent>
      <TabsContent value="captions">Panneau sous-titres</TabsContent>
    </Tabs>,
  );
  return { onValueChange, tab: (name: string) => screen.getByRole('tab', { name }) };
}

describe('Tabs', () => {
  it('shows the panel of the selected tab only', () => {
    const { tab } = renderTabs();
    expect(tab('Vidéo')).toHaveAttribute('aria-selected', 'true');
    expect(tab('Audio')).toHaveAttribute('aria-selected', 'false');
    expect(screen.getByRole('tabpanel')).toHaveTextContent('Panneau vidéo');
  });

  it('activates a tab on click', async () => {
    const user = userEvent.setup();
    const { tab, onValueChange } = renderTabs();
    await user.click(tab('Audio'));
    expect(onValueChange).toHaveBeenCalledWith('audio');
    expect(tab('Audio')).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByRole('tabpanel')).toHaveTextContent('Panneau audio');
  });

  it('moves with the arrow keys, skipping disabled tabs and wrapping around', async () => {
    const user = userEvent.setup();
    const { tab } = renderTabs();
    await user.tab();
    expect(tab('Vidéo')).toHaveFocus();

    await user.keyboard('{ArrowRight}');
    expect(tab('Audio')).toHaveFocus();
    expect(tab('Audio')).toHaveAttribute('aria-selected', 'true');

    await user.keyboard('{ArrowRight}');
    expect(tab('Sous-titres')).toHaveFocus();

    await user.keyboard('{ArrowRight}');
    expect(tab('Vidéo')).toHaveFocus();

    await user.keyboard('{ArrowLeft}');
    expect(tab('Sous-titres')).toHaveFocus();
  });

  it('jumps to the first and last tabs with Home and End', async () => {
    const user = userEvent.setup();
    const { tab } = renderTabs();
    await user.tab();
    await user.keyboard('{End}');
    expect(tab('Sous-titres')).toHaveFocus();
    await user.keyboard('{Home}');
    expect(tab('Vidéo')).toHaveFocus();
  });

  it('moves focus from the tab list to the active panel with Tab', async () => {
    const user = userEvent.setup();
    renderTabs();
    await user.tab();
    await user.tab();
    expect(screen.getByRole('tabpanel')).toHaveFocus();
  });

  it('styles the triggers from the list variant and size', () => {
    const { tab } = renderTabs();
    expect(tab('Vidéo')).toHaveClass('border-b-2', 'h-control-xs', 'text-xs');
  });
});
