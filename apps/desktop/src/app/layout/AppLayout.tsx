import { Outlet } from 'react-router';

import { AboutDialog } from '../../features/about';

import { useAiStatusSync } from '../../features/ai';
import { useExportHost } from '../../features/export';
import { OnboardingDialog } from '../../features/onboarding';
import { useTemplateCheckRequests } from '../../features/templates';
import { useAgentFrameRequests } from '../../features/viewer';
import type { KeyValueStorage } from '../../shared/storage';
import { ShortcutsDialogHost } from '../shortcuts/ShortcutsDialogHost';
import { StatusBar } from '../status-bar/StatusBar';

/**
 * The window frame shared by every route: the page above, the status bar below, and the keyboard
 * shortcuts help (`?`). It keeps the app-wide AI status in step with the core and answers AI
 * agents' frame requests and motion template checks (whatever page is shown, so refusals come
 * at once), and hosts the exports (their list, the renders the core asks for, the anti-sleep).
 * It also hosts « À propos » and the first-run wizard (prompt 30), which remembers in
 * `preferences` that it was finished.
 */
export function AppLayout({ preferences }: { readonly preferences: KeyValueStorage }) {
  useAiStatusSync();
  useAgentFrameRequests();
  useTemplateCheckRequests();
  useExportHost();
  return (
    <div className="flex h-screen flex-col bg-bg text-text">
      <div className="min-h-0 flex-1">
        <Outlet />
      </div>
      <StatusBar />
      <ShortcutsDialogHost />
      <AboutDialog />
      <OnboardingDialog preferences={preferences} />
    </div>
  );
}
