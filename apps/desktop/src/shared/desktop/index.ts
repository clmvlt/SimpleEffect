/**
 * Native capabilities of the Electron shell (folder picker, reveal in the file manager),
 * reached through the preload bridge and injected by context so tests can fake them.
 */
export type {
  ChooseFilesOptions,
  ChooseFolderOptions,
  NoticeEntry,
  RuntimeInfo,
  SimpleEffectBridge,
  ThirdPartyNotices,
} from '../../../electron/shared/bridge-api';
export { DesktopBridgeProvider } from './DesktopBridgeProvider';
export type { DesktopBridgeProviderProps } from './DesktopBridgeProvider';
export { useDesktopBridge } from './use-desktop-bridge';
