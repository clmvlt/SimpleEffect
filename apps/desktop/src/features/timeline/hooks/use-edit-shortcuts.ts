import { useShortcutAction } from '../../../shared/shortcuts';
import type { EditActions } from './use-edit-actions';

/**
 * Binds the editing shortcuts of the timeline: cut at the playhead, delete (Maj: ripple), copy,
 * cut, paste (Maj: insert), nudge by one frame (ten with `<` `>`).
 */
export function useEditShortcuts(actions: EditActions): void {
  useShortcutAction('splitAtPlayhead', actions.splitAtPlayhead);
  useShortcutAction('deleteSelection', () => {
    actions.deleteSelection(false);
  });
  useShortcutAction('rippleDelete', () => {
    actions.deleteSelection(true);
  });
  useShortcutAction('copy', actions.copySelection);
  useShortcutAction('cut', actions.cutSelection);
  useShortcutAction('paste', () => {
    actions.paste('overwrite');
  });
  useShortcutAction('pasteInsert', () => {
    actions.paste('insert');
  });
  useShortcutAction('nudgeLeft', () => {
    actions.nudge(-1);
  });
  useShortcutAction('nudgeRight', () => {
    actions.nudge(1);
  });
  useShortcutAction('nudgeLeftTen', () => {
    actions.nudge(-10);
  });
  useShortcutAction('nudgeRightTen', () => {
    actions.nudge(10);
  });
}
