import type { DockLayoutStorage } from '@se/ui';
import type { RouteObject } from 'react-router';

import { HomePage } from '../features/projects';
import { PROJECT_ROUTE_PATTERN } from '../shared/routing';
import type { KeyValueStorage } from '../shared/storage';
import { EditorPage } from './editor/EditorPage';
import { AppLayout } from './layout/AppLayout';

/** What the routes need from the composition root (injected so tests can pass fakes). */
export interface AppRouteDependencies {
  /** Where the editor saves its panel sizes. */
  readonly layoutStorage: DockLayoutStorage;
  /** Machine preferences of the window frame (the first-run wizard's « done »). */
  readonly preferences: KeyValueStorage;
}

/**
 * The app's routes: `/` home, `/project/:projectId` editor. The renderer is loaded from a file in
 * production, so `main.tsx` uses a hash router; tests use a memory router with the same routes.
 */
export function createAppRoutes({
  layoutStorage,
  preferences,
}: AppRouteDependencies): RouteObject[] {
  return [
    {
      element: <AppLayout preferences={preferences} />,
      children: [
        { index: true, element: <HomePage /> },
        { path: PROJECT_ROUTE_PATTERN, element: <EditorPage layoutStorage={layoutStorage} /> },
      ],
    },
  ];
}
