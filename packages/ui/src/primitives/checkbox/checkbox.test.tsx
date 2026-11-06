import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Checkbox } from './checkbox';

describe('Checkbox', () => {
  it('toggles with a click and with Space', async () => {
    const user = userEvent.setup();
    const onCheckedChange = vi.fn();
    render(<Checkbox aria-label="Verrouiller la piste" onCheckedChange={onCheckedChange} />);

    const checkbox = screen.getByRole('checkbox', { name: 'Verrouiller la piste' });
    expect(checkbox).not.toBeChecked();
    await user.click(checkbox);
    expect(checkbox).toBeChecked();
    await user.keyboard(' ');
    expect(checkbox).not.toBeChecked();
    expect(onCheckedChange.mock.calls).toEqual([[true], [false]]);
  });

  it('is announced as mixed when indeterminate', () => {
    render(<Checkbox aria-label="Toutes les pistes" checked="indeterminate" />);
    const checkbox = screen.getByRole('checkbox', { name: 'Toutes les pistes' });
    expect(checkbox).toBePartiallyChecked();
    expect(checkbox).toHaveAttribute('data-state', 'indeterminate');
  });

  it('ignores interaction when disabled', async () => {
    const user = userEvent.setup();
    render(<Checkbox aria-label="Verrouiller" disabled />);
    const checkbox = screen.getByRole('checkbox', { name: 'Verrouiller' });
    await user.click(checkbox);
    expect(checkbox).not.toBeChecked();
  });
});
