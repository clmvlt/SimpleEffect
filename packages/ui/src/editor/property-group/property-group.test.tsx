import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { PropertyGroup, PropertyRow } from './property-group';

describe('PropertyGroup', () => {
  it('folds and unfolds its rows from the title button', async () => {
    const user = userEvent.setup();
    render(
      <PropertyGroup title="Transformation">
        <PropertyRow label="Opacité" htmlFor="opacity">
          <input id="opacity" />
        </PropertyRow>
      </PropertyGroup>,
    );
    const trigger = screen.getByRole('button', { name: 'Transformation' });
    expect(trigger).toHaveAttribute('aria-expanded', 'true');
    expect(screen.getByLabelText('Opacité')).toBeVisible();

    await user.click(trigger);
    expect(trigger).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByLabelText('Opacité')).not.toBeInTheDocument();

    trigger.focus();
    await user.keyboard('{Enter}');
    expect(screen.getByLabelText('Opacité')).toBeVisible();
  });
});

describe('PropertyRow', () => {
  it('names its single control with the label', () => {
    render(
      <PropertyRow label="Opacité" htmlFor="opacity">
        <input id="opacity" />
      </PropertyRow>,
    );
    expect(screen.getByRole('textbox', { name: 'Opacité' })).toBeInTheDocument();
  });

  it('groups compound controls under the label', () => {
    render(
      <PropertyRow label="Position">
        <input aria-label="X" />
        <input aria-label="Y" />
      </PropertyRow>,
    );
    const group = screen.getByRole('group', { name: 'Position' });
    expect(group).toContainElement(screen.getByRole('textbox', { name: 'X' }));
  });

  it('marks a modified value for sighted and screen reader users', () => {
    render(
      <PropertyRow
        label="Échelle"
        modified
        modifiedLabel="modifié"
        keyframe={<button type="button">◆</button>}
      >
        <span>100 %</span>
      </PropertyRow>,
    );
    expect(screen.getByText('(modifié)')).toHaveClass('sr-only');
    expect(screen.getByRole('button', { name: '◆' })).toBeInTheDocument();
  });
});
