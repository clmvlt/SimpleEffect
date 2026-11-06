import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Spinner } from './spinner';

describe('Spinner', () => {
  it('announces its label as a status', () => {
    render(<Spinner label="Chargement…" />);
    expect(screen.getByRole('status', { name: 'Chargement…' })).toBeInTheDocument();
  });

  it('is decorative without a label', () => {
    const { container } = render(<Spinner />);
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
    expect(container.firstElementChild).toHaveAttribute('aria-hidden', 'true');
  });
});
