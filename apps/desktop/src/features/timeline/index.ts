/**
 * Timeline feature: display and navigation of the active sequence (prompt 10), editing from
 * prompt 11. Other features read the selection, the in/out span (the scope of AI requests,
 * prompt 20), the playhead and the active sequence through these exports.
 */
export { TimelinePanel } from './components/TimelinePanel';
export { useActiveSequence } from './hooks/use-active-sequence';
export { useInOutSpan } from './hooks/use-in-out';
export type { InOutPoints, TimeSpan } from './services/in-out';
export {
  playheadStore,
  playheadTimeOf,
  setPlayheadTime,
  usePlayheadTime,
} from './store/playhead-store';
export {
  clearSelection,
  selectionStore,
  selectItems,
  useSelectedItemIds,
} from './store/selection-store';
