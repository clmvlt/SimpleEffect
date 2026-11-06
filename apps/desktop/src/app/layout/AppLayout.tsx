import { Outlet } from 'react-router';

import { StatusBar } from '../status-bar/StatusBar';

/** The window frame shared by every route: the page above, the status bar below. */
export function AppLayout() {
  return (
    <div className="flex h-screen flex-col bg-bg text-text">
      <div className="min-h-0 flex-1">
        <Outlet />
      </div>
      <StatusBar />
    </div>
  );
}
