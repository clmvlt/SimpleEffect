import type { MediaSettings, MediaToolsStatus } from '@se/protocol';
import { Button, Spinner } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useDesktopBridge } from '../../../shared/desktop';

/** Props of {@link ToolsSection}. */
export interface ToolsSectionProps {
  readonly settings: MediaSettings;
  /** `null` while checking. */
  readonly tools: MediaToolsStatus | null;
  readonly onSave: (changes: Partial<MediaSettings>) => void;
}

/** The ffmpeg folder (bundled tools or the user's), what was found and the hardware encoders. */
export function ToolsSection({ settings, tools, onSave }: ToolsSectionProps) {
  const { t } = useTranslation();
  const bridge = useDesktopBridge();
  const chooseFolder = async () => {
    const folder = await bridge.chooseFolder({
      title: t('settings.media.chooseToolsDirTitle'),
      buttonLabel: t('settings.media.chooseToolsDirButton'),
      ...(settings.toolsDir === null ? {} : { defaultPath: settings.toolsDir }),
    });
    if (folder !== null) onSave({ toolsDir: folder });
  };
  return (
    <section aria-labelledby="settings-tools" className="flex flex-col gap-3">
      <h3 id="settings-tools" className="text-sm font-semibold text-text">
        {t('settings.media.toolsTitle')}
      </h3>
      <div className="flex flex-col gap-1.5">
        <span className="text-xs font-medium text-text">{t('settings.media.toolsDir')}</span>
        <div className="flex items-center gap-2">
          <span
            className="min-w-0 flex-1 truncate text-xs text-text-muted select-text"
            title={settings.toolsDir ?? undefined}
          >
            {settings.toolsDir ?? t('settings.media.toolsDirDefault')}
          </span>
          <Button size="xs" variant="secondary" onClick={() => void chooseFolder()}>
            {t('settings.media.chooseToolsDir')}
          </Button>
          {settings.toolsDir !== null && (
            <Button
              size="xs"
              variant="ghost"
              onClick={() => {
                onSave({ toolsDir: null });
              }}
            >
              {t('settings.media.resetToolsDir')}
            </Button>
          )}
        </div>
      </div>
      <ToolsStatus tools={tools} />
    </section>
  );
}

function ToolsStatus({ tools }: { tools: MediaToolsStatus | null }) {
  const { t } = useTranslation();
  if (tools === null) {
    return (
      <div className="flex items-center gap-2 text-xs text-text-muted" role="status">
        <Spinner size="sm" label={t('settings.media.checking')} />
        {t('settings.media.checking')}
      </div>
    );
  }
  return (
    <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
      {tools.problem !== null && (
        <dd className="col-span-2 text-warning" role="alert">
          {tools.problem}
        </dd>
      )}
      {tools.ffmpegPath !== null && (
        <>
          <dt className="text-text-muted">
            {t('settings.media.ffmpegFound', { version: tools.ffmpegVersion ?? '?' })}
          </dt>
          <dd className="truncate text-text-muted select-text" title={tools.ffmpegPath}>
            {t('settings.media.ffmpegPath', { path: tools.ffmpegPath })}
          </dd>
        </>
      )}
      <dt className="text-text-muted">{t('settings.media.hardware')}</dt>
      <dd className="text-text">
        {tools.hardwareEncoders.length === 0
          ? t('settings.media.hardwareNone')
          : tools.hardwareEncoders.map((name) => name.toUpperCase()).join(', ')}
      </dd>
    </dl>
  );
}
