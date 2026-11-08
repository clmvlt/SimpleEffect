import { Outlet } from 'react-router';

import { useAiStatusSync } from '../../features/ai';
import { useAgentFrameRequests } from '../../features/viewer';
import { ShortcutsDialogHost } from '../shortcuts/ShortcutsDialogHost';
import { StatusBar } from '../status-bar/StatusBar';

/**
 * The window frame shared by every route: the page above, the status bar below, and the keyboard
 * shortcuts help (`?`). It keeps the app-wide AI status in step with the core and answers AI
 * agents' frame requests (whatever page is shown, so refusals come at once).
 */
export function AppLayout() {
  useAiStatusSync();
  useAgentFrameRequests();
  return (
    <div className="flex h-screen flex-col bg-bg text-text">
      <div className="min-h-0 flex-1">
        <Outlet />
      </div>
      <StatusBar />
      <ShortcutsDialogHost />
    </div>
  );
}
