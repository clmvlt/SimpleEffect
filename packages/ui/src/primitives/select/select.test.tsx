import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './index';

function FrameRateSelect({ onValueChange }: { onValueChange?: (value: string) => void }) {
  return (
    <Select defaultValue="25" {...(onValueChange && { onValueChange })}>
      <SelectTrigger aria-label="Cadence">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="24">24 i/s</SelectItem>
        <SelectItem value="25">25 i/s</SelectItem>
        <SelectItem value="30">30 i/s</SelectItem>
        <SelectItem value="60" disabled>
          60 i/s
        </SelectItem>
      </SelectContent>
    </Select>
  );
}

describe('Select', () => {
  it('shows the selected value in a closed combobox', () => {
    render(<FrameRateSelect />);
    const trigger = screen.getByRole('combobox', { name: 'Cadence' });
    expect(trigger).toHaveTextContent('25 i/s');
    expect(trigger).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
  });

  it('opens on click and selects an option with the pointer', async () => {
    const user = userEvent.setup();
    const onValueChange = vi.fn();
    render(<FrameRateSelect onValueChange={onValueChange} />);

    await user.click(screen.getByRole('combobox', { name: 'Cadence' }));
    expect(screen.getByRole('listbox')).toBeInTheDocument();
    expect(screen.getByRole('option', { name: '25 i/s' })).toHaveAttribute('aria-selected', 'true');
    await user.click(screen.getByRole('option', { name: '30 i/s' }));

    expect(onValueChange).toHaveBeenCalledWith('30');
    expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: 'Cadence' })).toHaveTextContent('30 i/s');
  });

  it('opens with the keyboard, moves with the arrows and selects with Enter', async () => {
    const user = userEvent.setup();
    const onValueChange = vi.fn();
    render(<FrameRateSelect onValueChange={onValueChange} />);

    await user.tab();
    expect(screen.getByRole('combobox', { name: 'Cadence' })).toHaveFocus();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(screen.getByRole('option', { name: '25 i/s' })).toHaveFocus();
    });
    await user.keyboard('{ArrowDown}');
    expect(screen.getByRole('option', { name: '30 i/s' })).toHaveFocus();
    // The disabled option is skipped: there is nothing below.
    await user.keyboard('{ArrowDown}');
    expect(screen.getByRole('option', { name: '30 i/s' })).toHaveFocus();
    await user.keyboard('{ArrowUp}{ArrowUp}{Enter}');

    expect(onValueChange).toHaveBeenCalledWith('24');
    await waitFor(() => {
      expect(screen.getByRole('combobox', { name: 'Cadence' })).toHaveFocus();
    });
  });

  it('closes with Escape without changing the value and returns focus to the trigger', async () => {
    const user = userEvent.setup();
    const onValueChange = vi.fn();
    render(<FrameRateSelect onValueChange={onValueChange} />);

    const trigger = screen.getByRole('combobox', { name: 'Cadence' });
    trigger.focus();
    await user.keyboard('{ArrowDown}');
    expect(await screen.findByRole('listbox')).toBeInTheDocument();
    await user.keyboard('{Escape}');

    expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
    await waitFor(() => {
      expect(trigger).toHaveFocus();
    });
    expect(trigger).toHaveTextContent('25 i/s');
    expect(onValueChange).not.toHaveBeenCalled();
  });
});
