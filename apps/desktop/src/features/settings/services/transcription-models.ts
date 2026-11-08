import type { TranscriptionModelInfo, TranscriptionStatus } from '@se/protocol';

/** Value of « Automatique (recommandé) » in the model choice (`model: null`). */
export const AUTOMATIC_MODEL = 'automatic';

/** A choice of the model used: an installed model, or the current preference if it is gone. */
export interface ModelChoice {
  /** The model id (the radio value). */
  readonly value: string;
  /** What the list shows. */
  readonly label: string;
}

/** What a model is called in the list: the catalogue id, or an imported model's file name. */
export function modelLabel(model: TranscriptionModelInfo): string {
  return model.custom ? model.fileName : model.id;
}

/**
 * The models the user can choose besides « Automatique »: every installed one, plus the saved
 * choice when its file was removed (so the list still shows what is selected).
 */
export function modelChoices(status: TranscriptionStatus): readonly ModelChoice[] {
  const installed = status.models
    .filter((model) => model.installed)
    .map((model) => ({ value: model.id, label: modelLabel(model) }));
  const chosen = status.preferences.model;
  if (chosen === null || installed.some((choice) => choice.value === chosen)) return installed;
  return [...installed, { value: chosen, label: chosen }];
}

/** The radio value of the saved choice. */
export function modelChoiceValue(status: TranscriptionStatus): string {
  return status.preferences.model ?? AUTOMATIC_MODEL;
}

/** The preference for a radio value (`null` = automatic). */
export function modelFromChoice(value: string): string | null {
  return value === AUTOMATIC_MODEL ? null : value;
}

/** Whether at least one model is installed (transcription can run). */
export function hasInstalledModel(status: TranscriptionStatus): boolean {
  return status.models.some((model) => model.installed);
}

/** Whether a first model download also brings the voice model (it is missing and not coming). */
export function downloadsVoiceModelToo(status: TranscriptionStatus): boolean {
  return !status.voiceModel.installed && status.voiceModel.download === null;
}
