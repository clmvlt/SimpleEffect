import type { Meta, StoryObj } from '@storybook/react-vite';
import { useState } from 'react';

import { TimecodeDisplay, TimecodeInput, type TimecodeInputProps } from './timecode';
import { createSampleFrameCodec } from './timecode-sample-codec';

const codec = createSampleFrameCodec(25);

function ControlledTimecodeInput(props: Pick<TimecodeInputProps, 'size' | 'disabled' | 'label'>) {
  const [frames, setFrames] = useState(1803);
  return (
    <div className="flex flex-col items-start gap-2">
      <TimecodeInput {...props} {...codec} value={frames} onCommit={setFrames} />
      <p className="text-xs text-text-muted">
        Valeur : {frames} images — tapez par exemple 00:01:00:12 puis Entrée.
      </p>
    </div>
  );
}

const meta = {
  title: 'Editor/Timecode',
  component: ControlledTimecodeInput,
  args: { label: 'Position', size: 'sm', disabled: false },
  argTypes: { size: { control: 'inline-radio', options: ['xs', 'sm', 'md'] } },
} satisfies Meta<typeof ControlledTimecodeInput>;

export default meta;
type Story = StoryObj<typeof meta>;

/** HH:MM:SS:FF at 25 images/s; invalid text is flagged and Escape restores the value. */
export const Input: Story = {};

export const Disabled: Story = { args: { disabled: true } };

/** Read-only display, as in the transport bar. */
export const Display: Story = {
  render: () => (
    <div className="flex items-center gap-2">
      <TimecodeDisplay label="Position" value={1803} format={codec.format} />
      <span className="text-text-subtle">/</span>
      <TimecodeDisplay value={15000} format={codec.format} className="text-text-muted" />
    </div>
  ),
};
