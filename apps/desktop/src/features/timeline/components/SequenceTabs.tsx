import type { SequenceEntity } from '@se/protocol';
import { IconButton, Tabs, TabsList, TabsTrigger, Tooltip } from '@se/ui';
import { Clapperboard } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

/** Props of {@link SequenceTabs}. */
export interface SequenceTabsProps {
  readonly sequences: readonly SequenceEntity[];
  readonly activeId: string;
  readonly onSelect: (sequenceId: string) => void;
  readonly onCreate: () => void;
}

/**
 * The sequences of the project as tabs (shown when there are several) and a « + » to create one.
 * Choosing a tab shows that sequence in the timeline.
 */
export function SequenceTabs({ sequences, activeId, onSelect, onCreate }: SequenceTabsProps) {
  const { t } = useTranslation();
  return (
    <div className="flex min-w-0 items-center gap-1">
      {sequences.length > 1 && (
        <Tabs value={activeId} onValueChange={onSelect} className="min-w-0">
          <TabsList variant="underline" size="sm" aria-label={t('editor.timeline.sequences')}>
            {sequences.map((sequence) => (
              <TabsTrigger key={sequence.id} value={sequence.id} className="max-w-40 truncate">
                {sequence.name}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
      )}
      <Tooltip content={t('editor.timeline.newSequence')}>
        <IconButton
          size="xs"
          icon={Clapperboard}
          label={t('editor.timeline.newSequence')}
          onClick={onCreate}
        />
      </Tooltip>
    </div>
  );
}
