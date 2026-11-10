import type { ExportInfo } from '@se/protocol';
import { Button, Popover, PopoverContent, PopoverTrigger, toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import { errorMessageKey } from '../../../shared/errors';
import { exportsApi } from '../api';
import { isActiveExport, useExports } from '../store/exports-store';
import { ExportRow } from './ExportRow';

/** The pill's text: how the running exports stand, or how many exports there are. */
function pillLabel(
  exports: readonly ExportInfo[],
  t: (key: string, options?: object) => string,
): string {
  const rendering = exports.find((info) => info.state.kind === 'rendering');
  const active = exports.filter(isActiveExport).length;
  if (rendering !== undefined && rendering.state.kind === 'rendering') {
    return t('export.pill.rendering', {
      percent: Math.round(rendering.state.progress * 100),
      count: active,
    });
  }
  if (active > 0) return t('export.pill.active', { count: active });
  return t('export.pill.idle');
}

/**
 * « Exports » in the editor header (prompt 27): shown once an export exists; its menu lists them
 * with progress, time left, cancel, the finished file revealed in its folder and « Effacer les
 * exports terminés ».
 */
export function ExportsButton() {
  const { t } = useTranslation();
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const exports = useExports();
  if (exports.length === 0) return null;
  const run = (action: Promise<void>) => {
    action.catch((error: unknown) => {
      toast.error(t(errorMessageKey(error)));
    });
  };
  const finished = exports.some((info) => !isActiveExport(info));
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button variant="ghost" size="sm" aria-label={t('export.pill.open')}>
          {pillLabel(exports, t as (key: string, options?: object) => string)}
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-96 p-0">
        <div className="flex items-center justify-between border-b border-border px-3 py-2">
          <span className="text-sm font-medium text-text">{t('export.list.title')}</span>
          <Button
            variant="ghost"
            size="xs"
            disabled={!finished}
            onClick={() => {
              run(exportsApi(client).clearFinished());
            }}
          >
            {t('export.list.clear')}
          </Button>
        </div>
        <ul className="max-h-96 divide-y divide-border overflow-y-auto px-3">
          {[...exports].reverse().map((info) => (
            <ExportRow
              key={info.id}
              info={info}
              onCancel={(target) => {
                run(exportsApi(client).cancel(target.id));
              }}
              onReveal={(target) => {
                run(bridge.showInFolder(target.path));
              }}
            />
          ))}
        </ul>
      </PopoverContent>
    </Popover>
  );
}
