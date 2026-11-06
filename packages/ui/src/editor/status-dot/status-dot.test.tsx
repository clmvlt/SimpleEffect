import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { StatusDot } from './status-dot';

describe('StatusDot', () => {
  it('is an image named by its label', () => {
    render(<StatusDot tone="success" label="Cœur connecté" />);
    expect(screen.getByRole('img', { name: 'Cœur connecté' })).toHaveAttribute(
      'data-tone',
      'success',
    );
  });

  it('is decorative without a label', () => {
    const { container } = render(<StatusDot pulse />);
    const dot = container.querySelector('span');
    expect(dot).toHaveAttribute('aria-hidden', 'true');
    expect(dot).toHaveClass('animate-pulse');
    expect(screen.queryByRole('img')).not.toBeInTheDocument();
  });
});
