/** @vitest-environment jsdom */
import { TooltipProvider } from '@se/ui';
import { act, render, screen } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import { describe, expect, it } from 'vitest';

import { createI18n } from '../../i18n';
import {
  createProjectStore,
  ProjectStoreProvider,
  type ProjectSync,
} from '../../shared/project-store';
import { SaveStateIndicator } from './SaveStateIndicator';

/** The indicator only reads the store: its sync is never called. */
const idleSync: ProjectSync = {
  load: () => undefined,
  forget: () => undefined,
  resync: () => Promise.resolve(),
  dispose: () => undefined,
};

function renderIndicator() {
  const store = createProjectStore();
  render(
    <I18nextProvider i18n={createI18n()}>
      <ProjectStoreProvider store={store} sync={idleSync}>
        <TooltipProvider>
          <SaveStateIndicator />
        </TooltipProvider>
      </ProjectStoreProvider>
    </I18nextProvider>,
  );
  return store;
}

describe('SaveStateIndicator', () => {
  it('shows nothing while saving works and alerts when the core cannot save', () => {
    const store = renderIndicator();
    expect(screen.queryByRole('alert')).toBeNull();
    act(() => {
      store.setState({
        saveState: { projectId: 'p', failure: 'The disk is full.', unsavedChanges: 3 },
      });
    });
    expect(screen.getByRole('alert')).toHaveTextContent(
      'Sauvegarde impossible · 3 modifications non enregistrées',
    );
    act(() => {
      store.setState({ saveState: { projectId: 'p', failure: null, unsavedChanges: 0 } });
    });
    expect(screen.queryByRole('alert')).toBeNull();
  });
});
