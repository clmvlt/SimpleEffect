import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

/** Tabs of the editor's right panel. */
export type SidePanelTab = 'inspector' | 'animations' | 'ai' | 'transcript' | 'captions';

/** The tab shown in the editor's right panel (features ask for one, e.g. « Propriétés »). */
export interface SidePanelState {
  readonly tab: SidePanelTab;
}

/** The side panel store. */
export const sidePanelStore = createStore<SidePanelState>()(() => ({ tab: 'inspector' }));

/** Shows `tab` in the right panel. */
export function showSidePanelTab(tab: SidePanelTab): void {
  sidePanelStore.setState({ tab });
}

/** Whether `value` names a side panel tab. */
export function isSidePanelTab(value: string): value is SidePanelTab {
  return (
    value === 'inspector' ||
    value === 'animations' ||
    value === 'ai' ||
    value === 'transcript' ||
    value === 'captions'
  );
}

/** The tab shown in the right panel. */
export function useSidePanelTab(): SidePanelTab {
  return useStore(sidePanelStore, (state) => state.tab);
}
