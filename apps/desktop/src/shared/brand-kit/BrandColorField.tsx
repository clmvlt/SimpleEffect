import { ColorField, type ColorFieldProps } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useBrandShortcuts } from './use-brand-shortcuts';

/**
 * A `ColorField` offering the open project's brand kit palette as named swatches (prompt 25):
 * every colour field of the inspector and the caption style uses it. `palette` overrides the
 * project's kit (the kit editor shows its own draft).
 */
export function BrandColorField({ palette, labels, ...props }: ColorFieldProps) {
  const { t } = useTranslation();
  const shortcuts = useBrandShortcuts();
  return (
    <ColorField
      {...props}
      palette={palette ?? shortcuts.palette}
      labels={{ palette: t('brandKit.paletteGroup'), ...labels }}
    />
  );
}
