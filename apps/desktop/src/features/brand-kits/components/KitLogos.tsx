import { brandKitLogoPath, mediaUrl } from '@se/protocol';
import { Button, IconButton, Input } from '@se/ui';
import { ImagePlus, X } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useCoreEndpoint } from '../../../shared/core';
import { withLogoName, withoutLogo } from '../services/kit-draft';
import type { KitFieldsProps } from './KitIdentityFields';

/**
 * The kit's logos: a thumbnail (served from the kit's folder), a name to edit and « Retirer »
 * each, and « Ajouter un logo… » (images copied into the kit at once). The first logo is the
 * main one (template parameters bound to `logo` take it).
 */
export function KitLogos({
  kit,
  onChange,
  onAdd,
}: KitFieldsProps & { readonly onAdd: () => void }) {
  const { t } = useTranslation();
  const endpoint = useCoreEndpoint();
  return (
    <section aria-labelledby="kit-logos" className="flex flex-col gap-1.5">
      <h4 id="kit-logos" className="text-xs font-medium text-text-muted">
        {t('brandKit.settings.logos')}
      </h4>
      {kit.logos.length === 0 && (
        <p className="text-xs text-text-subtle">{t('brandKit.settings.noLogo')}</p>
      )}
      <ul className="flex flex-col gap-1">
        {kit.logos.map((logo) => (
          <li key={logo.file} className="flex items-center gap-2">
            <span className="flex size-8 shrink-0 items-center justify-center rounded-xs bg-bg">
              {endpoint !== null && (
                <img
                  alt=""
                  src={mediaUrl(endpoint, brandKitLogoPath(kit.id, logo.file), kit.version)}
                  className="max-h-full max-w-full object-contain"
                />
              )}
            </span>
            <Input
              size="xs"
              value={logo.name}
              aria-label={t('brandKit.settings.logoName')}
              className="min-w-0 flex-1"
              onChange={(event) => {
                onChange(withLogoName(kit, logo.file, event.target.value));
              }}
            />
            <IconButton
              size="xs"
              icon={X}
              label={t('brandKit.settings.removeLogo', { name: logo.name })}
              onClick={() => {
                onChange(withoutLogo(kit, logo.file));
              }}
            />
          </li>
        ))}
      </ul>
      <Button variant="secondary" size="xs" className="self-start" onClick={onAdd}>
        <ImagePlus aria-hidden className="size-3.5" />
        {t('brandKit.settings.addLogo')}
      </Button>
    </section>
  );
}
