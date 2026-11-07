import { Icon, Input, ToggleGroup, ToggleGroupItem, Toolbar, Tooltip } from '@se/ui';
import { LayoutGrid, List, Search } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';
import { useStore } from 'zustand';

import type { KindFilter } from '../services/asset-list';
import { mediaBinStore, updateMediaBin } from '../store/media-bin-store';

const KINDS: readonly KindFilter[] = ['all', 'video', 'audio', 'image'];

/** Search field, kind filter and grid/list switch of the media bin. */
export function MediaBinToolbar() {
  const { t } = useTranslation();
  const { view, kind, query } = useStore(mediaBinStore);
  return (
    <div className="flex shrink-0 flex-col gap-1.5 border-b border-border p-2">
      <div className="relative">
        <Icon
          icon={Search}
          size="sm"
          className="pointer-events-none absolute top-1/2 left-2 -translate-y-1/2 text-text-subtle"
        />
        <Input
          type="search"
          size="sm"
          value={query}
          aria-label={t('editor.media.toolbar.search')}
          placeholder={t('editor.media.toolbar.search')}
          className="pl-7"
          onChange={(event) => {
            updateMediaBin({ query: event.target.value });
          }}
        />
      </div>
      <Toolbar label={t('editor.media.toolbar.label')} className="justify-between gap-2">
        <ToggleGroup
          type="single"
          variant="segmented"
          size="xs"
          value={kind}
          aria-label={t('editor.media.toolbar.filter')}
          onValueChange={(value) => {
            if (value !== '') updateMediaBin({ kind: value as KindFilter });
          }}
        >
          {KINDS.map((item) => (
            <ToggleGroupItem key={item} value={item} className="px-2 text-2xs">
              {t(`editor.media.kinds.${item}`)}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
        <ToggleGroup
          type="single"
          size="xs"
          value={view}
          aria-label={t('editor.media.toolbar.view')}
          onValueChange={(value) => {
            if (value === 'grid' || value === 'list') updateMediaBin({ view: value });
          }}
        >
          <Tooltip content={t('editor.media.toolbar.grid')}>
            <ToggleGroupItem value="grid" aria-label={t('editor.media.toolbar.grid')}>
              <Icon icon={LayoutGrid} size="sm" />
            </ToggleGroupItem>
          </Tooltip>
          <Tooltip content={t('editor.media.toolbar.list')}>
            <ToggleGroupItem value="list" aria-label={t('editor.media.toolbar.list')}>
              <Icon icon={List} size="sm" />
            </ToggleGroupItem>
          </Tooltip>
        </ToggleGroup>
      </Toolbar>
    </div>
  );
}
