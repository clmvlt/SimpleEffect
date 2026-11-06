import type { Meta, StoryObj } from '@storybook/react-vite';
import { useState } from 'react';

import { KeyframeButton, type KeyframeButtonProps, type KeyframeState } from './keyframe-button';

/** Cycles through the states as an inspector would. */
function InteractiveKeyframeButton(props: KeyframeButtonProps) {
  const [state, setState] = useState<KeyframeState>('none');
  return (
    <KeyframeButton
      {...props}
      state={state}
      onToggle={() => {
        setState(state === 'onKeyframe' ? 'betweenKeyframes' : 'onKeyframe');
      }}
    />
  );
}

const meta = {
  title: 'Editor/KeyframeButton',
  component: KeyframeButton,
  args: {
    state: 'none',
    label: 'Clé d’opacité',
    previousLabel: 'Clé précédente',
    nextLabel: 'Clé suivante',
    onToggle: () => undefined,
    onPrevious: () => undefined,
    onNext: () => undefined,
  },
  argTypes: {
    state: { control: 'inline-radio', options: ['none', 'betweenKeyframes', 'onKeyframe'] },
  },
} satisfies Meta<typeof KeyframeButton>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Not animated: a hollow diamond, no navigation. */
export const None: Story = {};

/** Animated, playhead between two keyframes. */
export const BetweenKeyframes: Story = { args: { state: 'betweenKeyframes' } };

/** Playhead on a keyframe: filled and pressed. */
export const OnKeyframe: Story = { args: { state: 'onKeyframe' } };

/** First keyframe reached: no previous one. */
export const NoPrevious: Story = {
  render: (args) => <KeyframeButton {...args} state="onKeyframe" onPrevious={undefined} />,
};

/** Click to cycle through the states as an inspector would. */
export const Interactive: Story = {
  render: (args) => <InteractiveKeyframeButton {...args} />,
};
