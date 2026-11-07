import type { CreateProjectRequest, RecentProject } from '@se/protocol';
import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import { errorMessageKey } from '../../../shared/errors';
import { projectPath } from '../../../shared/routing';
import { createProject, duplicateProject, openProject, renameProject, trashProject } from '../api';
import { refreshRecentProjects } from '../store/recent-projects-store';

/**
 * What the home screen does with projects: open (from the list or a folder picker), create,
 * rename, duplicate, trash and reveal. Failures show a translated toast; mutations refresh the
 * recent list. Mutating functions resolve with whether they succeeded.
 */
export function useProjectActions() {
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const navigate = useNavigate();
  const { t } = useTranslation();

  const fail = (error: unknown) => {
    console.warn('[projects]', error);
    toast.error(t(errorMessageKey(error)));
  };
  const refresh = () => refreshRecentProjects(client);
  const attempt = async (work: () => Promise<void>): Promise<boolean> => {
    try {
      await work();
      return true;
    } catch (error) {
      fail(error);
      return false;
    } finally {
      void refresh();
    }
  };

  const openPath = (path: string) =>
    attempt(async () => {
      const session = await openProject(client, path);
      void navigate(projectPath(session.snapshot.project.id));
    });

  return {
    openPath,
    openWithPicker: async (): Promise<void> => {
      const path = await bridge.chooseFolder({
        title: t('projects.openDialog.title'),
        buttonLabel: t('projects.openDialog.button'),
      });
      if (path !== null) await openPath(path);
    },
    create: (request: CreateProjectRequest) =>
      attempt(async () => {
        const session = await createProject(client, request);
        void navigate(projectPath(session.snapshot.project.id));
      }),
    rename: (project: RecentProject, name: string) =>
      attempt(async () => {
        await renameProject(client, project.path, name);
        toast.success(t('projects.toasts.renamed'));
      }),
    duplicate: (project: RecentProject) =>
      attempt(async () => {
        const name = t('projects.duplicateName', { name: project.name });
        const copy = await duplicateProject(client, project.path, name);
        toast.success(t('projects.toasts.duplicated', { name: copy.name }));
      }),
    trash: (project: RecentProject) =>
      attempt(async () => {
        await trashProject(client, project.path);
        toast.success(t('projects.toasts.trashed', { name: project.name }));
      }),
    showInFolder: (project: RecentProject): void => {
      void bridge.showInFolder(project.path).catch(fail);
    },
  };
}

/** The actions returned by {@link useProjectActions}. */
export type ProjectActions = ReturnType<typeof useProjectActions>;
