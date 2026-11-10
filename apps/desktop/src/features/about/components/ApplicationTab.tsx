import { Button, toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreInfo } from '../../../shared/core';
import { aiTextValues, useAiStatus } from '../../ai';
import { useExportDiagnostics } from '../hooks/use-export-diagnostics';
import { useRuntimeInfo } from '../hooks/use-runtime-info';
import { versionRows, versionsText, type VersionRow } from '../services/about-versions';

/** Props of {@link ApplicationTab}. */
export interface ApplicationTabProps {
  /** Opens the first-run wizard again (« Assistant de bienvenue »). */
  readonly onOpenWelcome: () => void;
}

/**
 * « Application »: the versions of the app, se-core, the engine, Electron, Chromium and the AI
 * CLI (copyable for bug reports), « Exporter un diagnostic » and the welcome wizard.
 */
export function ApplicationTab({ onOpenWelcome }: ApplicationTabProps) {
  const { t } = useTranslation();
  const ai = useAiStatus();
  const rows = versionRows({
    app: __APP_VERSION__,
    engine: __ENGINE_VERSION__,
    core: useCoreInfo()?.version ?? null,
    runtime: useRuntimeInfo(),
    ai,
  });
  const aiName = aiTextValues(ai, t('statusBar.ai.fallbackName')).name;
  const label = (row: VersionRow) => (row.id === 'ai' ? aiName : t(`about.versions.${row.id}`));
  const diagnostics = useExportDiagnostics();
  const copy = () => {
    navigator.clipboard
      .writeText(versionsText(rows, label))
      .then(() => toast.success(t('about.versions.copied')))
      .catch(() => toast.error(t('about.versions.copyFailed')));
  };
  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-text-muted">{t('about.tagline')}</p>
      <section aria-labelledby="about-versions" className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <h3 id="about-versions" className="text-sm font-semibold text-text">
            {t('about.versions.title')}
          </h3>
          <Button size="xs" variant="ghost" onClick={copy}>
            {t('about.versions.copy')}
          </Button>
        </div>
        <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-1 text-xs">
          {rows.map((row) => (
            <div key={row.id} className="contents" data-version={row.id}>
              <dt className="text-text-muted">{label(row)}</dt>
              <dd className="text-text tabular-nums select-text">
                {row.version ??
                  t(row.id === 'ai' ? 'about.versions.aiMissing' : 'about.versions.unknown')}
              </dd>
            </div>
          ))}
        </dl>
      </section>
      <section aria-labelledby="about-diagnostics" className="flex flex-col gap-2">
        <h3 id="about-diagnostics" className="text-sm font-semibold text-text">
          {t('about.diagnostics.title')}
        </h3>
        <p className="text-xs text-text-muted">{t('about.diagnostics.description')}</p>
        <div className="flex gap-2">
          <Button size="sm" onClick={diagnostics.start} disabled={diagnostics.exporting}>
            {diagnostics.exporting
              ? t('about.diagnostics.exporting')
              : t('about.diagnostics.export')}
          </Button>
          <Button size="sm" variant="outline" onClick={onOpenWelcome}>
            {t('about.welcome')}
          </Button>
        </div>
      </section>
    </div>
  );
}
