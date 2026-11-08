import { EmptyState, ToggleGroup, ToggleGroupItem } from '@se/ui';
import { Captions } from '@se/ui/icons';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { clearCaptionPreview } from '../../../shared/caption-preview';
import { uiPreferencesStorage } from '../../../shared/storage';
import { useActiveSequence, useSelectedItemIds } from '../../timeline';
import { useCaptionCommands } from '../hooks/use-caption-commands';
import { useActiveRow, useCaptionRows } from '../hooks/use-caption-rows';
import { connectCaptionsPersistence } from '../store/captions-store';
import { CaptionList } from './CaptionList';
import { CaptionStylePanel } from './CaptionStylePanel';
import { CaptionsToolbar } from './CaptionsToolbar';

/** The tab's views: the list of captions, their style. */
type CaptionsTool = 'list' | 'style';

/**
 * The « Sous-titres » tab (prompt 23): generation from the transcript, SRT import and export,
 * the list of the active sequence's captions following playback (text edited inline, split with
 * Entrée, merged with Retour arrière, words put forward from their menu, global shift), and the
 * style panel (predefined styles previewed on hover, fields, applied to the selection or the
 * whole track).
 */
export function CaptionsView() {
  const { t } = useTranslation();
  const [tool, setTool] = useState<CaptionsTool>('list');
  const sequence = useActiveSequence();
  const { rows, styles, end } = useCaptionRows(sequence);
  const active = useActiveRow(sequence?.id ?? null, rows);
  const selected = useSelectedItemIds();
  const commands = useCaptionCommands(sequence);
  useEffect(() => connectCaptionsPersistence(uiPreferencesStorage()), []);
  useEffect(() => clearCaptionPreview, []);
  if (sequence === null || commands === null) {
    return <EmptyState icon={Captions} title={t('editor.captions.noSequence')} />;
  }
  return (
    <div className="flex size-full min-h-0 flex-col">
      <CaptionsToolbar sequence={sequence} rows={rows} contentEnd={end} commands={commands} />
      <ToggleGroup
        type="single"
        variant="segmented"
        size="xs"
        value={tool}
        aria-label={t('editor.captions.tools.label')}
        className="m-2 mb-0 self-start"
        onValueChange={(value) => {
          if (value === 'list' || value === 'style') setTool(value);
        }}
      >
        <ToggleGroupItem value="list">{t('editor.captions.tools.list')}</ToggleGroupItem>
        <ToggleGroupItem value="style">{t('editor.captions.tools.style')}</ToggleGroupItem>
      </ToggleGroup>
      {rows.length === 0 ? (
        <EmptyState
          icon={Captions}
          title={t('editor.captions.empty.title')}
          description={t('editor.captions.empty.description')}
        />
      ) : tool === 'list' ? (
        <CaptionList
          sequence={sequence}
          rows={rows}
          active={active}
          selected={selected}
          commands={commands}
        />
      ) : (
        <CaptionStylePanel
          rows={rows}
          styles={styles}
          active={active}
          selected={selected}
          commands={commands}
        />
      )}
    </div>
  );
}
