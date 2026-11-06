import { render, screen } from '@testing-library/react';
import { Film } from 'lucide-react';
import { describe, expect, it } from 'vitest';

import { EmptyState } from './empty-state';

describe('EmptyState', () => {
  it('shows its title, description and actions', () => {
    render(
      <EmptyState
        icon={Film}
        title="Aucun média"
        description="Importez des fichiers."
        actions={<button type="button">Importer</button>}
      />,
    );
    expect(screen.getByText('Aucun média')).toBeInTheDocument();
    expect(screen.getByText('Importez des fichiers.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Importer' })).toBeInTheDocument();
  });

  it('renders only the title when nothing else is given', () => {
    const { container } = render(<EmptyState title="Rien" />);
    expect(container.querySelectorAll('p')).toHaveLength(1);
    expect(container.querySelector('svg')).toBeNull();
  });
});
