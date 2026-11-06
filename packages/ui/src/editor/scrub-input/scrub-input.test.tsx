import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { ScrubInput, type ScrubInputProps } from './scrub-input';

type HarnessProps = Omit<ScrubInputProps, 'value' | 'onChange'> & {
  initialValue: number;
  onChange?: (value: number) => void;
};

/** Controlled wrapper, as the app uses it: `onChange` feeds the value back. */
function Harness({ initialValue, onChange, ...props }: HarnessProps) {
  const [value, setValue] = useState(initialValue);
  return (
    <ScrubInput
      label="Opacité"
      {...props}
      value={value}
      onChange={(next) => {
        setValue(next);
        onChange?.(next);
      }}
    />
  );
}

function getField() {
  return screen.getByRole('spinbutton', { name: 'Opacité' });
}

/** Presses at x = 0, moves through `positions` (with optional modifiers), then releases. */
function drag(
  element: HTMLElement,
  positions: readonly number[],
  modifiers: { shiftKey?: boolean; altKey?: boolean } = {},
) {
  fireEvent.pointerDown(element, { clientX: 0, clientY: 0, button: 0 });
  for (const clientX of positions) {
    fireEvent.pointerMove(element, { clientX, clientY: 0, ...modifiers });
  }
  fireEvent.pointerUp(element, { clientX: positions.at(-1) ?? 0, clientY: 0 });
}

describe('ScrubInput', () => {
  it('exposes a named spinbutton with its value, bounds and unit', () => {
    render(<Harness initialValue={50} min={0} max={100} unit="%" />);
    const field = getField();
    expect(field).toHaveAttribute('aria-valuenow', '50');
    expect(field).toHaveAttribute('aria-valuemin', '0');
    expect(field).toHaveAttribute('aria-valuemax', '100');
    expect(field).toHaveValue('50 %');
  });

  it('emits several onChange while dragging and exactly one onCommit on release', () => {
    const onChange = vi.fn();
    const onCommit = vi.fn();
    render(<Harness initialValue={10} onChange={onChange} onCommit={onCommit} />);

    // Default speed: 2 px per step of 1.
    drag(getField(), [4, 8, 12, 16]);

    expect(onChange.mock.calls.map(([value]) => value as number)).toEqual([12, 14, 16, 18]);
    expect(onCommit).toHaveBeenCalledTimes(1);
    expect(onCommit).toHaveBeenCalledWith(18);
    expect(getField()).toHaveValue('18');
  });

  it('scrubs ten times faster with Shift and ten times finer with Alt', () => {
    const onCommit = vi.fn();
    const { unmount } = render(<Harness initialValue={0} onCommit={onCommit} />);
    drag(getField(), [4], { shiftKey: true });
    expect(onCommit).toHaveBeenLastCalledWith(20);
    unmount();

    render(<Harness initialValue={0} onCommit={onCommit} />);
    drag(getField(), [6], { altKey: true });
    expect(onCommit).toHaveBeenLastCalledWith(0.3);
  });

  it('clamps dragged values to min and max', () => {
    const onCommit = vi.fn();
    render(<Harness initialValue={95} min={0} max={100} onCommit={onCommit} />);
    drag(getField(), [40]);
    expect(onCommit).toHaveBeenCalledWith(100);
  });

  it('does not commit a click without movement', () => {
    const onChange = vi.fn();
    const onCommit = vi.fn();
    render(<Harness initialValue={5} onChange={onChange} onCommit={onCommit} />);
    drag(getField(), [1]);
    expect(onChange).not.toHaveBeenCalled();
    expect(onCommit).not.toHaveBeenCalled();
  });

  it('steps with the arrow keys, committing each step', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness initialValue={5} max={20} onCommit={onCommit} />);
    await user.click(getField());

    await user.keyboard('{ArrowUp}');
    expect(onCommit).toHaveBeenLastCalledWith(6);
    await user.keyboard('{Shift>}{ArrowUp}{/Shift}');
    expect(onCommit).toHaveBeenLastCalledWith(16);
    await user.keyboard('{Shift>}{ArrowUp}{/Shift}');
    expect(onCommit).toHaveBeenLastCalledWith(20);
    await user.keyboard('{Alt>}{ArrowDown}{/Alt}');
    expect(onCommit).toHaveBeenLastCalledWith(19.9);
    expect(onCommit).toHaveBeenCalledTimes(4);
  });

  it('accepts a typed value after a double-click (decimal comma included)', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness initialValue={5} unit="px" onCommit={onCommit} />);

    await user.dblClick(getField());
    expect(getField()).toHaveValue('5');
    await user.keyboard('12,5{Enter}');

    expect(onCommit).toHaveBeenCalledTimes(1);
    expect(onCommit).toHaveBeenCalledWith(12.5);
    expect(getField()).toHaveValue('12.5 px');
  });

  it('starts typing directly when a digit is pressed and accepts it on blur', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness initialValue={5} onCommit={onCommit} />);
    await user.click(getField());

    await user.keyboard('42');
    expect(getField()).toHaveValue('42');
    await user.tab();
    expect(onCommit).toHaveBeenCalledWith(42);
  });

  it('cancels keyboard entry with Escape and rejects text that is not a number', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness initialValue={5} onCommit={onCommit} />);
    const field = getField();

    await user.click(field);
    await user.keyboard('{Enter}');
    await user.keyboard('99{Escape}');
    expect(field).toHaveValue('5');

    await user.keyboard('{Enter}abc{Enter}');
    expect(field).toHaveValue('5');
    expect(onCommit).not.toHaveBeenCalled();
  });

  it('ignores pointer and keyboard input when disabled', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<Harness initialValue={5} disabled onChange={onChange} />);
    drag(getField(), [20]);
    await user.keyboard('{ArrowUp}');
    expect(onChange).not.toHaveBeenCalled();
  });

  it('uses the injected formatter and parser', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(
      <Harness
        initialValue={0.5}
        step={0.01}
        format={(value) => String(Math.round(value * 100))}
        parse={(text) => (/^\d+$/.test(text) ? Number(text) / 100 : null)}
        onCommit={onCommit}
      />,
    );
    expect(getField()).toHaveValue('50');
    await user.dblClick(getField());
    await user.keyboard('75{Enter}');
    expect(onCommit).toHaveBeenCalledWith(0.75);
  });
});
