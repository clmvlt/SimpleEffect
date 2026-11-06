import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { AspectRatioPicker, sameAspectRatio, type AspectRatio } from './aspect-ratio-picker';

function Harness({
  initial,
  onChange,
}: {
  initial: AspectRatio;
  onChange?: (value: AspectRatio) => void;
}) {
  const [value, setValue] = useState(initial);
  return (
    <AspectRatioPicker
      label="Format"
      customLabel="Personnalisé"
      customWidthLabel="Largeur"
      customHeightLabel="Hauteur"
      value={value}
      onChange={(next) => {
        setValue(next);
        onChange?.(next);
      }}
    />
  );
}

describe('AspectRatioPicker', () => {
  it('selects the preset matching the value, whatever its scale', () => {
    render(<Harness initial={{ width: 1920, height: 1080 }} />);
    expect(screen.getByRole('radio', { name: '16:9' })).toBeChecked();
  });

  it('changes preset with a click and with the arrow keys', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<Harness initial={{ width: 16, height: 9 }} onChange={onChange} />);

    await user.click(screen.getByRole('radio', { name: '1:1' }));
    expect(onChange).toHaveBeenLastCalledWith({ width: 1, height: 1 });

    // Radix radio groups move the selection on key down (like native radios).
    await user.keyboard('{ArrowRight>}{/ArrowRight}');
    expect(screen.getByRole('radio', { name: '4:5' })).toBeChecked();
    expect(onChange).toHaveBeenLastCalledWith({ width: 4, height: 5 });
  });

  it('shows custom fields for a ratio that matches no preset', () => {
    render(<Harness initial={{ width: 21, height: 9 }} />);
    expect(screen.getByRole('radio', { name: 'Personnalisé' })).toBeChecked();
    expect(screen.getByRole('spinbutton', { name: 'Largeur' })).toHaveValue('21');
    expect(screen.getByRole('spinbutton', { name: 'Hauteur' })).toHaveValue('9');
  });

  it('keeps custom chosen while the fields are edited', async () => {
    const user = userEvent.setup();
    render(<Harness initial={{ width: 16, height: 9 }} />);
    await user.click(screen.getByRole('radio', { name: 'Personnalisé' }));
    const width = screen.getByRole('spinbutton', { name: 'Largeur' });
    await user.click(width);
    await user.keyboard('{ArrowUp}');
    expect(width).toHaveValue('17');
    expect(screen.getByRole('radio', { name: 'Personnalisé' })).toBeChecked();
  });

  it('compares ratios by shape', () => {
    expect(sameAspectRatio({ width: 32, height: 18 }, { width: 16, height: 9 })).toBe(true);
    expect(sameAspectRatio({ width: 4, height: 5 }, { width: 5, height: 4 })).toBe(false);
  });
});
