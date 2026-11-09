import { brandKitLogoPath, mediaUrl, type LibraryBrandKit } from '@se/protocol';
import { useTranslation } from 'react-i18next';

import { brandSwatches } from '../../../shared/brand-kit';
import { useCoreEndpoint } from '../../../shared/core';
import { useDocumentFonts } from '../../../shared/fonts';
import { staticValue } from '../services/kit-draft';

/**
 * A live preview of a kit (drawn by the page, for choosing values): a title in the new texts'
 * look over the background colour, body text in the body font, the main logo, a caption line in
 * the caption look with its spoken word, and the palette.
 */
export function BrandKitPreview({ kit }: { readonly kit: LibraryBrandKit }) {
  const { t } = useTranslation();
  const endpoint = useCoreEndpoint();
  const text = kit.textStyle;
  const caption = kit.captionStyle;
  const [logo] = kit.logos;
  useDocumentFonts([text.fontFamily, kit.fonts.body, caption.fontFamily]);
  const plate = text.background.enabled ? staticValue(text.background.color) : null;
  const outline = text.stroke.enabled ? staticValue(text.stroke.color) : null;
  const [spoken = '', ...rest] = t('brandKit.settings.previewCaption').split(' ');
  return (
    <figure aria-label={t('brandKit.settings.preview')} className="flex flex-col gap-2">
      <div
        className="relative flex aspect-[9/16] max-h-[44vh] flex-col items-center justify-between overflow-hidden rounded-md border border-border p-3"
        style={{ backgroundColor: kit.colors.background }}
      >
        {logo !== undefined && endpoint !== null ? (
          <img
            alt=""
            src={mediaUrl(endpoint, brandKitLogoPath(kit.id, logo.file), kit.version)}
            className="max-h-10 self-end object-contain"
          />
        ) : (
          <span />
        )}
        <div className="flex flex-col items-center gap-2 text-center">
          <span
            className="rounded-xs px-1 text-lg leading-tight"
            style={{
              fontFamily: text.fontFamily,
              fontWeight: text.fontWeight,
              color: staticValue(text.color) ?? kit.colors.text,
              textTransform: text.textCase === 'uppercase' ? 'uppercase' : 'none',
              backgroundColor: plate ?? undefined,
              WebkitTextStroke: outline === null ? undefined : `1px ${outline}`,
              textShadow: text.shadow.enabled ? '0 2px 4px var(--color-scrim)' : undefined,
            }}
          >
            {t('brandKit.settings.previewTitle')}
          </span>
          <span className="text-xs" style={{ fontFamily: kit.fonts.body, color: kit.colors.text }}>
            {t('brandKit.settings.previewBody')}
          </span>
        </div>
        <span
          className="rounded-xs px-1 text-xs"
          style={{
            fontFamily: caption.fontFamily,
            fontWeight: caption.fontWeight,
            color: caption.textColor,
            textTransform: caption.textCase === 'uppercase' ? 'uppercase' : 'none',
            backgroundColor: caption.background.enabled ? caption.background.color : undefined,
          }}
        >
          <span style={{ color: caption.activeColor }}>{spoken}</span> {rest.join(' ')}
        </span>
      </div>
      <figcaption className="flex gap-1">
        {brandSwatches(kit).map(({ role, color }) => (
          <span
            key={role}
            title={t(`brandKit.roles.${role}`)}
            className="h-4 flex-1 rounded-xs border border-border-strong"
            style={{ backgroundColor: color }}
          />
        ))}
      </figcaption>
    </figure>
  );
}
