import { render, screen } from '@testing-library/react';
import { Scissors } from 'lucide-react';
import { describe, expect, it } from 'vitest';

import { Icon } from './icon';

describe('Icon', () => {
  it('is decorative and hidden from assistive technologies by default', () => {
    const { container } = render(<Icon icon={Scissors} />);
    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('aria-hidden', 'true');
    expect(svg).toHaveAttribute('width', '16');
  });

  it('becomes an image with an accessible name when labelled', () => {
    render(<Icon icon={Scissors} label="Couper" size="lg" />);
    const image = screen.getByRole('img', { name: 'Couper' });
    expect(image).not.toHaveAttribute('aria-hidden');
    expect(image).toHaveAttribute('width', '20');
  });
});
