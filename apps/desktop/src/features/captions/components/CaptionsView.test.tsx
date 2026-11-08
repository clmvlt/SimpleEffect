/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Command } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { SimpleEffectBridge } from '../../../../electron/shared/bridge-api';
import { createI18n } from '../../../i18n';
import { captionPreviewStore, clearCaptionPreview } from '../../../shared/caption-preview';
import { CoreClientProvider } from '../../../shared/core';
import { DesktopBridgeProvider } from '../../../shared/desktop';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { playheadStore, playheadTimeOf, selectionStore, selectItems } from '../../timeline';
import { captionsStore } from '../store/captions-store';
import { captionSession, seconds } from '../testing/caption-fixture';
import { CaptionsView } from './CaptionsView';

const clients: CoreClient[] = [];

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearCaptionPreview();
  selectItems([]);
  captionsStore.setState({ scope: 'track', preset: 'karaoke' });
  window.localStorage.clear();
});

/** The core's answer to `command` in these tests. */
function answerTo(command: Command): unknown {
  switch (command.kind) {
    case 'generateCaptions':
    case 'importSubtitles':
      return {
        kind: command.kind,
        itemIds: ['new-1', 'new-2'],
        trackId: 'v2',
        styleId: 'style-9',
        replaced: 2,
        untranscribedAssets: [],
      };
    case 'exportSubtitles':
      return { kind: command.kind, cueCount: 3 };
    case 'splitCaption':
    case 'mergeCaptions':
      return { kind: command.kind, itemId: 'caption-1' };
    case 'updateCaptionStyle':
      return { kind: command.kind, styleIds: ['style-1'] };
    case 'applyCaptionPreset':
      return { kind: command.kind, styleId: 'style-9' };
    default:
      return { kind: command.kind };
  }
}

function renderPanel(bridge: Partial<SimpleEffectBridge> = {}) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command: Command) => Promise.resolve(answerTo(command) as never));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(captionSession());
  const fullBridge = {
    getCoreConnection: () => new Promise(() => undefined),
    chooseFiles: () => Promise.resolve([]),
    chooseSaveFile: () => Promise.resolve(null),
    ...bridge,
  } as unknown as SimpleEffectBridge;
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <DesktopBridgeProvider bridge={fullBridge}>
          <ProjectStoreProvider store={store} sync={sync}>
            <TooltipProvider>
              <CaptionsView />
            </TooltipProvider>
          </ProjectStoreProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { send };
}

const row = (id: string) => {
  const element = document.querySelector(`[data-caption-id="${id}"]`);
  if (!(element instanceof HTMLElement)) throw new Error(`no row ${id}`);
  return element;
};

describe('the captions tab', () => {
  it('lists the captions and follows the playhead; a click selects and seeks', async () => {
    renderPanel();
    expect(within(row('caption-1')).getByText('Bonjour')).toBeInTheDocument();
    expect(row('caption-3')).toHaveTextContent('un titre');
    act(() => {
      playheadStore.setState({ times: { 'sequence-1': seconds(1.5) } });
    });
    await waitFor(() => {
      expect(row('caption-2')).toHaveAttribute('data-current');
    });
    await userEvent.click(row('caption-3'));
    expect(selectionStore.getState().itemIds).toEqual(['caption-3']);
    expect(playheadTimeOf(playheadStore.getState(), 'sequence-1')).toBe(seconds(5));
  });

  it('edits a caption inline: commit, split at the caret, merge from the start', async () => {
    const { send } = renderPanel();
    fireEvent.doubleClick(row('caption-1'));
    const field = screen.getByRole('textbox', { name: 'Texte du sous-titre' });
    await userEvent.clear(field);
    await userEvent.type(field, 'Bonjour a tous{Control>}{Enter}{/Control}');
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'editCaptionText',
        itemId: 'caption-1',
        text: 'Bonjour a tous',
      }),
    );
    fireEvent.doubleClick(row('caption-1'));
    const again = screen.getByRole('textbox', { name: 'Texte du sous-titre' });
    if (!(again instanceof HTMLTextAreaElement)) throw new Error('a textarea');
    again.setSelectionRange(8, 8);
    fireEvent.keyDown(again, { key: 'Enter' });
    await waitFor(() => {
      expect(send).toHaveBeenCalledWith(
        expect.objectContaining({ kind: 'splitCaption', itemId: 'caption-1', wordIndex: 1 }),
      );
    });
    fireEvent.doubleClick(row('caption-2'));
    const second = screen.getByRole('textbox', { name: 'Texte du sous-titre' });
    if (!(second instanceof HTMLTextAreaElement)) throw new Error('a textarea');
    second.setSelectionRange(0, 0);
    fireEvent.keyDown(second, { key: 'Backspace' });
    await waitFor(() => {
      expect(send).toHaveBeenCalledWith(
        expect.objectContaining({ kind: 'mergeCaptions', itemIds: ['caption-1', 'caption-2'] }),
      );
    });
  });

  it('puts a word forward from its menu', async () => {
    const { send } = renderPanel();
    fireEvent.contextMenu(within(row('caption-1')).getByText('tous'));
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Mettre ce mot en avant' }));
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'setCaptionEmphasis',
        itemId: 'caption-1',
        wordIndices: [2],
        emphasis: true,
      }),
    );
  });

  it('generates with the chosen style and exports to the file chosen', async () => {
    const chooseSaveFile = vi.fn(() => Promise.resolve('/out/Film.srt'));
    const { send } = renderPanel({ chooseSaveFile });
    await userEvent.click(screen.getByRole('button', { name: /Régénérer/ }));
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'generateCaptions',
        sequenceId: 'sequence-1',
        trackName: 'Sous-titres',
        style: { kind: 'preset', preset: 'karaoke' },
      }),
    );
    await userEvent.click(screen.getByRole('button', { name: 'Exporter en SRT…' }));
    await waitFor(() => {
      expect(send).toHaveBeenCalledWith(
        expect.objectContaining({ kind: 'exportSubtitles', path: '/out/Film.srt' }),
      );
    });
    expect(chooseSaveFile).toHaveBeenCalledWith(
      expect.objectContaining({ defaultName: 'Film.srt', extensions: ['srt'] }),
    );
  });

  it('previews a preset on hover and applies it to the track, fields to the selection', async () => {
    const { send } = renderPanel();
    await userEvent.click(screen.getByRole('radio', { name: 'Style' }));
    const boxPreset = document.querySelector('[data-caption-preset="box"]');
    if (!(boxPreset instanceof HTMLElement)) throw new Error('the box preset');
    fireEvent.pointerEnter(boxPreset);
    expect(captionPreviewStore.getState().preview?.itemIds).toEqual(['caption-1', 'caption-2']);
    expect(captionPreviewStore.getState().preview?.style.animation).toBe('boxPerLine');
    fireEvent.pointerLeave(boxPreset);
    expect(captionPreviewStore.getState().preview).toBeNull();
    await userEvent.click(boxPreset);
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'applyCaptionPreset',
        itemIds: ['caption-1', 'caption-2'],
        preset: 'box',
      }),
    );
    await userEvent.click(screen.getByRole('radio', { name: 'Sélection' }));
    act(() => {
      selectItems(['caption-2']);
    });
    await userEvent.click(screen.getByRole('switch', { name: 'Majuscules' }));
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'updateCaptionStyle',
        itemIds: ['caption-2'],
        patch: { textCase: 'uppercase' },
      }),
    );
  });
});
