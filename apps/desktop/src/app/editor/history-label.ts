import type { TFunction } from 'i18next';

/**
 * Translation keys of the labels se-core gives the user transactions it builds itself
 * (`RENAME_LABEL`, `CREATE_SEQUENCE_LABEL` in `crates/se-app/src/project_service/`, `labels` in
 * `crates/se-app/src/media_service.rs`). Keep both sides in step. Other labels (from AI agents,
 * later from the UI) are shown as they are.
 */
const CORE_LABEL_KEYS = {
  'Create sequence': 'editor.history.createSequence',
  'Rename project': 'editor.history.renameProject',
  'Import media': 'editor.history.importMedia',
  'Relink media': 'editor.history.relinkMedia',
  'Remove media': 'editor.history.removeMedia',
} as const;

/** The undo/redo label to show for a transaction label received from the core. */
export function historyLabel(t: TFunction, label: string): string {
  return label in CORE_LABEL_KEYS
    ? t(CORE_LABEL_KEYS[label as keyof typeof CORE_LABEL_KEYS])
    : label;
}
