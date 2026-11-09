import type { CaptionStyle } from '@se/protocol';
import { useTranslation } from 'react-i18next';

/**
 * A caption look shown on a sample line (the library's caption styles in the browser): its font,
 * weight, case, colours — the second word as the active one —, outline and plate. Drawn by the
 * page, close enough to the engine for choosing a look.
 */
export function CaptionStyleSample({ style }: { readonly style: CaptionStyle }) {
  const { t } = useTranslation();
  const [first = '', second = '', ...rest] = t('editor.templates.browser.captionSample').split(' ');
  const outline = style.outline.enabled ? `0 0 2px ${style.outline.color}` : undefined;
  return (
    <span
      aria-hidden
      className="flex size-full items-end justify-center overflow-hidden p-1.5"
      style={{ fontFamily: style.fontFamily, fontWeight: style.fontWeight }}
    >
      <span
        className="max-w-full truncate rounded-xs px-1 text-xs leading-tight"
        style={{
          color: style.textColor,
          textTransform: style.textCase === 'uppercase' ? 'uppercase' : 'none',
          fontStyle: style.italic ? 'italic' : 'normal',
          textShadow: outline,
          backgroundColor: style.background.enabled ? style.background.color : undefined,
        }}
      >
        {first} <span style={{ color: style.activeColor }}>{second}</span> {rest.join(' ')}
      </span>
    </span>
  );
}
