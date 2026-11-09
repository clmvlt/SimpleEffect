import {
  Badge,
  IconButton,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@se/ui';
import { X } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useLibraryTemplates } from '../../templates';
import { withFavorite, withoutFavorite, type FavoriteList } from '../services/kit-draft';
import type { KitFieldsProps } from './KitIdentityFields';

const LISTS: readonly { readonly list: FavoriteList; readonly label: 'intro' | 'outro' }[] = [
  { list: 'introTemplates', label: 'intro' },
  { list: 'outroTemplates', label: 'outro' },
];

/**
 * The kit's favourite intro and outro templates, chosen among the library's motion templates
 * (agents read them in `get_brand_kit`).
 */
export function KitFavorites({ kit, onChange }: KitFieldsProps) {
  const { t } = useTranslation();
  const library = useLibraryTemplates().filter((entry) => entry.content.kind === 'motionTemplate');
  return (
    <section aria-labelledby="kit-favorites" className="flex flex-col gap-1.5">
      <h4 id="kit-favorites" className="text-xs font-medium text-text-muted">
        {t('brandKit.settings.favorites')}
      </h4>
      {LISTS.map(({ list, label }) => {
        const chosen = kit[list];
        const offered = library.filter((entry) => !chosen.some((item) => item.id === entry.id));
        return (
          <div key={list} className="flex flex-col gap-1">
            <span className="text-xs">{t(`brandKit.settings.${label}`)}</span>
            <ul className="flex flex-wrap gap-1">
              {chosen.length === 0 && (
                <li className="text-xs text-text-subtle">{t('brandKit.settings.noFavorite')}</li>
              )}
              {chosen.map((favorite) => (
                <li key={favorite.id}>
                  <Badge size="sm" className="gap-0.5 pr-0">
                    {favorite.name}
                    <IconButton
                      size="xs"
                      icon={X}
                      label={t('brandKit.settings.removeFavorite', { name: favorite.name })}
                      className="size-4"
                      onClick={() => {
                        onChange(withoutFavorite(kit, list, favorite.id));
                      }}
                    />
                  </Badge>
                </li>
              ))}
            </ul>
            {offered.length > 0 && (
              <Select
                value=""
                onValueChange={(value) => {
                  const entry = offered.find((candidate) => candidate.id === value);
                  if (entry !== undefined) onChange(withFavorite(kit, list, entry));
                }}
              >
                <SelectTrigger
                  size="xs"
                  className="w-full"
                  aria-label={t('brandKit.settings.addFavorite')}
                >
                  <SelectValue placeholder={t('brandKit.settings.addFavorite')} />
                </SelectTrigger>
                <SelectContent>
                  {offered.map((entry) => (
                    <SelectItem key={entry.id} value={entry.id}>
                      {entry.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          </div>
        );
      })}
    </section>
  );
}
