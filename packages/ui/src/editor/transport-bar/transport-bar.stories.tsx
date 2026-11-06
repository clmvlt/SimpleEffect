import type { Meta, StoryObj } from '@storybook/react-vite';
import { useState } from 'react';

import { TimecodeInput } from '../timecode';
import { createSampleFrameCodec } from '../timecode/timecode-sample-codec';
import { TransportBar, type TransportBarLabels } from './transport-bar';

const codec = createSampleFrameCodec(25);
const DURATION = 25 * 90;

const FRENCH_LABELS: TransportBarLabels = {
  toolbar: 'Lecture',
  goToStart: 'Aller au début',
  previousFrame: 'Image précédente',
  play: 'Lire',
  pause: 'Pause',
  stop: 'Arrêter',
  nextFrame: 'Image suivante',
  goToEnd: 'Aller à la fin',
  loop: 'Lire en boucle',
};

/** A transport bar driving a frame counter, as the viewer will drive the playhead. */
function ViewerTransport({ disabled }: { disabled?: boolean }) {
  const [frame, setFrame] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [looping, setLooping] = useState(false);
  const goTo = (next: number) => {
    setFrame(Math.min(DURATION, Math.max(0, next)));
  };
  return (
    <TransportBar
      className="w-[40rem] rounded-md border border-border bg-panel"
      labels={FRENCH_LABELS}
      disabled={disabled}
      playing={playing}
      looping={looping}
      onPlayPause={() => {
        setPlaying(!playing);
      }}
      onStop={() => {
        setPlaying(false);
        goTo(0);
      }}
      onPreviousFrame={() => {
        goTo(frame - 1);
      }}
      onNextFrame={() => {
        goTo(frame + 1);
      }}
      onGoToStart={() => {
        goTo(0);
      }}
      onGoToEnd={() => {
        goTo(DURATION);
      }}
      onLoopChange={setLooping}
      timecode={
        <TimecodeInput label="Position" size="xs" value={frame} {...codec} onCommit={goTo} />
      }
    />
  );
}

const meta = {
  title: 'Editor/TransportBar',
  component: ViewerTransport,
} satisfies Meta<typeof ViewerTransport>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Play/pause, frame steps, jumps and loop, with an editable timecode. */
export const Default: Story = {};

/** No media loaded yet. */
export const Disabled: Story = { args: { disabled: true } };
