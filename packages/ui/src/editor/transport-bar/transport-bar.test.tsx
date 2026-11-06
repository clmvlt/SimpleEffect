import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { TransportBar } from './transport-bar';

describe('TransportBar', () => {
  it('switches the main button between play and pause', async () => {
    const user = userEvent.setup();
    const onPlayPause = vi.fn();
    const { rerender } = render(<TransportBar playing={false} onPlayPause={onPlayPause} />);
    await user.click(screen.getByRole('button', { name: 'Play' }));
    expect(onPlayPause).toHaveBeenCalledOnce();

    rerender(<TransportBar playing onPlayPause={onPlayPause} />);
    expect(screen.getByRole('button', { name: 'Pause' })).toBeInTheDocument();
  });

  it('calls frame and jump callbacks, and disables missing ones', async () => {
    const user = userEvent.setup();
    const onNextFrame = vi.fn();
    const onGoToStart = vi.fn();
    render(
      <TransportBar
        playing={false}
        onPlayPause={() => undefined}
        onNextFrame={onNextFrame}
        onGoToStart={onGoToStart}
        labels={{ nextFrame: 'Image suivante', goToStart: 'Aller au début' }}
      />,
    );
    await user.click(screen.getByRole('button', { name: 'Image suivante' }));
    await user.click(screen.getByRole('button', { name: 'Aller au début' }));
    expect(onNextFrame).toHaveBeenCalledOnce();
    expect(onGoToStart).toHaveBeenCalledOnce();
    expect(screen.getByRole('button', { name: 'Previous frame' })).toBeDisabled();
  });

  it('toggles looping and shows the timecode slot', async () => {
    const user = userEvent.setup();
    const onLoopChange = vi.fn();
    render(
      <TransportBar
        playing={false}
        onPlayPause={() => undefined}
        looping={false}
        onLoopChange={onLoopChange}
        timecode={<span>00:00:01:00</span>}
      />,
    );
    const loop = screen.getByRole('button', { name: 'Loop playback' });
    expect(loop).toHaveAttribute('aria-pressed', 'false');
    await user.click(loop);
    expect(onLoopChange).toHaveBeenCalledWith(true);
    expect(screen.getByText('00:00:01:00')).toBeInTheDocument();
  });
});
