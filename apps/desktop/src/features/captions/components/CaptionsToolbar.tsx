import type { CaptionStyleChoice, Flicks, SequenceEntity, TimeRange } from '@se/protocol';
import { CAPTION_STYLE_PRESETS, type CaptionPresetId } from '@se/engine/caption-styles';
import {
  Button,
  IconButton,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tooltip,
} from '@se/ui';
import { FileDown, FileUp, Sparkles } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useInOutSpan } from '../../timeline';
import type { CaptionCommands } from '../hooks/use-caption-commands';
import type { CaptionRow } from '../services/caption-rows';
import { useProjectSelector } from '../../../shared/project-store';
import {
  FALLBACK_PRESET,
  setGenerationPreset,
  useCaptionsPreferences,
} from '../store/captions-store';
import { ShiftPopover } from './ShiftPopover';

/** Props of {@link CaptionsToolbar}. */
export interface CaptionsToolbarProps {
  readonly sequence: SequenceEntity;
  readonly rows: readonly CaptionRow[];
  /** Where the sequence's content ends (an in point alone spans to it). */
  readonly contentEnd: Flicks;
  readonly commands: CaptionCommands;
}

/**
 * The tab's actions: the style of new captions, « Générer » (the in/out span when one is marked,
 * else the whole sequence; captions it crosses are replaced), SRT import and export, the global
 * shift of the captions.
 */
export function CaptionsToolbar({ sequence, rows, contentEnd, commands }: CaptionsToolbarProps) {
  const { t } = useTranslation();
  const { preset } = useCaptionsPreferences();
  const hasKit = useProjectSelector((state) => (state.mirror?.project.brandKit ?? null) !== null);
  const span = useInOutSpan(sequence.id, contentEnd);
  const range: TimeRange | null =
    span === null ? null : { start: span.start, duration: (span.end - span.start) as Flicks };
  const chosen = preset === 'brandKit' && !hasKit ? FALLBACK_PRESET : preset;
  const style: CaptionStyleChoice =
    chosen === 'brandKit' ? { kind: 'brandKit' } : { kind: 'preset', preset: chosen };
  return (
    <div className="flex flex-wrap items-center gap-1.5 border-b border-border p-2">
      <Select
        value={chosen}
        onValueChange={(value) => {
          if (value === 'brandKit') {
            setGenerationPreset('brandKit');
            return;
          }
          const picked = CAPTION_STYLE_PRESETS.find((candidate) => candidate.id === value);
          if (picked !== undefined) setGenerationPreset(picked.id);
        }}
      >
        <SelectTrigger size="xs" className="w-32" aria-label={t('editor.captions.generationStyle')}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {hasKit && <SelectItem value="brandKit">{t('brandKit.fontsGroup')}</SelectItem>}
          {CAPTION_STYLE_PRESETS.map((candidate) => (
            <SelectItem key={candidate.id} value={candidate.id}>
              {t(presetKey(candidate.id))}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Tooltip
        content={t(range === null ? 'editor.captions.generateAll' : 'editor.captions.generateSpan')}
      >
        <Button
          size="xs"
          variant="primary"
          disabled={commands.busy}
          onClick={() => {
            void commands.generate(range, style);
          }}
        >
          <Sparkles aria-hidden className="size-3.5" />
          {t(rows.length === 0 ? 'editor.captions.generate' : 'editor.captions.regenerate')}
        </Button>
      </Tooltip>
      <div className="ml-auto flex items-center gap-0.5">
        <Tooltip content={t('editor.captions.import.title')}>
          <IconButton
            icon={FileUp}
            size="xs"
            label={t('editor.captions.import.title')}
            disabled={commands.busy}
            onClick={() => {
              void commands.importFile(style);
            }}
          />
        </Tooltip>
        <Tooltip content={t('editor.captions.export.title')}>
          <IconButton
            icon={FileDown}
            size="xs"
            label={t('editor.captions.export.title')}
            disabled={commands.busy || rows.length === 0}
            onClick={() => {
              void commands.exportFile();
            }}
          />
        </Tooltip>
        <ShiftPopover rows={rows} commands={commands} />
      </div>
    </div>
  );
}

/** The translation key of a preset's name. */
export function presetKey(id: CaptionPresetId): `editor.captions.presets.${CaptionPresetId}` {
  return `editor.captions.presets.${id}`;
}
