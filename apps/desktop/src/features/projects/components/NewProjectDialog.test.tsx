/** @vitest-environment jsdom */
import type { CreateProjectRequest, NewProjectDefaults } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { NewProjectDialog } from './NewProjectDialog';

const DEFAULTS: NewProjectDefaults = {
  parentDir: '/home/me/Documents/SimpleEffect',
  format: {
    width: 1920,
    height: 1080,
    frameRate: { numerator: 30, denominator: 1 },
    sampleRate: 48_000,
  },
  presets: [],
  frameRates: [
    { numerator: 25, denominator: 1 },
    { numerator: 30, denominator: 1 },
  ],
};

function renderDialog(onCreate = vi.fn(() => Promise.resolve(true))) {
  render(
    <I18nextProvider i18n={createI18n()}>
      <TooltipProvider>
        <NewProjectDialog
          open
          onOpenChange={vi.fn()}
          loadDefaults={() => Promise.resolve(DEFAULTS)}
          chooseFolder={() => Promise.resolve('/elsewhere')}
          onCreate={onCreate}
          onError={vi.fn()}
        />
      </TooltipProvider>
    </I18nextProvider>,
  );
  return onCreate;
}

describe('NewProjectDialog', () => {
  it('refuses an empty name and explains why', async () => {
    const user = userEvent.setup();
    const onCreate = renderDialog();
    await user.click(await screen.findByRole('button', { name: 'Créer le projet' }));
    expect(screen.getByRole('textbox', { name: 'Nom' })).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByText('Donnez un nom au projet.')).toBeInTheDocument();
    expect(onCreate).not.toHaveBeenCalled();
  });

  it('creates the project with the chosen name, folder and format', async () => {
    const user = userEvent.setup();
    const onCreate = renderDialog();
    await user.type(await screen.findByRole('textbox', { name: 'Nom' }), 'Short');
    await user.click(screen.getByRole('button', { name: 'Choisir…' }));
    await user.click(screen.getByRole('radio', { name: '9:16' }));
    await user.click(screen.getByRole('button', { name: 'Créer le projet' }));
    expect(onCreate).toHaveBeenCalledWith({
      name: 'Short',
      parentDir: '/elsewhere',
      format: { ...DEFAULTS.format, width: 1080, height: 1920 },
    } satisfies CreateProjectRequest);
  });

  it('asks for a location when the folder is cleared', async () => {
    const user = userEvent.setup();
    const onCreate = renderDialog();
    await user.type(await screen.findByRole('textbox', { name: 'Nom' }), 'Film');
    await user.clear(screen.getByRole('textbox', { name: 'Emplacement' }));
    await user.click(screen.getByRole('button', { name: 'Créer le projet' }));
    expect(screen.getByText('Choisissez un emplacement.')).toBeInTheDocument();
    expect(onCreate).not.toHaveBeenCalled();
  });
});
