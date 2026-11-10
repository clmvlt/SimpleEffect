import { Button } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useProjectsFolder } from '../hooks/use-projects-folder';

/** Props of {@link ProjectsStep}. */
export interface ProjectsStepProps {
  readonly active: boolean;
}

/** « Dossier des projets »: where new projects are proposed, and « Choisir un autre dossier… ». */
export function ProjectsStep({ active }: ProjectsStepProps) {
  const { t } = useTranslation();
  const { folder, choose } = useProjectsFolder(active);
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm text-text-muted">{t('onboarding.projects.intro')}</p>
      <div className="flex items-center gap-2 rounded-md border border-border bg-panel-raised p-2">
        <code
          className="min-w-0 flex-1 truncate text-xs text-text select-text"
          data-testid="projects-folder"
        >
          {folder ?? t('onboarding.projects.loading')}
        </code>
        <Button size="xs" onClick={choose}>
          {t('onboarding.projects.choose')}
        </Button>
      </div>
      <p className="text-2xs text-text-muted">{t('onboarding.projects.hint')}</p>
    </div>
  );
}
