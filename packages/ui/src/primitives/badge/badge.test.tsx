import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Badge } from './badge';

describe('Badge', () => {
  it('renders a neutral, small, rounded label by default', () => {
    render(<Badge>3 clips</Badge>);
    expect(screen.getByText('3 clips')).toHaveClass('bg-control', 'h-5', 'rounded-sm');
  });

  it('uses the muted fill and a readable text colour of semantic variants', () => {
    render(<Badge variant="danger">Erreur</Badge>);
    expect(screen.getByText('Erreur')).toHaveClass('bg-danger-muted', 'text-danger');
  });

  it('uses the item-kind colours and the pill shape', () => {
    render(
      <Badge variant="audio" shape="pill" size="xs">
        Audio
      </Badge>,
    );
    const badge = screen.getByText('Audio');
    expect(badge).toHaveClass('bg-item-audio-muted', 'text-item-audio-strong', 'rounded-full');
    expect(badge).toHaveClass('text-2xs', 'px-1.5');
  });
});
