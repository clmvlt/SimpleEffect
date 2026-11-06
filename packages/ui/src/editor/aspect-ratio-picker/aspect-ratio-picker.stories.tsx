import type { Meta, StoryObj } from '@storybook/react-vite';
import { useState } from 'react';

import {
  AspectRatioPicker,
  formatAspectRatio,
  type AspectRatio,
  type AspectRatioPickerProps,
} from './aspect-ratio-picker';

function ControlledPicker(
  props: Pick<AspectRatioPickerProps, 'disabled'> & { initial: AspectRatio },
) {
  const [value, setValue] = useState(props.initial);
  return (
    <div className="flex flex-col gap-2">
      <AspectRatioPicker
        label="Format de la séquence"
        customLabel="Personnalisé"
        customWidthLabel="Largeur du ratio"
        customHeightLabel="Hauteur du ratio"
        disabled={props.disabled}
        value={value}
        onChange={setValue}
      />
      <p className="text-xs text-text-muted">Valeur : {formatAspectRatio(value)}</p>
    </div>
  );
}

const meta = {
  title: 'Editor/AspectRatioPicker',
  component: ControlledPicker,
  args: { initial: { width: 16, height: 9 } },
} satisfies Meta<typeof ControlledPicker>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Presets with thumbnails to scale; arrow keys move the choice. */
export const Default: Story = {};

/** A ratio outside the presets shows the custom fields. */
export const Custom: Story = { args: { initial: { width: 21, height: 9 } } };

export const Disabled: Story = { args: { disabled: true } };
