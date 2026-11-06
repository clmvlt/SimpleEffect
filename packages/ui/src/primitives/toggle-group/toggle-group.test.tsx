import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ToggleGroup, ToggleGroupItem } from './toggle-group';

function renderToolGroup(onValueChange = vi.fn()) {
  render(
    <ToggleGroup
      type="single"
      defaultValue="select"
      aria-label="Outils"
      onValueChange={onValueChange}
    >
      <ToggleGroupItem value="select" aria-label="Sélection" />
      <ToggleGroupItem value="razor" aria-label="Lame" />
      <ToggleGroupItem value="slip" aria-label="Glissement" disabled />
      <ToggleGroupItem value="hand" aria-label="Main" />
    </ToggleGroup>,
  );
  return { onValueChange, tool: (name: string) => screen.getByRole('radio', { name }) };
}

describe('ToggleGroup (single)', () => {
  it('exposes its items as radio buttons inside a labelled radio group', () => {
    const { tool } = renderToolGroup();
    expect(screen.getByRole('radiogroup', { name: 'Outils' })).toBeInTheDocument();
    expect(tool('Sélection')).toHaveAttribute('aria-checked', 'true');
    expect(tool('Lame')).toHaveAttribute('aria-checked', 'false');
  });

  it('selects one item at a time', async () => {
    const user = userEvent.setup();
    const { tool, onValueChange } = renderToolGroup();
    await user.click(tool('Lame'));
    expect(onValueChange).toHaveBeenLastCalledWith('razor');
    expect(tool('Lame')).toHaveAttribute('aria-checked', 'true');
    expect(tool('Sélection')).toHaveAttribute('aria-checked', 'false');
  });

  it('moves focus with the arrow keys, skipping disabled items, and toggles with Space', async () => {
    const user = userEvent.setup();
    const { tool, onValueChange } = renderToolGroup();
    await user.tab();
    expect(tool('Sélection')).toHaveFocus();

    await user.keyboard('{ArrowRight}');
    expect(tool('Lame')).toHaveFocus();
    await user.keyboard('{ArrowRight}');
    expect(tool('Main')).toHaveFocus();
    expect(onValueChange).not.toHaveBeenCalled();

    await user.keyboard(' ');
    expect(onValueChange).toHaveBeenLastCalledWith('hand');
  });

  it('jumps to the ends with Home and End and wraps around', async () => {
    const user = userEvent.setup();
    const { tool } = renderToolGroup();
    await user.tab();
    await user.keyboard('{End}');
    expect(tool('Main')).toHaveFocus();
    await user.keyboard('{ArrowRight}');
    expect(tool('Sélection')).toHaveFocus();
    await user.keyboard('{End}{Home}');
    expect(tool('Sélection')).toHaveFocus();
  });

  it('is a single tab stop', async () => {
    const user = userEvent.setup();
    render(
      <>
        <ToggleGroup type="single" aria-label="Alignement">
          <ToggleGroupItem value="left">Gauche</ToggleGroupItem>
          <ToggleGroupItem value="right">Droite</ToggleGroupItem>
        </ToggleGroup>
        <button type="button">Après</button>
      </>,
    );
    await user.tab();
    expect(screen.getByRole('radio', { name: 'Gauche' })).toHaveFocus();
    await user.tab();
    expect(screen.getByRole('button', { name: 'Après' })).toHaveFocus();
  });
});

describe('ToggleGroup (multiple)', () => {
  it('toggles items independently as pressed buttons of a toolbar', async () => {
    const user = userEvent.setup();
    const onValueChange = vi.fn();
    render(
      <ToggleGroup type="multiple" aria-label="Style du texte" onValueChange={onValueChange}>
        <ToggleGroupItem value="bold">Gras</ToggleGroupItem>
        <ToggleGroupItem value="italic">Italique</ToggleGroupItem>
      </ToggleGroup>,
    );
    const bold = screen.getByRole('button', { name: 'Gras' });
    const italic = screen.getByRole('button', { name: 'Italique' });

    await user.click(bold);
    await user.click(italic);
    expect(bold).toHaveAttribute('aria-pressed', 'true');
    expect(italic).toHaveAttribute('aria-pressed', 'true');
    expect(onValueChange).toHaveBeenLastCalledWith(['bold', 'italic']);

    await user.click(bold);
    expect(bold).toHaveAttribute('aria-pressed', 'false');
    expect(onValueChange).toHaveBeenLastCalledWith(['italic']);
  });

  it('styles the items from the group variant and size', () => {
    render(
      <ToggleGroup type="multiple" variant="segmented" size="xs" aria-label="Pistes">
        <ToggleGroupItem value="video">V</ToggleGroupItem>
      </ToggleGroup>,
    );
    expect(screen.getByRole('toolbar', { name: 'Pistes' })).toHaveClass(
      'bg-control',
      'h-control-xs',
    );
    expect(screen.getByRole('button', { name: 'V' })).toHaveClass('h-full', 'text-xs');
  });
});
