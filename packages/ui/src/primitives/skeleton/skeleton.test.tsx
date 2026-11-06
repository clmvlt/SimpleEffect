import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Skeleton } from './skeleton';

describe('Skeleton', () => {
  it('is hidden from assistive technologies and pulses', () => {
    const { container } = render(<Skeleton className="h-4 w-32" />);
    const skeleton = container.firstElementChild;
    expect(skeleton).toHaveAttribute('aria-hidden', 'true');
    expect(skeleton).toHaveClass('animate-pulse', 'bg-control', 'h-4', 'w-32');
  });

  it('lets callers override the shape', () => {
    const { container } = render(<Skeleton className="rounded-full" />);
    expect(container.firstElementChild).toHaveClass('rounded-full');
    expect(container.firstElementChild).not.toHaveClass('rounded-md');
  });
});
