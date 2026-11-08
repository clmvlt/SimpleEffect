import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { AiTaskCard, type AiTaskStatus } from './ai-task-card';

describe('AiTaskCard', () => {
  it.each<[AiTaskStatus, string]>([
    ['queued', 'Queued'],
    ['running', 'Running'],
    ['completed', 'Completed'],
    ['failed', 'Failed'],
    ['cancelled', 'Cancelled'],
  ])('labels the %s status', (status, label) => {
    render(<AiTaskCard status={status} title="Retirer les silences" />);
    expect(screen.getByRole('article')).toHaveAttribute('data-status', status);
    expect(screen.getByText(label)).toBeInTheDocument();
  });

  it('shows progress only while running', () => {
    const { rerender } = render(<AiTaskCard status="running" title="Tâche" progress={40} />);
    expect(screen.getByRole('progressbar', { name: 'Task progress' })).toBeInTheDocument();
    rerender(<AiTaskCard status="completed" title="Tâche" />);
    expect(screen.queryByRole('progressbar')).not.toBeInTheDocument();
  });

  it('renders the streaming log, error, cost and actions slots', () => {
    render(
      <AiTaskCard
        status="failed"
        title="Tâche"
        labels={{ log: 'Journal', status: { failed: 'Échec' } }}
        log="Analyse de la séquence…"
        error="Le modèle n'a pas répondu."
        cost="0,02 $"
        actions={<button type="button">Relancer</button>}
      />,
    );
    expect(screen.getByText('Échec')).toBeInTheDocument();
    expect(screen.getByRole('log', { name: 'Journal' })).toHaveTextContent(
      'Analyse de la séquence…',
    );
    expect(screen.getByRole('alert')).toHaveTextContent("Le modèle n'a pas répondu.");
    expect(screen.getByText('0,02 $')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Relancer' })).toBeInTheDocument();
  });

  it('shows its context line and extra content', () => {
    render(
      <AiTaskCard status="completed" title="Titre" context="00:51 → 00:54 · V2">
        <form aria-label="Suite" />
      </AiTaskCard>,
    );
    expect(screen.getByText('00:51 → 00:54 · V2')).toBeInTheDocument();
    expect(screen.getByRole('form', { name: 'Suite' })).toBeInTheDocument();
  });
});
