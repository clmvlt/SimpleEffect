/**
 * Contract between the Electron main process, the preload script and the renderer.
 * Types and constants only: this file is bundled into all three.
 */

/** How the renderer reaches the running se-core process. */
export interface CoreConnection {
  /** HTTP origin of se-core, e.g. `http://127.0.0.1:51234`. The RPC socket is `/rpc` on it. */
  readonly url: string;
  /** Session token to pass as the `t` query parameter. Never log it. */
  readonly token: string;
  /** Version announced by se-core in its handshake. */
  readonly version: string;
}

/** Options of {@link SimpleEffectBridge.chooseFolder}. */
export interface ChooseFolderOptions {
  /** Window title of the native dialog (translated by the renderer). */
  readonly title: string;
  /** Label of the confirm button (translated by the renderer). */
  readonly buttonLabel: string;
  /** Folder shown first, when it exists. */
  readonly defaultPath?: string;
}

/** Options of {@link SimpleEffectBridge.chooseFiles}. */
export interface ChooseFilesOptions {
  /** Window title of the native dialog (translated by the renderer). */
  readonly title: string;
  /** Label of the confirm button (translated by the renderer). */
  readonly buttonLabel: string;
  /** Name of the file type filter (translated), e.g. « Médias ». */
  readonly filterName: string;
  /** Extensions of the filter, without dots (`mp4`, `wav`…). */
  readonly extensions: readonly string[];
  /** Name of the « all files » filter (translated). */
  readonly allFilesName: string;
}

/** Options of {@link SimpleEffectBridge.chooseFile}. */
export interface ChooseFileOptions {
  /** Window title of the native dialog (translated by the renderer). */
  readonly title: string;
  /** Label of the confirm button (translated by the renderer). */
  readonly buttonLabel: string;
}

/** API exposed by the preload script as `window.simpleEffect`. */
export interface SimpleEffectBridge {
  /**
   * Resolves with the connection of the running core, waiting while it (re)starts.
   * Call it again after every disconnect: a restarted core has a new port and token.
   */
  getCoreConnection(): Promise<CoreConnection>;
  /** Shows the native folder picker; resolves with the absolute path, or `null` if cancelled. */
  chooseFolder(options: ChooseFolderOptions): Promise<string | null>;
  /**
   * Shows the native file picker (several files allowed); resolves with their absolute paths,
   * or an empty list if cancelled.
   */
  chooseFiles(options: ChooseFilesOptions): Promise<string[]>;
  /**
   * Shows the native file picker for one file of any type (an executable); resolves with its
   * absolute path, or `null` if cancelled.
   */
  chooseFile(options: ChooseFileOptions): Promise<string | null>;
  /**
   * Opens an `https:` page in the user's default browser (installation guides). Other URLs are
   * refused by the main process.
   */
  openExternal(url: string): Promise<void>;
  /** Reveals a file or folder in the OS file manager (Explorer, Finder…). */
  showInFolder(path: string): Promise<void>;
  /**
   * Absolute path of a `File` dropped from the OS (`webUtils.getPathForFile`); an empty string
   * for files that are not on disk (e.g. dragged from a web page).
   */
  getPathForFile(file: File): string;
}

/** IPC channels between preload and main. */
export const IPC_CHANNELS = {
  getCoreConnection: 'core:get-connection',
  chooseFolder: 'dialog:choose-folder',
  chooseFiles: 'dialog:choose-files',
  chooseFile: 'dialog:choose-file',
  openExternal: 'shell:open-external',
  showInFolder: 'shell:show-in-folder',
} as const;
