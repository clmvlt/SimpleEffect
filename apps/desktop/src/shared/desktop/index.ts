/**
 * Native capabilities of the Electron shell (folder picker, reveal in the file manager),
 * reached through the preload bridge and injected by context so tests can fake them.
 */
export type { ChooseFolderOptions, SimpleEffectBridge } from '../../../electron/shared/bridge-api';
export { DesktopBridgeProvider } from './DesktopBridgeProvider';
export type { DesktopBridgeProviderProps } from './DesktopBridgeProvider';
export { useDesktopBridge } from './use-desktop-bridge';
