import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { TimecodeDisplay, TimecodeInput } from './timecode';
import { createSampleFrameCodec } from './timecode-sample-codec';

const codec = createSampleFrameCodec(25);

function Harness({ onCommit }: { onCommit: (value: number) => void }) {
  const [value, setValue] = useState(0);
  return (
    <TimecodeInput
      label="Position"
      value={value}
      {...codec}
      onCommit={(next) => {
        setValue(next);
        onCommit(next);
      }}
    />
  );
}

function getField() {
  return screen.getByRole('textbox', { name: 'Position' });
}

describe('TimecodeInput', () => {
  it('shows the formatted value and commits a valid typed timecode', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness onCommit={onCommit} />);
    expect(getField()).toHaveValue('00:00:00:00');

    await user.click(getField());
    await user.keyboard('00:00:02:10{Enter}');

    expect(onCommit).toHaveBeenCalledOnce();
    expect(onCommit).toHaveBeenCalledWith(60);
    expect(getField()).toHaveValue('00:00:02:10');
    expect(getField()).not.toHaveAttribute('aria-invalid');
  });

  it('flags an invalid timecode, keeps the text and does not commit', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness onCommit={onCommit} />);

    await user.click(getField());
    await user.keyboard('00:00:02:99{Enter}');

    expect(onCommit).not.toHaveBeenCalled();
    expect(getField()).toHaveAttribute('aria-invalid', 'true');
    expect(getField()).toHaveValue('00:00:02:99');

    await user.keyboard('{Escape}');
    expect(getField()).toHaveValue('00:00:00:00');
    expect(getField()).not.toHaveAttribute('aria-invalid');
  });

  it('drops an invalid draft on blur', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness onCommit={onCommit} />);
    await user.click(getField());
    await user.keyboard('pas un timecode');
    await user.tab();
    expect(getField()).toHaveValue('00:00:00:00');
    expect(onCommit).not.toHaveBeenCalled();
  });
});

describe('TimecodeDisplay', () => {
  it('formats the value with the injected formatter', () => {
    render(<TimecodeDisplay label="Durée" value={25 * 61 + 3} format={codec.format} />);
    expect(screen.getByRole('timer', { name: 'Durée' })).toHaveTextContent('00:01:01:03');
  });
});
