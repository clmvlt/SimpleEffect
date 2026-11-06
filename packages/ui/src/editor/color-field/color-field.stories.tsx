import type { Meta, StoryObj } from '@storybook/react-vite';
import { useState } from 'react';

import { ColorField, type ColorFieldProps } from './color-field';
import { hexFromColor } from './color-model';

// Sample user colours are computed: hex literals are reserved to token files (raw-colour guard).
const RECENT = [25, 65, 145, 250, 300].map((hue) =>
  hexFromColor({ lightness: 0.72, chroma: 0.15, hue }),
);
const WHITE = hexFromColor({ lightness: 1, chroma: 0, hue: 0 });

const FRENCH_LABELS = {
  hex: 'Hexadécimal',
  lightness: 'Luminosité',
  chroma: 'Chroma',
  hue: 'Teinte',
  eyeDropper: 'Prélever une couleur à l’écran',
  recent: 'Couleurs récentes',
};

function ControlledColorField(
  props: Pick<ColorFieldProps, 'disabled' | 'recentColors'> & { initial: string },
) {
  const { initial, ...rest } = props;
  const [value, setValue] = useState(initial);
  const [committed, setCommitted] = useState(initial);
  return (
    <div className="flex flex-col items-start gap-2">
      <ColorField
        {...rest}
        label="Couleur du texte"
        labels={FRENCH_LABELS}
        value={value}
        onChange={setValue}
        onCommit={setCommitted}
      />
      <p className="text-xs text-text-muted">Dernier commit : {committed}</p>
    </div>
  );
}

const meta = {
  title: 'Editor/ColorField',
  component: ControlledColorField,
  args: { initial: RECENT[3] ?? WHITE, recentColors: RECENT },
} satisfies Meta<typeof ControlledColorField>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Click the swatch: OKLCH sliders and fields, hex, eyedropper (Chromium) and recent colours. */
export const Default: Story = {};

/** A grey keeps its hue slider where it was while chroma is 0. */
export const Grey: Story = {
  args: { initial: hexFromColor({ lightness: 0.6, chroma: 0, hue: 0 }) },
};

export const WithoutRecentColors: Story = { args: { recentColors: [] } };

export const Disabled: Story = { args: { disabled: true } };
