import type { TFunction } from 'i18next';

/**
 * Translation keys of the labels se-core gives the user transactions it builds itself
 * (`RENAME_LABEL`, `CREATE_SEQUENCE_LABEL`, `tracks::labels`, `markers::labels`, `items::labels`,
 * `transitions::labels` in `crates/se-app/src/project_service/`, `labels` in
 * `crates/se-app/src/media_service.rs`). Keep
 * both sides in step. Other labels (from AI agents, later from the UI) are shown as they are.
 */
const CORE_LABEL_KEYS = {
  'Create sequence': 'editor.history.createSequence',
  'Rename project': 'editor.history.renameProject',
  'Import media': 'editor.history.importMedia',
  'Relink media': 'editor.history.relinkMedia',
  'Remove media': 'editor.history.removeMedia',
  'Add track': 'editor.history.addTrack',
  'Remove track': 'editor.history.removeTrack',
  'Reorder tracks': 'editor.history.reorderTracks',
  'Rename track': 'editor.history.renameTrack',
  'Mute track': 'editor.history.muteTrack',
  'Unmute track': 'editor.history.unmuteTrack',
  'Solo track': 'editor.history.soloTrack',
  'Unsolo track': 'editor.history.unsoloTrack',
  'Lock track': 'editor.history.lockTrack',
  'Unlock track': 'editor.history.unlockTrack',
  'Hide track': 'editor.history.hideTrack',
  'Show track': 'editor.history.showTrack',
  'Change track volume': 'editor.history.changeTrackVolume',
  'Change track pan': 'editor.history.changeTrackPan',
  'Update track': 'editor.history.updateTrack',
  'Add marker': 'editor.history.addMarker',
  'Remove marker': 'editor.history.removeMarker',
  'Move marker': 'editor.history.moveMarker',
  'Rename marker': 'editor.history.renameMarker',
  'Change marker color': 'editor.history.changeMarkerColor',
  'Update marker': 'editor.history.updateMarker',
  'Add clips': 'editor.history.addClips',
  'Move items': 'editor.history.moveItems',
  'Duplicate items': 'editor.history.duplicateItems',
  'Paste items': 'editor.history.pasteItems',
  'Delete items': 'editor.history.deleteItems',
  'Ripple delete': 'editor.history.rippleDelete',
  'Link items': 'editor.history.linkItems',
  'Unlink items': 'editor.history.unlinkItems',
  'Trim items': 'editor.history.trimItems',
  'Ripple trim': 'editor.history.rippleTrim',
  'Split items': 'editor.history.splitItems',
  'Close gap': 'editor.history.closeGap',
  'Resize gap': 'editor.history.resizeGap',
  'Change fades': 'editor.history.changeFades',
  'Edit items': 'editor.history.editItems',
  'Change volume': 'editor.history.changeVolume',
  'Change keyframes': 'editor.history.changeKeyframes',
  'Transform items': 'editor.history.transformItems',
  'Add text': 'editor.history.addText',
  'Add shape': 'editor.history.addShape',
  'Add image': 'editor.history.addImage',
  'Apply animation': 'editor.history.applyAnimation',
  'Add transitions': 'editor.history.addTransitions',
  'Change transition': 'editor.history.changeTransition',
  'Remove transitions': 'editor.history.removeTransitions',
} as const;

/** The undo/redo label to show for a transaction label received from the core. */
export function historyLabel(t: TFunction, label: string): string {
  return label in CORE_LABEL_KEYS
    ? t(CORE_LABEL_KEYS[label as keyof typeof CORE_LABEL_KEYS])
    : label;
}
