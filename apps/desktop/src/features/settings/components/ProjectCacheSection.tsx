import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
  Button,
  Skeleton,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { byteSize } from '../../../shared/text';
import type { ProjectCacheState } from '../hooks/use-project-cache';

/** Props of {@link ProjectCacheSection}. */
export interface ProjectCacheSectionProps {
  /** The open project, or `null` (home screen). */
  readonly projectId: string | null;
  readonly state: ProjectCacheState;
}

/** Where the open project's cache is, its size, « Afficher dans le dossier » and « Nettoyer ». */
export function ProjectCacheSection({ projectId, state }: ProjectCacheSectionProps) {
  const { t, i18n } = useTranslation();
  const { cache } = state;
  return (
    <section aria-labelledby="settings-cache" className="flex flex-col gap-3">
      <h3 id="settings-cache" className="text-sm font-semibold text-text">
        {t('settings.cache.title')}
      </h3>
      {projectId === null ? (
        <p className="text-xs text-text-muted">{t('settings.cache.noProject')}</p>
      ) : (
        <>
          <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
            <dt className="text-text-muted">{t('settings.cache.location')}</dt>
            <dd className="truncate text-text select-text" title={cache?.path}>
              {cache === null ? <Skeleton className="h-3 w-48" /> : cache.path}
            </dd>
            <dt className="text-text-muted">{t('settings.cache.size')}</dt>
            <dd className="text-text timecode">
              {cache === null ? (
                <Skeleton className="h-3 w-16" />
              ) : (
                byteSize(cache.sizeBytes, i18n.language)
              )}
            </dd>
          </dl>
          <div className="flex gap-2">
            <Button size="sm" variant="secondary" disabled={cache === null} onClick={state.reveal}>
              {t('settings.cache.reveal')}
            </Button>
            <AlertDialog>
              <AlertDialogTrigger asChild>
                <Button
                  size="sm"
                  variant="danger"
                  disabled={cache === null}
                  loading={state.clearing}
                >
                  {t('settings.cache.clear')}
                </Button>
              </AlertDialogTrigger>
              <AlertDialogContent size="sm">
                <AlertDialogHeader>
                  <AlertDialogTitle>{t('settings.cache.confirmTitle')}</AlertDialogTitle>
                  <AlertDialogDescription>
                    {t('settings.cache.confirmDescription')}
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel>{t('settings.cache.cancel')}</AlertDialogCancel>
                  <AlertDialogAction variant="danger" onClick={() => void state.clear()}>
                    {t('settings.cache.confirm')}
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </div>
        </>
      )}
    </section>
  );
}
