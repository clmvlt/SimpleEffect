import { Button, EmptyState, Icon, Spinner, type DockLayoutStorage } from '@se/ui';
import { CircleOff, House, TriangleAlert } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';
import { useParams } from 'react-router';

import { EditorHeader } from './EditorHeader';
import { EditorLayout } from './EditorLayout';
import { useEditorProject, type EditorProjectStatus } from './use-editor-project';
import { useLeaveProject } from './use-leave-project';

/** Props of {@link EditorPage}. */
export interface EditorPageProps {
  /** Where panel sizes are saved (the app's UI preferences). */
  readonly layoutStorage: DockLayoutStorage;
}

/** The editor of the project named by the `:projectId` route. */
export function EditorPage({ layoutStorage }: EditorPageProps) {
  const { projectId = '' } = useParams();
  const status = useEditorProject(projectId);
  const leave = useLeaveProject(projectId);
  return (
    <main className="flex size-full flex-col bg-bg">
      {status.kind === 'ready' ? (
        <>
          <EditorHeader projectId={projectId} onHome={leave} />
          <EditorLayout layoutStorage={layoutStorage} />
        </>
      ) : (
        <EditorStatus status={status} onHome={leave} />
      )}
    </main>
  );
}

/** What the editor shows while its project loads, or when it cannot be shown. */
function EditorStatus({
  status,
  onHome,
}: {
  readonly status: Exclude<EditorProjectStatus, { kind: 'ready' }>;
  readonly onHome: () => void;
}) {
  const { t } = useTranslation();
  if (status.kind === 'loading') {
    return (
      <div className="flex size-full items-center justify-center">
        <Spinner size="lg" label={t('editor.loading')} />
      </div>
    );
  }
  const home = (
    <Button variant="primary" size="md" onClick={onHome}>
      <Icon icon={House} size="md" />
      {t('editor.backHome')}
    </Button>
  );
  return status.kind === 'closed' ? (
    <EmptyState
      size="md"
      icon={CircleOff}
      title={t('editor.closed.title')}
      description={t('editor.closed.description')}
      actions={home}
    />
  ) : (
    <EmptyState
      size="md"
      icon={TriangleAlert}
      title={t('editor.loadFailed.title')}
      description={t(status.errorKey)}
      actions={home}
    />
  );
}
