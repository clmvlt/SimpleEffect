import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Panel, PanelActions, PanelBody, PanelHeader, PanelTitle } from './panel';

describe('Panel', () => {
  it('is a region named by its title', () => {
    render(
      <Panel>
        <PanelHeader>
          <PanelTitle>Médias</PanelTitle>
          <PanelActions>
            <button type="button">Importer</button>
          </PanelActions>
        </PanelHeader>
        <PanelBody>Contenu</PanelBody>
      </Panel>,
    );
    const region = screen.getByRole('region', { name: 'Médias' });
    expect(region).toHaveTextContent('Contenu');
    expect(screen.getByRole('heading', { name: 'Médias' })).toBeInTheDocument();
  });

  it('takes its name from aria-label when it has no title', () => {
    render(
      <Panel aria-label="Inspecteur">
        <PanelBody scrollable={false}>Contenu</PanelBody>
      </Panel>,
    );
    const region = screen.getByRole('region', { name: 'Inspecteur' });
    expect(region).not.toHaveAttribute('aria-labelledby');
  });
});
