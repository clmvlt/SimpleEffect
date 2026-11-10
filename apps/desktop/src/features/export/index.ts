/**
 * Video and sound export (prompt 27, ADR-0038): « Exporter » (Ctrl/⌘+M) opens the dialog —
 * presets, custom settings, range, destination, estimate and warnings —; « Exports » in the header
 * follows them (progress, time left, cancel, reveal). `useExportHost` (mounted by `AppLayout`)
 * mirrors the core's exports, draws the renders it asks for with `@se/engine/export` and keeps
 * the computer awake while exports run.
 */
export { ExportButton, type ExportButtonProps } from './components/ExportButton';
export { ExportsButton } from './components/ExportsButton';
export { useExportHost } from './hooks/use-export-host';
