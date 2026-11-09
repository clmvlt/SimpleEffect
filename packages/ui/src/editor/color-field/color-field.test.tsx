import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { toHex } from '../../tokens/color-math';
import { ColorField } from './color-field';
import { colorFromHex, hexFromColor } from './color-model';

// Colours are built from channels: hex literals are reserved to token files (raw-colour guard).
const RED = toHex({ red: 1, green: 0, blue: 0 });
const GREY = toHex({ red: 0.5, green: 0.5, blue: 0.5 });
const SKY = toHex({ red: 0.2, green: 0.6, blue: 1 });

function Harness({
  initial,
  onChange,
  onCommit,
  recentColors,
  palette,
}: {
  initial: string;
  onChange?: (hex: string) => void;
  onCommit?: (hex: string) => void;
  recentColors?: string[];
  palette?: { color: string; label: string }[];
}) {
  const [value, setValue] = useState(initial);
  return (
    <ColorField
      label="Couleur du texte"
      value={value}
      recentColors={recentColors}
      palette={palette}
      labels={{
        hex: 'Hex',
        hue: 'Teinte',
        recent: 'Récentes',
        eyeDropper: 'Pipette',
        palette: 'Brand kit',
      }}
      onChange={(next) => {
        setValue(next);
        onChange?.(next);
      }}
      onCommit={onCommit}
    />
  );
}

async function openPicker(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole('button', { name: 'Couleur du texte' }));
}

afterEach(() => {
  Reflect.deleteProperty(globalThis, 'EyeDropper');
});

describe('ColorField', () => {
  it('shows the colour on its swatch and opens the picker', async () => {
    const user = userEvent.setup();
    render(<Harness initial={RED} />);
    expect(screen.getByRole('button', { name: 'Couleur du texte' })).toHaveTextContent(RED);
    await openPicker(user);
    expect(screen.getByRole('textbox', { name: 'Hex' })).toHaveValue(RED);
    expect(screen.getByRole('slider', { name: 'Teinte' })).toBeInTheDocument();
  });

  it('accepts a typed hex and commits it once', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness initial={RED} onCommit={onCommit} />);
    await openPicker(user);
    const hexField = screen.getByRole('textbox', { name: 'Hex' });

    await user.clear(hexField);
    await user.type(hexField, `${SKY.toUpperCase().slice(1)}{Enter}`);

    expect(onCommit).toHaveBeenCalledOnce();
    expect(onCommit).toHaveBeenCalledWith(SKY);
    expect(hexField).toHaveValue(SKY);
  });

  it('flags a hex that is not a colour', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness initial={RED} onCommit={onCommit} />);
    await openPicker(user);
    const hexField = screen.getByRole('textbox', { name: 'Hex' });
    await user.clear(hexField);
    await user.type(hexField, 'zz{Enter}');
    expect(hexField).toHaveAttribute('aria-invalid', 'true');
    expect(onCommit).not.toHaveBeenCalled();
  });

  it('changes and commits the hue with the keyboard', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    const onCommit = vi.fn();
    render(<Harness initial={RED} onChange={onChange} onCommit={onCommit} />);
    await openPicker(user);
    screen.getByRole('slider', { name: 'Teinte' }).focus();
    await user.keyboard('{PageUp}');
    expect(onChange).toHaveBeenCalled();
    expect(onCommit).toHaveBeenCalledTimes(1);
    expect(onCommit.mock.calls[0]?.[0]).not.toBe(RED);
  });

  it('picks a recent colour in one click', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(<Harness initial={RED} onCommit={onCommit} recentColors={[SKY, GREY]} />);
    await openPicker(user);
    const recent = screen.getByRole('group', { name: 'Récentes' });
    expect(recent).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: GREY }));
    expect(onCommit).toHaveBeenCalledWith(GREY);
  });

  it('picks a named palette colour in one click', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    render(
      <Harness
        initial={RED}
        onCommit={onCommit}
        palette={[
          { color: SKY, label: 'Primaire' },
          { color: GREY, label: 'Fond' },
        ]}
      />,
    );
    await openPicker(user);
    expect(screen.getByRole('group', { name: 'Brand kit' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: `Fond ${GREY}` }));
    expect(onCommit).toHaveBeenCalledWith(GREY);
  });

  it('offers the eyedropper only when the platform has one', async () => {
    const user = userEvent.setup();
    const onCommit = vi.fn();
    const { unmount } = render(<Harness initial={RED} onCommit={onCommit} />);
    await openPicker(user);
    expect(screen.queryByRole('button', { name: 'Pipette' })).not.toBeInTheDocument();
    unmount();

    Reflect.set(
      globalThis,
      'EyeDropper',
      class {
        open() {
          return Promise.resolve({ sRGBHex: SKY });
        }
      },
    );
    render(<Harness initial={RED} onCommit={onCommit} />);
    await openPicker(user);
    await user.click(screen.getByRole('button', { name: 'Pipette' }));
    expect(onCommit).toHaveBeenCalledWith(SKY);
  });
});

describe('color model', () => {
  it('keeps the current hue for greys', () => {
    expect(colorFromHex(GREY, 200)?.hue).toBe(200);
    expect(colorFromHex('not a colour')).toBeNull();
  });

  it('round-trips hex values', () => {
    const color = colorFromHex(SKY);
    expect(color === null ? null : hexFromColor(color)).toBe(SKY);
  });
});
