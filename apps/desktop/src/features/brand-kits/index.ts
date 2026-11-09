/**
 * Brand kits (prompt 25, ADR-0036): the editor header's « Brand kit » menu (`BrandKitButton`:
 * the project's kit, every library kit, an update when the library holds a newer version, a kit
 * made elsewhere added to the library) and « Réglages → Brand kits » (`BrandKitsSection`: the
 * library's kits, each edited as a draft with a live preview and saved as a new version). Fields
 * of other features read the project's kit through `shared/brand-kit`.
 */
export { BrandKitButton } from './components/BrandKitButton';
export { BrandKitsSection } from './components/BrandKitsSection';
