import { mediaUrl, type ExportInfo, type Flicks } from '@se/protocol';
import { Icon, IconButton, Progress } from '@se/ui';
import { Film, FolderOpen, X } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useCoreEndpoint } from '../../../shared/core';
import { byteSize } from '../../../shared/text';
import { FLICKS_PER_SECOND, formatDuration } from '../../../shared/time';
import { fileNameOf } from '../services/export-notice';
import { isActiveExport } from '../store/exports-store';

/** Props of {@link ExportRow}. */
export interface ExportRowProps {
  readonly info: ExportInfo;
  readonly onCancel: (info: ExportInfo) => void;
  readonly onReveal: (info: ExportInfo) => void;
}

const clock = (seconds: number) =>
  formatDuration(Math.round(seconds * FLICKS_PER_SECOND) as Flicks);

function Thumbnail({ info }: { readonly info: ExportInfo }) {
  const endpoint = useCoreEndpoint();
  if (!info.hasThumbnail || endpoint === null) {
    return (
      <span className="flex h-9 w-16 shrink-0 items-center justify-center rounded-sm bg-control text-text-subtle">
        <Icon icon={Film} size="sm" />
      </span>
    );
  }
  const url = mediaUrl(endpoint, `/media/exports/${encodeURIComponent(info.id)}/thumbnail`);
  return <img src={url} alt="" className="h-9 w-16 shrink-0 rounded-sm bg-control object-cover" />;
}

function StateLine({ info }: { readonly info: ExportInfo }) {
  const { t, i18n } = useTranslation();
  const { state } = info;
  switch (state.kind) {
    case 'queued':
    case 'cancelled':
    case 'finalizing':
      return <span className="text-text-muted">{t(`export.list.state.${state.kind}`)}</span>;
    case 'rendering': {
      const percent = Math.round(state.progress * 100);
      return (
        <span className="text-text-muted">
          {state.etaSeconds === null
            ? t('export.list.percent', { percent })
            : t('export.list.remaining', { percent, time: clock(state.etaSeconds) })}
        </span>
      );
    }
    case 'completed':
      return (
        <span className="flex flex-col">
          <span className="text-text-muted">
            {t('export.list.completed', {
              size: byteSize(state.sizeBytes, i18n.language),
              time: clock(state.elapsedSeconds),
            })}
          </span>
          {state.check?.warning != null && (
            <span className="text-warning">{state.check.warning}</span>
          )}
        </span>
      );
    case 'failed':
      return <span className="break-words text-danger">{state.message}</span>;
  }
}

/** One export of the « Exports » menu: picture, name, progress or outcome, actions. */
export function ExportRow({ info, onCancel, onReveal }: ExportRowProps) {
  const { t } = useTranslation();
  const name = fileNameOf(info.path);
  const { state } = info;
  return (
    <li className="flex items-start gap-2 py-2" aria-label={name}>
      <Thumbnail info={info} />
      <div className="flex min-w-0 flex-1 flex-col gap-1 text-xs">
        <span className="truncate font-medium text-text" title={info.path}>
          {name}
        </span>
        <span className="truncate text-text-subtle">
          {t('export.list.origin', { preset: info.presetName, sequence: info.sequenceName })}
        </span>
        {(state.kind === 'rendering' || state.kind === 'finalizing') && (
          <Progress
            size="sm"
            value={state.kind === 'rendering' ? Math.round(state.progress * 100) : null}
            aria-label={t('export.list.progress', { name })}
          />
        )}
        <StateLine info={info} />
      </div>
      {isActiveExport(info) && (
        <IconButton
          icon={X}
          size="xs"
          label={t('export.list.cancel', { name })}
          onClick={() => {
            onCancel(info);
          }}
        />
      )}
      {state.kind === 'completed' && (
        <IconButton
          icon={FolderOpen}
          size="xs"
          label={t('export.list.reveal', { name })}
          onClick={() => {
            onReveal(info);
          }}
        />
      )}
    </li>
  );
}
