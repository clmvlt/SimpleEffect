/**
 * Sequences feature (prompt 16, ADR-0026): renaming, duplicating, removing sequences, changing
 * their format and creating versions in other formats, with the warning about items a change
 * pushes out of the frame. The timeline opens these from its sequence tabs.
 */
export { SequenceDialogs, type SequenceDialogsProps } from './components/SequenceDialogs';
export {
  useSequenceActions,
  type SequenceActionHandlers,
  type SequenceActions,
} from './hooks/use-sequence-actions';
export {
  closeSequenceDialog,
  openSequenceDialog,
  type SequenceDialog,
} from './store/sequence-dialog-store';
