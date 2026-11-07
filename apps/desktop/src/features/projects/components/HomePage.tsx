import type { RecentProject } from '@se/protocol';
import { Button, EmptyState, Icon, Input, Spinner, toast } from '@se/ui';
import { Clapperboard, FolderOpen, Plus, SearchX, TriangleAlert } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { SettingsButton } from '../../settings';
import { useDesktopBridge } from '../../../shared/desktop';
import { errorMessageKey } from '../../../shared/errors';
import { getNewProjectDefaults } from '../api';
import { useProjectActions } from '../hooks/use-project-actions';
import { useRecentProjects } from '../hooks/use-recent-projects';
import { NewProjectDialog } from './NewProjectDialog';
import { ProjectCard } from './ProjectCard';
import { RenameProjectDialog } from './RenameProjectDialog';
import { TrashProjectDialog } from './TrashProjectDialog';

/**
 * The home screen: recent projects (search, open, rename, duplicate, trash, reveal), « Nouveau
 * projet » and « Ouvrir un projet ».
 */
export function HomePage() {
  const { t } = useTranslation();
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const recent = useRecentProjects();
  const actions = useProjectActions();
  const [creating, setCreating] = useState(false);
  const [renaming, setRenaming] = useState<RecentProject | null>(null);
  const [trashing, setTrashing] = useState<RecentProject | null>(null);
  const newProject = () => {
    setCreating(true);
  };

  return (
    <main className="flex size-full flex-col bg-bg">
      <h1 className="sr-only">{t('home.title')}</h1>
      <header className="flex flex-wrap items-center gap-2 border-b border-border px-6 py-3">
        <h2 className="text-lg font-semibold text-text">{t('home.heading')}</h2>
        <Input
          type="search"
          className="ml-auto w-64"
          aria-label={t('home.search')}
          placeholder={t('home.search')}
          value={recent.query}
          onChange={(event) => {
            recent.setQuery(event.target.value);
          }}
        />
        <Button
          variant="secondary"
          size="md"
          onClick={() => {
            void actions.openWithPicker();
          }}
        >
          <Icon icon={FolderOpen} size="md" />
          {t('home.openProject')}
        </Button>
        <Button variant="primary" size="md" onClick={newProject}>
          <Icon icon={Plus} size="md" />
          {t('home.newProject')}
        </Button>
        <SettingsButton projectId={null} />
      </header>
      <section className="min-h-0 flex-1 overflow-auto p-6">
        <RecentProjectsContent
          recent={recent}
          onNewProject={newProject}
          onOpen={(project) => void actions.openPath(project.path)}
          onRename={setRenaming}
          onDuplicate={(project) => void actions.duplicate(project)}
          onShowInFolder={actions.showInFolder}
          onTrash={setTrashing}
        />
      </section>
      <NewProjectDialog
        open={creating}
        onOpenChange={setCreating}
        loadDefaults={() => getNewProjectDefaults(client)}
        chooseFolder={(current) =>
          bridge.chooseFolder({
            title: t('projects.locationDialog.title'),
            buttonLabel: t('projects.locationDialog.button'),
            defaultPath: current,
          })
        }
        onCreate={actions.create}
        onError={(error) => {
          setCreating(false);
          toast.error(t(errorMessageKey(error)));
        }}
      />
      <RenameProjectDialog
        project={renaming}
        onClose={() => {
          setRenaming(null);
        }}
        onRename={actions.rename}
      />
      <TrashProjectDialog
        project={trashing}
        onClose={() => {
          setTrashing(null);
        }}
        onConfirm={(project) => void actions.trash(project)}
      />
    </main>
  );
}

interface RecentProjectsContentProps {
  readonly recent: ReturnType<typeof useRecentProjects>;
  readonly onNewProject: () => void;
  readonly onOpen: (project: RecentProject) => void;
  readonly onRename: (project: RecentProject) => void;
  readonly onDuplicate: (project: RecentProject) => void;
  readonly onShowInFolder: (project: RecentProject) => void;
  readonly onTrash: (project: RecentProject) => void;
}

/** Loading, failure, empty and no-match states, or the grid of recent projects. */
function RecentProjectsContent({ recent, onNewProject, ...handlers }: RecentProjectsContentProps) {
  const { t } = useTranslation();
  if (recent.status === 'loading' && recent.projects.length === 0) {
    return (
      <div className="flex size-full items-center justify-center">
        <Spinner size="lg" label={t('home.loading')} />
      </div>
    );
  }
  if (recent.status === 'failed' && recent.projects.length === 0) {
    return (
      <EmptyState
        size="md"
        icon={TriangleAlert}
        title={t('home.loadFailed')}
        actions={
          <Button variant="secondary" onClick={() => void recent.refresh()}>
            {t('home.retry')}
          </Button>
        }
      />
    );
  }
  if (recent.projects.length === 0) {
    return (
      <EmptyState
        size="md"
        icon={Clapperboard}
        title={t('home.empty.title')}
        description={t('home.empty.description')}
        actions={
          <Button variant="primary" size="md" onClick={onNewProject}>
            <Icon icon={Plus} size="md" />
            {t('home.newProject')}
          </Button>
        }
      />
    );
  }
  if (recent.visible.length === 0) {
    return (
      <EmptyState
        size="md"
        icon={SearchX}
        title={t('home.noMatch.title')}
        description={t('home.noMatch.description', { query: recent.query })}
      />
    );
  }
  return (
    <ul className="grid grid-cols-[repeat(auto-fill,minmax(18rem,1fr))] gap-3">
      {recent.visible.map((project) => (
        <ProjectCard key={project.path} project={project} {...handlers} />
      ))}
    </ul>
  );
}
