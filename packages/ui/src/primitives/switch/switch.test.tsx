import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Switch } from './switch';

describe('Switch', () => {
  it('toggles with a click and with Space', async () => {
    const user = userEvent.setup();
    const onCheckedChange = vi.fn();
    render(<Switch aria-label="Proxys de lecture" onCheckedChange={onCheckedChange} />);

    const toggle = screen.getByRole('switch', { name: 'Proxys de lecture' });
    expect(toggle).not.toBeChecked();
    await user.click(toggle);
    expect(toggle).toBeChecked();
    await user.keyboard(' ');
    expect(toggle).not.toBeChecked();
    expect(onCheckedChange.mock.calls).toEqual([[true], [false]]);
  });

  it('can be controlled', () => {
    render(<Switch aria-label="Proxys de lecture" checked />);
    expect(screen.getByRole('switch', { name: 'Proxys de lecture' })).toBeChecked();
  });

  it('ignores interaction when disabled', async () => {
    const user = userEvent.setup();
    render(<Switch aria-label="Proxys de lecture" disabled />);
    const toggle = screen.getByRole('switch', { name: 'Proxys de lecture' });
    await user.click(toggle);
    expect(toggle).not.toBeChecked();
  });
});
