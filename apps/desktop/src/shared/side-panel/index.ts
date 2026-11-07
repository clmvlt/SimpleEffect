/** Which tab the editor's right panel shows (inspector or AI); features can ask for one. */
export {
  isSidePanelTab,
  showSidePanelTab,
  sidePanelStore,
  useSidePanelTab,
} from './side-panel-store';
export type { SidePanelState, SidePanelTab } from './side-panel-store';
