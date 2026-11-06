import type { Meta, StoryObj } from '@storybook/react-vite';
import { useEffect, useState } from 'react';

import { Meter, type MeterProps } from './meter';
import { updatePeakHold, type PeakHoldState } from './meter-math';

const meta = {
  title: 'Editor/Meter',
  component: Meter,
  args: {
    levels: [-18, -9],
    peaks: [-12, -2],
    orientation: 'vertical',
    label: 'Niveau audio',
    channelLabels: ['Gauche', 'Droite'],
  },
  argTypes: { orientation: { control: 'inline-radio', options: ['vertical', 'horizontal'] } },
  decorators: [
    (Story, context) => (
      <div className={context.args.orientation === 'horizontal' ? 'w-72' : 'h-56'}>
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Meter>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Green to −12 dBFS, orange to −3, red above; white lines are held peaks. */
export const Vertical: Story = {};

export const Horizontal: Story = { args: { orientation: 'horizontal' } };

export const Silence: Story = {
  args: { levels: [Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY], peaks: undefined },
};

export const Clipping: Story = { args: { levels: [0, -1], peaks: [0, 0] } };

/** Pseudo-random levels at 30 updates/s with peak hold, as the audio engine will feed it. */
function LiveMeter(props: Omit<MeterProps, 'levels' | 'peaks'>) {
  const [state, setState] = useState<{ levels: number[]; hold: PeakHoldState | undefined }>({
    levels: [-60, -60],
    hold: undefined,
  });
  useEffect(() => {
    let phase = 0;
    const timer = setInterval(() => {
      phase += 0.15;
      const levels = [
        -22 + 14 * Math.sin(phase) * Math.sin(phase * 0.37),
        -20 + 15 * Math.sin(phase * 1.3),
      ];
      setState((current) => ({
        levels,
        hold: updatePeakHold(current.hold, levels, performance.now()),
      }));
    }, 33);
    return () => {
      clearInterval(timer);
    };
  }, []);
  return <Meter {...props} levels={state.levels} peaks={state.hold?.peaks} />;
}

export const Live: Story = { render: (args) => <LiveMeter {...args} /> };
