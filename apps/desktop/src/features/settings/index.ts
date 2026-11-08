/**
 * Settings feature (prompt 09): the « Réglages » dialog opened by `SettingsButton` from the home
 * screen and the editor, or by `openSettings(section)` (the status bar opens « IA »). Section
 * « Médias »: mezzanine quality, proxy resolution, automatic proxies, the ffmpeg folder and what
 * it can do, and the open project's cache (size, reveal, clean). Section « IA » (prompt 17): the
 * AI CLI found, its login state with « Actualiser », a custom CLI path, the default model and
 * effort, and the number of tasks running at once.
 */
export { SettingsButton } from './components/SettingsButton';
export { openSettings, type SettingsSection } from './store/settings-dialog-store';
