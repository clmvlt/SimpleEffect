/**
 * « À propos » (prompt 30): versions of the app, se-core, the engine, Electron, Chromium and the AI
 * CLI; « Exporter un diagnostic » (a zip written by the core with personal data masked); the
 * licences of every bundled component, npm package and Cargo crate; the performance measure of
 * prompt 29. `AboutDialog` is mounted once by the app layout; `AboutButton` (the status bar's
 * version) and `openAbout(tab)` open it.
 */
export { AboutButton } from './components/AboutButton';
export { AboutDialog } from './components/AboutDialog';
export { aboutDialogStore, openAbout, type AboutTab } from './store/about-dialog-store';
export { PROBED_CODECS, probeCodecs, type ProbedCodec } from './services/client-diagnostics';
export { browserProbes } from './services/browser-probes';
