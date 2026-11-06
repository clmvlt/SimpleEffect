import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { KeyframeButton, type KeyframeState } from './keyframe-button';

function getToggle() {
  return screen.getByRole('button', { name: 'Clé' });
}

describe('KeyframeButton', () => {
  it.each<[KeyframeState, string]>([
    ['none', 'false'],
    ['betweenKeyframes', 'false'],
    ['onKeyframe', 'true'],
  ])('exposes the %s state', (state, pressed) => {
    render(<KeyframeButton state={state} label="Clé" onToggle={() => undefined} />);
    expect(getToggle()).toHaveAttribute('data-state', state);
    expect(getToggle()).toHaveAttribute('aria-pressed', pressed);
  });

  it('toggles the keyframe on click and from the keyboard', async () => {
    const user = userEvent.setup();
    const onToggle = vi.fn();
    render(<KeyframeButton state="none" label="Clé" onToggle={onToggle} />);
    await user.click(getToggle());
    await user.keyboard('{Enter}');
    expect(onToggle).toHaveBeenCalledTimes(2);
  });

  it('hides navigation while the property is not animated', () => {
    render(<KeyframeButton state="none" label="Clé" onToggle={() => undefined} />);
    expect(screen.queryByRole('button', { name: 'Previous keyframe' })).not.toBeInTheDocument();
  });

  it('navigates between keyframes and disables a missing direction', async () => {
    const user = userEvent.setup();
    const onPrevious = vi.fn();
    render(
      <KeyframeButton
        state="betweenKeyframes"
        label="Clé"
        previousLabel="Clé précédente"
        nextLabel="Clé suivante"
        onToggle={() => undefined}
        onPrevious={onPrevious}
      />,
    );
    await user.click(screen.getByRole('button', { name: 'Clé précédente' }));
    expect(onPrevious).toHaveBeenCalledOnce();
    expect(screen.getByRole('button', { name: 'Clé suivante' })).toBeDisabled();
  });
});
