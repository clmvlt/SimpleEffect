import { Button, Icon, cn } from '@se/ui';
import { Info, TriangleAlert } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { isBlocking, type ExportWarning } from '../services/export-warnings';

/** Props of {@link ExportWarnings}. */
export interface ExportWarningsProps {
  readonly warnings: readonly ExportWarning[];
  /** Opens « Créer une version… » of the sequence (aspect warnings). */
  readonly onCreateVersion: () => void;
}

/** Kinds that only inform (the export goes as expected, slower or through ffmpeg). */
const INFORMATIVE = new Set<ExportWarning['kind']>(['softwareEncoding', 'aacByFfmpeg']);

function WarningText({ warning }: { readonly warning: ExportWarning }) {
  const { t } = useTranslation();
  switch (warning.kind) {
    case 'missingMedia':
    case 'preparingMedia':
      return t(`export.warnings.${warning.kind}`, { count: warning.count });
    case 'missingFonts':
      return t('export.warnings.missingFonts', {
        count: warning.families.length,
        families: warning.families.join(', '),
      });
    case 'aspect':
      return t('export.warnings.aspect', {
        ratio: `${String(warning.aspect.width)}:${String(warning.aspect.height)}`,
      });
    default:
      return t(`export.warnings.${warning.kind}`);
  }
}

/** What to know before exporting, blocking problems first. */
export function ExportWarnings({ warnings, onCreateVersion }: ExportWarningsProps) {
  const { t } = useTranslation();
  if (warnings.length === 0) return null;
  return (
    <ul className="flex flex-col gap-1.5" aria-label={t('export.warnings.label')}>
      {warnings.map((warning) => {
        const informative = INFORMATIVE.has(warning.kind);
        return (
          <li
            key={warning.kind}
            role={isBlocking(warning) ? 'alert' : undefined}
            className={cn(
              'flex items-start gap-2 text-xs',
              isBlocking(warning)
                ? 'text-danger'
                : informative
                  ? 'text-text-muted'
                  : 'text-warning',
            )}
          >
            <Icon icon={informative ? Info : TriangleAlert} size="sm" className="mt-px shrink-0" />
            <span className="min-w-0 flex-1">
              <WarningText warning={warning} />
            </span>
            {warning.kind === 'aspect' && (
              <Button type="button" variant="outline" size="xs" onClick={onCreateVersion}>
                {t('export.warnings.createVersion')}
              </Button>
            )}
          </li>
        );
      })}
    </ul>
  );
}
