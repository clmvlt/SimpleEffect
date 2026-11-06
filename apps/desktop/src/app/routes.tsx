import type { DockLayoutStorage } from '@se/ui';
import type { RouteObject } from 'react-router';

import { HomePage } from '../features/projects';
import { EditorPage } from './editor/EditorPage';
import { AppLayout } from './layout/AppLayout';

/** What the routes need from the composition root (injected so tests can pass fakes). */
export interface AppRouteDependencies {
  /** Where the editor saves its panel sizes. */
  readonly layoutStorage: DockLayoutStorage;
}

/**
 * The app's routes: `/` home, `/project/:projectId` editor. The renderer is loaded from a file in
 * production, so `main.tsx` uses a hash router; tests use a memory router with the same routes.
 */
export function createAppRoutes({ layoutStorage }: AppRouteDependencies): RouteObject[] {
  return [
    {
      element: <AppLayout />,
      children: [
        { index: true, element: <HomePage /> },
        { path: 'project/:projectId', element: <EditorPage layoutStorage={layoutStorage} /> },
      ],
    },
  ];
}
