import type { Meta, StoryObj } from '@storybook/react-vite';
import { useState } from 'react';

import { EasingCurveEditor } from './easing-curve-editor';
import { EASING_PRESETS, toCssTimingFunction, type EasingCurve } from './easing-math';

const FRENCH_LABELS = {
  editor: 'Courbe d’interpolation',
  presets: 'Préréglages',
  firstHandle: 'Premier point de contrôle',
  secondHandle: 'Second point de contrôle',
  preset: {
    linear: 'Linéaire',
    easeIn: 'Entrée',
    easeOut: 'Sortie',
    easeInOut: 'Entrée-sortie',
    back: 'Rebond',
    hold: 'Maintien',
  },
};

function ControlledEasingEditor({ initial }: { initial: EasingCurve }) {
  const [value, setValue] = useState(initial);
  return (
    <div className="flex w-64 flex-col gap-2">
      <EasingCurveEditor labels={FRENCH_LABELS} value={value} onChange={setValue} />
      <p className="timecode text-2xs text-text-muted">{toCssTimingFunction(value)}</p>
    </div>
  );
}

const meta = {
  title: 'Editor/EasingCurveEditor',
  component: ControlledEasingEditor,
  args: { initial: EASING_PRESETS.easeInOut },
} satisfies Meta<typeof ControlledEasingEditor>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Drag the yellow points (or focus them and use the arrows); the dot previews the motion. */
export const Default: Story = {};

/** Overshoot: control points above 1 make the value pass its target before settling. */
export const Back: Story = { args: { initial: EASING_PRESETS.back } };

/** Hold: the value jumps at the next keyframe. */
export const Hold: Story = { args: { initial: EASING_PRESETS.hold } };
