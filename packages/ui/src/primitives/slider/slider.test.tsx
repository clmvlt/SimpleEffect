import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Slider } from './slider';

describe('Slider', () => {
  it('exposes one named slider with its value and bounds', () => {
    render(<Slider thumbLabels={['Volume']} defaultValue={[40]} min={0} max={100} />);
    const thumb = screen.getByRole('slider', { name: 'Volume' });
    expect(thumb).toHaveAttribute('aria-valuenow', '40');
    expect(thumb).toHaveAttribute('aria-valuemin', '0');
    expect(thumb).toHaveAttribute('aria-valuemax', '100');
  });

  it('changes its value with arrows, Page keys, Home and End', async () => {
    const user = userEvent.setup();
    const onValueChange = vi.fn();
    render(
      <Slider
        thumbLabels={['Volume']}
        defaultValue={[40]}
        step={1}
        onValueChange={onValueChange}
      />,
    );
    const thumb = screen.getByRole('slider', { name: 'Volume' });

    await user.tab();
    expect(thumb).toHaveFocus();
    await user.keyboard('{ArrowRight}');
    expect(thumb).toHaveAttribute('aria-valuenow', '41');
    await user.keyboard('{ArrowLeft}{ArrowLeft}');
    expect(thumb).toHaveAttribute('aria-valuenow', '39');
    await user.keyboard('{PageUp}');
    expect(thumb).toHaveAttribute('aria-valuenow', '49');
    await user.keyboard('{End}');
    expect(thumb).toHaveAttribute('aria-valuenow', '100');
    await user.keyboard('{Home}');
    expect(thumb).toHaveAttribute('aria-valuenow', '0');
    expect(onValueChange).toHaveBeenLastCalledWith([0]);
  });

  it('renders a range with two independently named thumbs', async () => {
    const user = userEvent.setup();
    const onValueChange = vi.fn();
    render(
      <Slider
        thumbLabels={['Début', 'Fin']}
        defaultValue={[20, 80]}
        onValueChange={onValueChange}
      />,
    );
    const start = screen.getByRole('slider', { name: 'Début' });
    const end = screen.getByRole('slider', { name: 'Fin' });
    expect(start).toHaveAttribute('aria-valuenow', '20');
    expect(end).toHaveAttribute('aria-valuenow', '80');

    await user.tab();
    await user.tab();
    expect(end).toHaveFocus();
    await user.keyboard('{ArrowLeft}');
    expect(end).toHaveAttribute('aria-valuenow', '79');
    expect(onValueChange).toHaveBeenLastCalledWith([20, 79]);
  });

  it('ignores the keyboard when disabled', async () => {
    const user = userEvent.setup();
    render(<Slider thumbLabels={['Volume']} defaultValue={[40]} disabled />);
    const thumb = screen.getByRole('slider', { name: 'Volume' });
    thumb.focus();
    await user.keyboard('{ArrowRight}');
    expect(thumb).toHaveAttribute('aria-valuenow', '40');
  });
});
