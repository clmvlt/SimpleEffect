/**
 * Timeline feature: display and navigation of the active sequence (prompt 10), editing from
 * prompt 11. Other features read the selection, the in/out span (the scope of AI requests,
 * prompt 20), the playhead and the active sequence through these exports; the AI feature reveals
 * a task's scope with `revealTimeRange`. The timeline draws AI scopes, pulses and origin badges
 * from `shared/ai-tasks`.
 */
export { TimelinePanel } from './components/TimelinePanel';
export { useActiveSequence } from './hooks/use-active-sequence';
export { inOutControls, useInOutPoints, useInOutSpan } from './hooks/use-in-out';
export { revealTimeRange } from './store/reveal-range';
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
export { formatVolume, shiftVolume } from './services/volume-line';
