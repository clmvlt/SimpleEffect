import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Separator } from './separator';

describe('Separator', () => {
  it('is decorative and hidden from assistive technologies by default', () => {
    const { container } = render(<Separator />);
    expect(screen.queryByRole('separator')).not.toBeInTheDocument();
    expect(container.firstElementChild).toHaveAttribute('role', 'none');
    expect(container.firstElementChild).toHaveClass('h-px', 'w-full');
  });

  it('exposes its role and orientation when it is not decorative', () => {
    render(<Separator decorative={false} orientation="vertical" />);
    const separator = screen.getByRole('separator');
    expect(separator).toHaveAttribute('aria-orientation', 'vertical');
    expect(separator).toHaveClass('w-px');
  });
});
