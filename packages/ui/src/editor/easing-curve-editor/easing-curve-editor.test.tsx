import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { EasingCurveEditor } from './easing-curve-editor';
import {
  EASING_PRESETS,
  cubicBezier,
  fromClientPoint,
  matchEasingPreset,
  toCssTimingFunction,
  type EasingCurve,
} from './easing-math';

function Harness({
  initial,
  onChange,
  onCommit,
}: {
  initial: EasingCurve;
  onChange?: (value: EasingCurve) => void;
  onCommit?: (value: EasingCurve) => void;
}) {
  const [value, setValue] = useState(initial);
  return (
    <EasingCurveEditor
      value={value}
      onChange={(next) => {
        setValue(next);
        onChange?.(next);
      }}
      onCommit={onCommit}
    />
  );
}

/** The SVG is 100 × 100 px at the origin, so client pixels equal viewBox units. */
function mockSvgBox() {
  const svg = screen.getByRole('group', { name: 'Easing curve' });
  svg.getBoundingClientRect = () => DOMRect.fromRect({ x: 0, y: 0, width: 100, height: 100 });
}

describe('EasingCurveEditor', () => {
  it('moves the first handle to the dragged position, committing once', () => {
    const onChange = vi.fn();
    const onCommit = vi.fn();
    render(<Harness initial={EASING_PRESETS.linear} onChange={onChange} onCommit={onCommit} />);
    mockSvgBox();
    const handle = screen.getByRole('button', { name: /First control point/ });

    fireEvent.pointerDown(handle, { clientX: 8, clientY: 54, button: 0 });
    fireEvent.pointerMove(handle, { clientX: 29, clientY: 54 });
    fireEvent.pointerMove(handle, { clientX: 50, clientY: 16 });
    fireEvent.pointerUp(handle, { clientX: 50, clientY: 16 });

    // x = (50 − 8) / 84 = 0.5; y = 1.6 − (16 − 8) / 84 × 2.2 ≈ 1.390.
    expect(onChange).toHaveBeenCalledTimes(2);
    expect(onChange).toHaveBeenLastCalledWith(cubicBezier(0.5, 1.39, 1, 1));
    expect(onCommit).toHaveBeenCalledOnce();
    expect(onCommit).toHaveBeenCalledWith(cubicBezier(0.5, 1.39, 1, 1));
  });

  it('moves a focused handle with the arrow keys', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness initial={EASING_PRESETS.linear} onCommit={onCommit} />);
    screen.getByRole('button', { name: /Second control point/ }).focus();

    await user.keyboard('{ArrowLeft}');
    expect(onCommit).toHaveBeenLastCalledWith(cubicBezier(0, 0, 0.99, 1));
    await user.keyboard('{Shift>}{ArrowDown}{/Shift}');
    expect(onCommit).toHaveBeenLastCalledWith(cubicBezier(0, 0, 0.99, 0.9));
  });

  it('applies a preset and reflects the preset matching the value', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness initial={EASING_PRESETS.easeIn} onCommit={onCommit} />);
    expect(screen.getByRole('radio', { name: 'Ease in' })).toBeChecked();

    await user.click(screen.getByRole('radio', { name: 'Back' }));
    expect(onCommit).toHaveBeenCalledWith(EASING_PRESETS.back);
    expect(screen.getByRole('radio', { name: 'Back' })).toBeChecked();
  });

  it('shows a step and no handles for hold', async () => {
    const user = userEvent.setup();
    render(<Harness initial={EASING_PRESETS.linear} />);
    await user.click(screen.getByRole('radio', { name: 'Hold' }));
    expect(screen.queryByRole('button', { name: /control point/ })).not.toBeInTheDocument();
  });
});

describe('easing math', () => {
  it('maps pointer positions to clamped curve coordinates', () => {
    const rect = { left: 0, top: 0, width: 200, height: 200 };
    expect(fromClientPoint(16, 100, rect)).toEqual({ x: 0, y: 0.5 });
    expect(fromClientPoint(-50, -50, rect)).toEqual({ x: 0, y: 1.6 });
  });

  it('recognises presets and writes CSS timing functions', () => {
    expect(matchEasingPreset(cubicBezier(0.42, 0, 0.58, 1))).toBe('easeInOut');
    expect(matchEasingPreset(cubicBezier(0.3, 0.2, 0.5, 0.9))).toBeUndefined();
    expect(toCssTimingFunction(EASING_PRESETS.easeOut)).toBe('cubic-bezier(0, 0, 0.58, 1)');
    expect(toCssTimingFunction(EASING_PRESETS.hold)).toBe('steps(1, end)');
  });
});
