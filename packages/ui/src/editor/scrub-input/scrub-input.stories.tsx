import type { Meta, StoryObj } from '@storybook/react-vite';
import { useState } from 'react';

import { ScrubInput, type ScrubInputProps } from './scrub-input';

/** Stories keep the value in local state, as the app keeps it in its store. */
function ControlledScrubInput(
  props: Omit<ScrubInputProps, 'value' | 'onChange'> & { initialValue: number },
) {
  const { initialValue, ...rest } = props;
  const [value, setValue] = useState(initialValue);
  const [committed, setCommitted] = useState(initialValue);
  return (
    <div className="flex w-48 flex-col gap-2">
      <ScrubInput
        {...rest}
        value={value}
        onChange={setValue}
        onCommit={(next) => {
          setCommitted(next);
        }}
      />
      <p className="text-xs text-text-muted">Dernier commit : {committed}</p>
    </div>
  );
}

const meta = {
  title: 'Editor/ScrubInput',
  component: ControlledScrubInput,
  args: { label: 'Position X', initialValue: 120, unit: 'px', size: 'sm' },
  argTypes: { size: { control: 'inline-radio', options: ['xs', 'sm', 'md'] } },
} satisfies Meta<typeof ControlledScrubInput>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Drag horizontally (Shift = fast, Alt = fine), arrows to step, double-click to type. */
export const Default: Story = {};

/** Bounded value with a fractional step. */
export const Bounded: Story = {
  args: { label: 'Opacité', initialValue: 80, min: 0, max: 100, step: 0.5, unit: '%' },
};

/** A formatter and parser injected by the caller: a 0–1 value shown as a percentage. */
export const CustomFormat: Story = {
  args: {
    label: 'Échelle',
    initialValue: 1,
    step: 0.01,
    unit: '%',
    format: (value: number) => String(Math.round(value * 100)),
    parse: (text: string) => {
      const percent = Number(text.replace('%', '').replace(',', '.'));
      return Number.isFinite(percent) ? percent / 100 : null;
    },
  },
};

export const Rotation: Story = { args: { label: 'Rotation', initialValue: 0, unit: '°' } };

export const Sizes: Story = {
  render: () => (
    <div className="flex w-48 flex-col gap-2">
      <ScrubInput label="Très petit" size="xs" value={12} onChange={() => undefined} />
      <ScrubInput label="Petit" size="sm" value={12} onChange={() => undefined} />
      <ScrubInput label="Moyen" size="md" value={12} onChange={() => undefined} />
    </div>
  ),
};

export const Disabled: Story = { args: { disabled: true } };
