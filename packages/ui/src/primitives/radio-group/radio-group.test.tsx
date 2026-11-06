import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Label } from '../label';
import { RadioGroup, RadioGroupItem } from './radio-group';

function FormatGroup({ onValueChange }: { onValueChange?: (value: string) => void }) {
  return (
    <RadioGroup aria-label="Format" defaultValue="16:9" {...(onValueChange && { onValueChange })}>
      {['16:9', '9:16', '1:1'].map((format) => (
        <div key={format} className="flex items-center gap-2">
          <RadioGroupItem value={format} id={`format-${format}`} />
          <Label htmlFor={`format-${format}`}>{format}</Label>
        </div>
      ))}
    </RadioGroup>
  );
}

describe('RadioGroup', () => {
  it('names each option with its label and checks the default value', () => {
    render(<FormatGroup />);
    expect(screen.getByRole('radiogroup', { name: 'Format' })).toBeInTheDocument();
    expect(screen.getByRole('radio', { name: '16:9' })).toBeChecked();
    expect(screen.getByRole('radio', { name: '9:16' })).not.toBeChecked();
  });

  it('moves the selection with the arrow keys', async () => {
    const user = userEvent.setup();
    const onValueChange = vi.fn();
    render(<FormatGroup onValueChange={onValueChange} />);

    await user.tab();
    expect(screen.getByRole('radio', { name: '16:9' })).toHaveFocus();
    // Radix moves focus on a timer and selects only while the arrow key is still down, as it is
    // when a person presses it: hold the key, then release it.
    await user.keyboard('{ArrowDown>}{/ArrowDown}');
    expect(screen.getByRole('radio', { name: '9:16' })).toBeChecked();
    await user.keyboard('{ArrowDown>}{/ArrowDown}');
    expect(screen.getByRole('radio', { name: '1:1' })).toBeChecked();
    expect(onValueChange.mock.calls).toEqual([['9:16'], ['1:1']]);
  });

  it('selects an option on click', async () => {
    const user = userEvent.setup();
    render(<FormatGroup />);
    await user.click(screen.getByRole('radio', { name: '1:1' }));
    expect(screen.getByRole('radio', { name: '1:1' })).toBeChecked();
    expect(screen.getByRole('radio', { name: '16:9' })).not.toBeChecked();
  });
});
