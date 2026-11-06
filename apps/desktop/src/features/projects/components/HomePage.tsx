import { Button, EmptyState, Icon } from '@se/ui';
import { Clapperboard, Plus } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';
import { Link } from 'react-router';

/** Route of the editor opened by the development preview link. */
const EDITOR_PREVIEW_PATH = '/project/preview';

/**
 * The home screen: the project list, empty for now.
 * TODO(prompt-07): list recent projects, enable « Nouveau projet » and remove the preview link.
 */
export function HomePage() {
  const { t } = useTranslation();
  return (
    <main className="flex size-full items-center justify-center bg-bg">
      <h1 className="sr-only">{t('home.title')}</h1>
      <EmptyState
        size="md"
        icon={Clapperboard}
        title={t('home.empty.title')}
        description={t('home.empty.description')}
        actions={
          <>
            <Button variant="primary" size="md" disabled>
              <Icon icon={Plus} size="md" />
              {t('home.newProject')}
            </Button>
            {import.meta.env.DEV && (
              <Button variant="ghost" size="md" asChild>
                <Link to={EDITOR_PREVIEW_PATH}>{t('home.openEditorPreview')}</Link>
              </Button>
            )}
          </>
        }
      />
    </main>
  );
}
