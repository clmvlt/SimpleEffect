import type { ExportPreset, Flicks, SequenceEntity } from '@se/protocol';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useFontFamilies } from '../../../shared/fonts';
import { useProjectSelector } from '../../../shared/project-store';
import { useInOutSpan, type TimeSpan } from '../../timeline';
import { normalizedDraft, type ExportDraft } from '../services/export-draft';
import {
  contentEnd,
  exportPlan,
  sequenceItems,
  type ExportPlan,
  type ExportSpan,
} from '../services/export-plan';
import {
  builtInPreset,
  DEFAULT_PRESET_CHOICE,
  draftOfSaved,
  savedPresetOf,
  type PresetChoice,
} from '../services/export-presets';
import type { EncoderSupport } from '../services/export-warnings';
import { isValidTemplate } from '../services/file-name';
import { useEncoderSupport, type EncoderProbe } from './use-encoder-support';
import { useExportPreferences } from './use-export-preferences';

/** The part of the sequence exported. */
export type RangeKind = 'sequence' | 'inOut';

/** What the export dialog shows and how it changes (see {@link useExportForm}). */
export interface ExportForm {
  /** Whether the preferences are still being read. */
  readonly loading: boolean;
  readonly choice: PresetChoice;
  readonly savedPresets: readonly ExportPreset[];
  readonly draft: ExportDraft;
  readonly presetName: string;
  readonly rangeKind: RangeKind;
  /** The in/out span of the sequence, `null` without in point. */
  readonly inOut: TimeSpan | null;
  /** Where the sequence's content ends (the whole sequence's span ends there). */
  readonly contentEnd: Flicks;
  readonly span: ExportSpan;
  readonly directory: string;
  readonly template: string;
  readonly templateValid: boolean;
  readonly support: EncoderSupport | null;
  readonly plan: ExportPlan;
  readonly choose: (choice: PresetChoice) => void;
  readonly editCustom: (draft: ExportDraft) => void;
  readonly setRangeKind: (kind: RangeKind) => void;
  readonly setDirectory: (folder: string) => void;
  readonly setTemplate: (template: string) => void;
  readonly savePreset: (name: string) => Promise<void>;
  readonly removePreset: (presetId: string) => Promise<void>;
  /** Remembers the folder and template (after starting an export). */
  readonly rememberDestination: () => Promise<void>;
}

/** Injected parts of the form (tests use fakes). */
export interface ExportFormDeps {
  readonly probe: EncoderProbe;
  readonly now: () => Date;
  readonly newId: () => string;
}

/**
 * The state of the export dialog for `sequence`: preset or custom settings, range, destination,
 * and the plan they give (settings, file name, size estimate, warnings).
 */
export function useExportForm(sequence: SequenceEntity, deps: ExportFormDeps): ExportForm {
  const { t } = useTranslation();
  const mirror = useProjectSelector((state) => state.mirror);
  const families = useFontFamilies();
  const { preferences, save } = useExportPreferences();
  const items = mirror === null ? [] : sequenceItems(sequence, mirror.tracks, mirror.items);
  const end = contentEnd(items);
  const inOut = useInOutSpan(sequence.id, end);
  const [choice, setChoice] = useState<PresetChoice>(DEFAULT_PRESET_CHOICE);
  const [customDraft, setCustomDraft] = useState<ExportDraft>(builtInPreset('youtube1080').draft);
  const [rangeKind, setRangeKind] = useState<RangeKind>(inOut === null ? 'sequence' : 'inOut');
  const [directory, setDirectory] = useState<string | null>(null);
  const [template, setTemplate] = useState<string | null>(null);
  const savedPresets = preferences?.presets ?? [];
  const saved =
    choice.kind === 'saved' ? savedPresets.find((preset) => preset.id === choice.id) : undefined;
  const draft =
    choice.kind === 'builtIn'
      ? builtInPreset(choice.id).draft
      : saved === undefined
        ? customDraft
        : draftOfSaved(saved);
  const presetName =
    choice.kind === 'builtIn'
      ? t(`export.presets.${choice.id}`)
      : (saved?.name ?? t('export.presets.custom'));
  const span =
    rangeKind === 'inOut' && inOut !== null ? inOut : { start: 0 as ExportSpan['start'], end };
  const shownTemplate = template ?? preferences?.fileNameTemplate ?? '{sequence}';
  const format = sequence.format;
  const settingsProbe = exportPlan(planInput(null));
  const support = useEncoderSupport(
    settingsProbe.settings,
    format.frameRate.numerator / format.frameRate.denominator,
    deps.probe,
  );
  const plan = exportPlan(planInput(support));

  function planInput(found: EncoderSupport | null) {
    return {
      draft,
      format,
      span,
      template: shownTemplate,
      names: { project: mirror?.project.name ?? '', sequence: sequence.name, preset: presetName },
      now: deps.now(),
      support: found,
      aspect: choice.kind === 'builtIn' ? builtInPreset(choice.id).aspect : null,
      items,
      assets: mirror?.assets ?? {},
      captionStyles: mirror?.project.captionStyles ?? [],
      families: families.length === 0 ? null : families,
    };
  }

  const remember = async (changes: Partial<NonNullable<typeof preferences>>) => {
    if (preferences === null) return;
    await save({ ...preferences, ...changes });
  };
  return {
    loading: preferences === null,
    choice,
    savedPresets,
    draft,
    presetName,
    rangeKind,
    inOut,
    contentEnd: end,
    span,
    directory: directory ?? preferences?.directory ?? '',
    template: shownTemplate,
    templateValid: isValidTemplate(shownTemplate),
    support,
    plan,
    choose: (next) => {
      if (next.kind === 'custom') setCustomDraft(draft);
      setChoice(next);
    },
    editCustom: (next) => {
      setCustomDraft(normalizedDraft(next));
      setChoice({ kind: 'custom' });
    },
    setRangeKind,
    setDirectory,
    setTemplate,
    savePreset: async (name) => {
      const preset = savedPresetOf(deps.newId(), name, draft, format);
      await remember({ presets: [...savedPresets, preset] });
      setChoice({ kind: 'saved', id: preset.id });
    },
    removePreset: async (presetId) => {
      await remember({ presets: savedPresets.filter((preset) => preset.id !== presetId) });
      setCustomDraft(draft);
      setChoice({ kind: 'custom' });
    },
    rememberDestination: () =>
      remember({
        directory: directory ?? preferences?.directory ?? '',
        fileNameTemplate: shownTemplate,
      }),
  };
}
