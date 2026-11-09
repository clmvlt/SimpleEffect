import { EmptyState, IconButton, Input, ToggleGroup, ToggleGroupItem } from '@se/ui';
import { FileUp, Search, WandSparkles } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { useBrowserActions } from '../hooks/use-browser-actions';
import {
  BROWSER_CATEGORIES,
  browserEntries,
  filterEntries,
  type BrowserCategory,
  type BrowserSource,
} from '../services/browser-entries';
import { useLibraryTemplates } from '../store/library-store';
import { BrowserCard } from './BrowserCard';

const SOURCES: readonly BrowserSource[] = ['all', 'project', 'library'];
/** Value of the « Toutes » category chip. */
const ALL_CATEGORIES = 'all';

const NO_TEMPLATES: readonly never[] = [];

/**
 * The template browser (prompt 25, the « Modèles » tab of the Animations panel): the project's
 * motion templates and the user's library (motion templates and caption styles) as a grid of
 * previewed cards, searched by name, description and tags, filtered by category and by where they
 * live; cards insert at the playhead, drag to the timeline, save to the library, update from it,
 * export; « Importer… » reads `.setemplate` files.
 */
export function TemplateBrowser() {
  const { t } = useTranslation();
  const templates = useProjectSelector((state) => state.mirror?.project.templates) ?? NO_TEMPLATES;
  const items = useProjectSelector((state) => state.mirror?.items);
  const library = useLibraryTemplates();
  const actions = useBrowserActions();
  const [source, setSource] = useState<BrowserSource>('all');
  const [category, setCategory] = useState<BrowserCategory | null>(null);
  const [search, setSearch] = useState('');
  const used = new Set<string>();
  for (const item of Object.values(items ?? {})) {
    if (item.kind.kind === 'motionTemplate') used.add(item.kind.templateId);
  }
  const all = browserEntries(templates, library, used);
  const shown = filterEntries(all, { source, category, search });
  return (
    <section aria-label={t('editor.templates.browser.title')} className="flex flex-col gap-2 p-2">
      <div className="flex items-center gap-1">
        <div className="relative min-w-0 flex-1">
          <Input
            size="xs"
            type="search"
            value={search}
            aria-label={t('editor.templates.browser.search')}
            placeholder={t('editor.templates.browser.search')}
            className="pl-6"
            onChange={(event) => {
              setSearch(event.target.value);
            }}
          />
          <Search
            aria-hidden
            className="pointer-events-none absolute top-1/2 left-1.5 size-3.5 -translate-y-1/2 text-text-subtle"
          />
        </div>
        <IconButton
          size="xs"
          icon={FileUp}
          label={t('editor.templates.browser.importTitle')}
          disabled={!actions.ready}
          onClick={actions.importFiles}
        />
      </div>
      <ToggleGroup
        type="single"
        variant="segmented"
        size="xs"
        value={source}
        aria-label={t('editor.templates.browser.sourceLabel')}
        onValueChange={(value) => {
          const chosen = SOURCES.find((candidate) => candidate === value);
          if (chosen !== undefined) setSource(chosen);
        }}
      >
        {SOURCES.map((option) => (
          <ToggleGroupItem key={option} value={option}>
            {t(`editor.templates.browser.sources.${option}`)}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      <ToggleGroup
        type="single"
        size="xs"
        value={category ?? ALL_CATEGORIES}
        aria-label={t('editor.templates.browser.categoryLabel')}
        className="flex-wrap"
        onValueChange={(value) => {
          setCategory(BROWSER_CATEGORIES.find((candidate) => candidate === value) ?? null);
        }}
      >
        <ToggleGroupItem value={ALL_CATEGORIES}>
          {t('editor.templates.categories.all')}
        </ToggleGroupItem>
        {BROWSER_CATEGORIES.map((option) => (
          <ToggleGroupItem key={option} value={option}>
            {t(`editor.templates.categories.${option}`)}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      {shown.length === 0 ? (
        <EmptyState
          icon={WandSparkles}
          title={
            all.length === 0
              ? t('editor.templates.browser.emptyTitle')
              : t('editor.templates.browser.noMatch')
          }
          description={
            all.length === 0 ? t('editor.templates.browser.emptyDescription') : undefined
          }
        />
      ) : (
        <ul className="grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-2">
          {shown.map((entry) => (
            <BrowserCard key={entry.key} entry={entry} actions={actions} />
          ))}
        </ul>
      )}
    </section>
  );
}
