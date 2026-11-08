import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { PromptComposer, type PromptScope } from './prompt-composer';

const SCOPES: PromptScope[] = [
  { id: 'clip-1', label: 'Interview A' },
  { id: 'range', label: '00:12 → 00:45' },
];

function Harness({ onSubmit, busy }: { onSubmit: (value: string) => void; busy?: boolean }) {
  const [value, setValue] = useState('');
  const [scopes, setScopes] = useState(SCOPES);
  return (
    <PromptComposer
      label="Demande"
      submitLabel="Envoyer"
      removeScopeLabel={(scopeLabel) => `Retirer ${scopeLabel}`}
      value={value}
      onValueChange={setValue}
      onSubmit={onSubmit}
      scopes={scopes}
      onRemoveScope={(id) => {
        setScopes(scopes.filter((scope) => scope.id !== id));
      }}
      busy={busy}
    />
  );
}

function getField() {
  return screen.getByRole('textbox', { name: 'Demande' });
}

describe('PromptComposer', () => {
  it('sends with Ctrl+Enter or ⌘+Enter, and Enter alone adds a new line', async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();
    render(<Harness onSubmit={onSubmit} />);
    await user.click(getField());

    await user.keyboard('Coupe les silences{Enter}partout');
    expect(onSubmit).not.toHaveBeenCalled();
    expect(getField()).toHaveValue('Coupe les silences\npartout');

    await user.keyboard('{Control>}{Enter}{/Control}');
    expect(onSubmit).toHaveBeenCalledWith('Coupe les silences\npartout');
    await user.keyboard('{Meta>}{Enter}{/Meta}');
    expect(onSubmit).toHaveBeenCalledTimes(2);
  });

  it('does not send a blank prompt or while busy', async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();
    const { unmount } = render(<Harness onSubmit={onSubmit} />);
    await user.click(getField());
    await user.keyboard('   {Control>}{Enter}{/Control}');
    expect(onSubmit).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: 'Envoyer' })).toBeDisabled();
    unmount();

    render(<Harness onSubmit={onSubmit} busy />);
    await user.click(getField());
    await user.keyboard('Go{Control>}{Enter}{/Control}');
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('sends with the button', async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();
    render(<Harness onSubmit={onSubmit} />);
    await user.type(getField(), 'Ajoute des sous-titres');
    await user.click(screen.getByRole('button', { name: 'Envoyer' }));
    expect(onSubmit).toHaveBeenCalledWith('Ajoute des sous-titres');
  });

  it('removes a scope chip with its button', async () => {
    const user = userEvent.setup();
    render(<Harness onSubmit={() => undefined} />);
    await user.click(screen.getByRole('button', { name: 'Retirer Interview A' }));
    expect(screen.queryByText('Interview A')).not.toBeInTheDocument();
    expect(screen.getByText('00:12 → 00:45')).toBeInTheDocument();
  });

  it('removes the last scope with Backspace in an empty field', async () => {
    const user = userEvent.setup();
    render(<Harness onSubmit={() => undefined} />);
    await user.click(getField());
    await user.keyboard('{Backspace}');
    expect(screen.queryByText('00:12 → 00:45')).not.toBeInTheDocument();
    expect(screen.getByText('Interview A')).toBeInTheDocument();
  });

  it('keeps fixed chips, skips them with Backspace and shows the scope control', async () => {
    const user = userEvent.setup();
    const onRemoveScope = vi.fn();
    render(
      <PromptComposer
        label="Demande"
        value=""
        onValueChange={() => undefined}
        onSubmit={() => undefined}
        scopes={[
          { id: 'range', label: '00:51 → 00:54', detected: true, description: 'Lu dans le texte' },
          { id: 'sequence', label: 'Séquence', removable: false },
        ]}
        onRemoveScope={onRemoveScope}
        scopeAction={<button type="button">Portée</button>}
        notice="Reprendre la session"
      />,
    );
    expect(screen.queryByRole('button', { name: 'Remove Séquence' })).not.toBeInTheDocument();
    expect(screen.getByText('00:51 → 00:54').closest('li')).toHaveAttribute('data-detected');
    expect(screen.getByRole('button', { name: 'Portée' })).toBeInTheDocument();
    expect(screen.getByText('Reprendre la session')).toBeInTheDocument();
    await user.click(getField());
    await user.keyboard('{Backspace}');
    expect(onRemoveScope).toHaveBeenCalledWith('range');
  });

  it('browses a history with ↑ on the first line and ↓ on the last one', async () => {
    const user = userEvent.setup();
    const history = ['Premier', 'Second'];
    function HistoryHarness() {
      const [value, setValue] = useState('');
      const [cursor, setCursor] = useState<number | null>(null);
      return (
        <PromptComposer
          label="Demande"
          value={value}
          onValueChange={setValue}
          onSubmit={() => undefined}
          onNavigateHistory={(direction) => {
            const next =
              direction === 'previous' ? (cursor ?? history.length) - 1 : (cursor ?? -1) + 1;
            if (next < 0 || next >= history.length) return false;
            setCursor(next);
            setValue(history[next] ?? '');
            return true;
          }}
        />
      );
    }
    render(<HistoryHarness />);
    await user.click(getField());
    await user.keyboard('{ArrowUp}');
    expect(getField()).toHaveValue('Second');
    await user.keyboard('{ArrowUp}');
    expect(getField()).toHaveValue('Premier');
    await user.keyboard('{ArrowDown}');
    expect(getField()).toHaveValue('Second');
  });
});
