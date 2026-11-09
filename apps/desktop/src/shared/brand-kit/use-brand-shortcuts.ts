import type { ColorSwatch } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../project-store';
import { brandFonts, brandSwatches } from './brand-shortcuts';

/** What the open project's brand kit offers colour and font fields as shortcuts. */
export interface BrandShortcuts {
  /** The palette, labelled by role (« Accent »); empty without a kit. */
  readonly palette: readonly ColorSwatch[];
  /** The heading and body families; empty without a kit. */
  readonly fonts: readonly string[];
}

/**
 * The open project's brand kit as shortcuts for colour fields (named swatches) and font menus
 * (prompt 25): every colour and font field of the inspector and the caption style uses them.
 */
export function useBrandShortcuts(): BrandShortcuts {
  const { t } = useTranslation();
  const kit = useProjectSelector((state) => state.mirror?.project.brandKit ?? null);
  return {
    palette: brandSwatches(kit).map(({ role, color }) => ({
      color,
      label: t(`brandKit.roles.${role}`),
    })),
    fonts: brandFonts(kit),
  };
}
