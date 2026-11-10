import { Button, Input, Spinner } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useDesktopBridge, type ThirdPartyNotices } from '../../../shared/desktop';
import { useThirdPartyNotices } from '../hooks/use-third-party-notices';
import {
  filterNotices,
  listedNotices,
  sourceCounts,
  type ListedNotice,
} from '../services/licence-list';

/** Props of {@link LicencesTab}. */
export interface LicencesTabProps {
  /** Whether the tab shows (the notices are read the first time). */
  readonly active: boolean;
}

/**
 * « Licences »: every bundled component, npm package and Cargo crate with its licence, searchable,
 * the texts of the selected one, and « Ouvrir le dossier des licences ».
 */
export function LicencesTab({ active }: LicencesTabProps) {
  const { t } = useTranslation();
  const bridge = useDesktopBridge();
  const state = useThirdPartyNotices(active);
  const openFolder = () => {
    bridge.showLicencesFolder().catch((error: unknown) => {
      console.warn('[about] cannot open the licences folder', error);
    });
  };
  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between gap-2">
        <p className="text-xs text-text-muted">{t('about.licences.intro')}</p>
        <Button size="xs" variant="outline" onClick={openFolder}>
          {t('about.licences.openFolder')}
        </Button>
      </div>
      {state.kind === 'loading' && (
        <div className="flex justify-center py-8">
          <Spinner size="lg" label={t('about.licences.loading')} />
        </div>
      )}
      {state.kind === 'missing' && (
        <p className="text-xs text-text-muted">{t('about.licences.missing')}</p>
      )}
      {state.kind === 'failed' && (
        <p className="text-xs text-danger">{t('about.licences.failed')}</p>
      )}
      {state.kind === 'loaded' && <NoticesBrowser notices={state.notices} />}
    </div>
  );
}

function NoticesBrowser({ notices }: { readonly notices: ThirdPartyNotices }) {
  const { t } = useTranslation();
  const [query, setQuery] = useState('');
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const all = listedNotices(notices.entries);
  const shown = filterNotices(all, query);
  const selected = shown.find((notice) => notice.key === selectedKey) ?? shown[0] ?? null;
  const counts = sourceCounts(notices.entries);
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-3">
        <Input
          size="sm"
          type="search"
          value={query}
          onChange={(event) => {
            setQuery(event.target.value);
          }}
          placeholder={t('about.licences.search')}
          aria-label={t('about.licences.search')}
          className="max-w-xs"
        />
        <p className="text-2xs text-text-muted">{t('about.licences.counts', counts)}</p>
      </div>
      <div className="grid h-[45vh] grid-cols-[minmax(12rem,1fr)_2fr] gap-3">
        <ul
          aria-label={t('about.licences.list')}
          className="overflow-auto rounded-md border border-border"
        >
          {shown.map((notice) => (
            <li key={notice.key}>
              <button
                type="button"
                aria-current={notice.key === selected?.key}
                onClick={() => {
                  setSelectedKey(notice.key);
                }}
                className="flex w-full items-baseline justify-between gap-2 px-2 py-1 text-left text-xs hover:bg-hover aria-[current=true]:bg-accent-muted"
              >
                <span className="truncate text-text">{notice.name}</span>
                <span className="shrink-0 text-2xs text-text-muted">{notice.license}</span>
              </button>
            </li>
          ))}
        </ul>
        {selected === null ? (
          <p className="text-xs text-text-muted">{t('about.licences.noMatch')}</p>
        ) : (
          <NoticeDetails notice={selected} texts={notices.texts} />
        )}
      </div>
    </div>
  );
}

function NoticeDetails({
  notice,
  texts,
}: {
  readonly notice: ListedNotice;
  readonly texts: ThirdPartyNotices['texts'];
}) {
  const { t } = useTranslation();
  const bridge = useDesktopBridge();
  const url = notice.url?.startsWith('https://') === true ? notice.url : null;
  return (
    <section aria-label={notice.name} className="flex min-h-0 flex-col gap-2 overflow-auto">
      <div>
        <h3 className="text-sm font-semibold text-text">
          {notice.name} <span className="font-normal text-text-muted">{notice.version}</span>
        </h3>
        <p className="text-xs text-text-muted">
          {t(`about.licences.sources.${notice.source}`)} · {notice.license}
        </p>
        {notice.note !== undefined && <p className="text-xs text-text-muted">{notice.note}</p>}
        {url !== null && (
          <Button
            size="xs"
            variant="ghost"
            className="-ml-2"
            onClick={() => {
              void bridge.openExternal(url);
            }}
          >
            {url}
          </Button>
        )}
      </div>
      {notice.textIds.length === 0 ? (
        <p className="text-xs text-text-muted">{t('about.licences.noText')}</p>
      ) : (
        notice.textIds.map((id) => (
          <pre
            key={id}
            className="rounded-md bg-panel-raised p-2 text-2xs whitespace-pre-wrap text-text select-text"
          >
            {texts[String(id)]}
          </pre>
        ))
      )}
    </section>
  );
}
