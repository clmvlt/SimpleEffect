import type { LibraryBrandKit } from '@se/protocol';
import { Button, EmptyState, Spinner, cn } from '@se/ui';
import { Palette, Plus } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useBrandKitActions } from '../hooks/use-brand-kit-actions';
import { useBrandKits } from '../store/brand-kits-store';
import { BrandKitEditor } from './BrandKitEditor';

/**
 * « Réglages → Brand kits » (prompt 25): the library's kits on the left (« Nouveau brand kit »),
 * the chosen kit's editor on the right. `projectId` is the open project (its kit can be chosen
 * from here), `null` on the home screen.
 */
export function BrandKitsSection({ projectId }: { readonly projectId: string | null }) {
  const { t } = useTranslation();
  const kits = useBrandKits();
  const actions = useBrandKitActions();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  if (kits === null) {
    return (
      <div className="flex justify-center py-8">
        <Spinner size="lg" label={t('settings.media.checking')} />
      </div>
    );
  }
  const selected = kits.find((kit) => kit.id === selectedId) ?? kits[0] ?? null;
  const create = (copyOf: LibraryBrandKit | null) => {
    const name =
      copyOf === null
        ? t('brandKit.settings.newName')
        : t('brandKit.settings.copyName', { name: copyOf.name });
    void actions.create(name, copyOf).then((kit) => {
      if (kit !== null) setSelectedId(kit.id);
    });
  };
  return (
    <div className="flex flex-col gap-3">
      <p className="text-xs text-text-muted">{t('brandKit.settings.description')}</p>
      <div className="flex min-h-[26rem] gap-3">
        <nav aria-label={t('brandKit.settings.list')} className="flex w-44 shrink-0 flex-col gap-1">
          <ul className="flex flex-col gap-0.5">
            {kits.map((kit) => (
              <li key={kit.id}>
                <button
                  type="button"
                  aria-current={kit.id === selected?.id ? 'true' : undefined}
                  className={cn(
                    'flex w-full items-center gap-2 rounded-sm px-2 py-1 text-left text-sm hover:bg-hover',
                    kit.id === selected?.id && 'bg-accent-muted',
                  )}
                  onClick={() => {
                    setSelectedId(kit.id);
                  }}
                >
                  <span
                    aria-hidden
                    className="size-3 shrink-0 rounded-full border border-border-strong"
                    style={{ backgroundColor: kit.colors.primary }}
                  />
                  <span className="truncate">{kit.name}</span>
                </button>
              </li>
            ))}
          </ul>
          <Button
            variant="ghost"
            size="sm"
            className="justify-start"
            onClick={() => {
              create(null);
            }}
          >
            <Plus aria-hidden className="size-4" />
            {t('brandKit.settings.new')}
          </Button>
        </nav>
        <div className="min-w-0 flex-1">
          {selected === null ? (
            <EmptyState
              icon={Palette}
              title={t('brandKit.settings.empty')}
              description={t('brandKit.settings.emptyDescription')}
            />
          ) : (
            <BrandKitEditor
              key={selected.id}
              kit={selected}
              projectId={projectId}
              onDuplicate={() => {
                create(selected);
              }}
            />
          )}
        </div>
      </div>
    </div>
  );
}
