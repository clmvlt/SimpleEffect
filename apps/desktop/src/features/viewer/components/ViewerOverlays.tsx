import { Badge, EmptyState, Spinner } from '@se/ui';
import { MonitorX } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useDelayedFlag } from '../hooks/use-delayed-flag';
import { summarizeIssues } from '../services/issue-summary';
import type { ViewerEngineStatus } from '../services/viewer-engine-controller';

/** Decodes shorter than this show no spinner (most scrub steps). */
const LOADING_DELAY_MS = 150;

/** Props of {@link ViewerOverlays}. */
export interface ViewerOverlaysProps {
  readonly status: ViewerEngineStatus;
}

/**
 * What the viewer says over the picture: the decoding spinner, media problems of the frame, or
 * why the engine could not start.
 */
export function ViewerOverlays({ status }: ViewerOverlaysProps) {
  const { t } = useTranslation();
  const loading = useDelayedFlag(
    status.kind === 'running' && status.preview.status === 'loading',
    LOADING_DELAY_MS,
  );
  if (status.kind === 'failed') {
    return (
      <div className="absolute inset-0 bg-bg">
        <EmptyState
          icon={MonitorX}
          title={t('editor.viewer.failed.title')}
          description={
            status.reason === 'unsupported'
              ? t('editor.viewer.failed.unsupported')
              : t('editor.viewer.failed.error', { message: status.message })
          }
        />
      </div>
    );
  }
  const issues = status.kind === 'running' ? summarizeIssues(status.preview.issues) : null;
  const missingFonts = status.kind === 'running' ? status.preview.missingFonts : [];
  return (
    <>
      {loading && (
        <div className="pointer-events-none absolute top-2 right-2 rounded-sm bg-overlay p-1">
          <Spinner size="sm" label={t('editor.viewer.loading')} />
        </div>
      )}
      {issues !== null && (
        <Badge
          variant={issues.kind === 'preparing' ? 'neutral' : 'warning'}
          className="pointer-events-none absolute bottom-2 left-2"
        >
          {t(`editor.viewer.issues.${issues.kind}`, { count: issues.count })}
        </Badge>
      )}
      {missingFonts.length > 0 && (
        <Badge variant="warning" className="pointer-events-none absolute right-2 bottom-2">
          {t('editor.viewer.missingFonts', {
            count: missingFonts.length,
            families: missingFonts.join(', '),
          })}
        </Badge>
      )}
    </>
  );
}
