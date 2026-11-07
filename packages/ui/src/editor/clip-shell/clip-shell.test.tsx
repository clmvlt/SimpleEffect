import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { ClipBadge, ClipShell } from './clip-shell';

describe('ClipShell', () => {
  it('shows the label, badges and picture layer', () => {
    render(
      <ClipShell tone="audio" label="Interview.wav" badges={<ClipBadge label="IA" />}>
        <canvas data-testid="waveform" />
      </ClipShell>,
    );
    expect(screen.getByText('Interview.wav')).toBeInTheDocument();
    expect(screen.getByText('IA')).toBeInTheDocument();
    expect(screen.getByTestId('waveform')).toBeInTheDocument();
  });

  it('marks the tone and the selected and dimmed states', () => {
    const { container, rerender } = render(<ClipShell tone="graphic" label="Titre" />);
    const shell = container.firstElementChild;
    expect(shell).toHaveAttribute('data-tone', 'graphic');
    expect(shell).not.toHaveAttribute('data-selected');
    rerender(<ClipShell tone="graphic" label="Titre" selected dimmed />);
    expect(shell).toHaveAttribute('data-selected');
    expect(shell).toHaveClass('ring-accent', 'opacity-50');
  });

  it('describes a badge for screen readers', () => {
    render(<ClipBadge label="IA" description="Créé par l’IA" />);
    expect(screen.getByText('IA')).toHaveAttribute('title', 'Créé par l’IA');
    expect(screen.getByText('IA')).toHaveTextContent('IA — Créé par l’IA');
  });
});
