import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Progress } from './progress';

describe('Progress', () => {
  it('announces its value as a labelled progress bar', () => {
    render(<Progress value={42} aria-label="Export" />);
    const bar = screen.getByRole('progressbar', { name: 'Export' });
    expect(bar).toHaveAttribute('aria-valuenow', '42');
    expect(bar).toHaveAttribute('aria-valuemax', '100');
    expect(bar).toHaveAttribute('aria-valuetext', '42%');
  });

  it('moves the indicator to the completed fraction of a custom maximum', () => {
    render(<Progress value={3} max={4} aria-label="Analyse" />);
    const indicator = screen.getByRole('progressbar').firstElementChild;
    expect(indicator).toHaveStyle({ transform: 'translateX(-25%)' });
  });

  it('announces no value and sweeps a segment when indeterminate', () => {
    render(<Progress value={null} aria-label="Préparation" tone="ai" />);
    const bar = screen.getByRole('progressbar', { name: 'Préparation' });
    expect(bar).not.toHaveAttribute('aria-valuenow');
    expect(bar).toHaveAttribute('data-state', 'indeterminate');
    expect(bar.firstElementChild).toHaveClass('animate-progress-indeterminate', 'bg-ai');
    expect(bar.firstElementChild).not.toHaveAttribute('style');
  });
});
