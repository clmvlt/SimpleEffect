/**
 * The open project's brand kit seen by fields (prompt 25): its palette as named swatches and its
 * fonts as the first group of font menus (`useBrandShortcuts`), and the pure helpers behind them.
 * Choosing and editing kits lives in `features/brand-kits`.
 */
export {
  PALETTE_ROLES,
  brandFonts,
  brandSwatches,
  isPaletteRole,
  type BrandSwatch,
  type PaletteRole,
} from './brand-shortcuts';
export { BrandColorField } from './BrandColorField';
export { useBrandShortcuts, type BrandShortcuts } from './use-brand-shortcuts';
