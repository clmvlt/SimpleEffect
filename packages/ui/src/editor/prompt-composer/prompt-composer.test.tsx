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
});
