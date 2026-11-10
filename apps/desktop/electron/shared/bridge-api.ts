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

/** Options of {@link SimpleEffectBridge.chooseSaveFile}. */
export interface ChooseSaveFileOptions {
  /** Window title of the native dialog (translated by the renderer). */
  readonly title: string;
  /** Label of the confirm button (translated by the renderer). */
  readonly buttonLabel: string;
  /** File name proposed (no folder), e.g. `Interview.srt`. */
  readonly defaultName: string;
  /** Name of the file type filter (translated), e.g. « Sous-titres SRT ». */
  readonly filterName: string;
  /** Extensions of the filter, without dots (`srt`). */
  readonly extensions: readonly string[];
}

/** A notification of the operating system (an AI task finished while the window was hidden). */
export interface SystemNotification {
  /** Its title (translated by the renderer). */
  readonly title: string;
  /** Its text (translated by the renderer). */
  readonly body: string;
  /** Given back to `onNotificationClick` listeners when the user clicks it (a task id). */
  readonly tag: string;
}

/** Options of {@link SimpleEffectBridge.resolveFilePath}. */
export interface ResolveFilePathOptions {
  /** An absolute folder. */
  readonly folder: string;
  /** A file name in it (no folder separators). */
  readonly fileName: string;
}

/** A file path built by the main process with the platform's path rules. */
export interface ResolvedFilePath {
  /** The absolute path of the file. */
  readonly path: string;
  /** Whether a file already exists there (the renderer asks before replacing it). */
  readonly exists: boolean;
}

/**
 * Exports running in the renderer (prompt 27): while one is given, the system does not suspend
 * the app and closing the window asks first, with these texts (translated by the renderer).
 */
export interface ExportActivity {
  /** Title of the confirmation dialog. */
  readonly title: string;
  /** Its message (how many exports would stop). */
  readonly message: string;
  /** Label of the button that closes anyway. */
  readonly quitLabel: string;
  /** Label of the button that keeps the app open. */
  readonly stayLabel: string;
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
   * Shows the native « save as » dialog; resolves with the absolute path chosen (the OS asked
   * about replacing an existing file), or `null` if cancelled.
   */
  chooseSaveFile(options: ChooseSaveFileOptions): Promise<string | null>;
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
  /** Shows a system notification (nothing happens where the OS has none). */
  showNotification(notification: SystemNotification): Promise<void>;
  /**
   * Calls `listener` with the tag of a system notification the user clicked (the window was
   * brought to the front first); returns the function that stops listening.
   */
  onNotificationClick(listener: (tag: string) => void): () => void;
  /**
   * The absolute path of `fileName` in `folder`, joined with the platform's rules, and whether a
   * file exists there (export destinations, prompt 27).
   */
  resolveFilePath(options: ResolveFilePathOptions): Promise<ResolvedFilePath>;
  /**
   * Tells the main process exports are running (`null`: none any more): it keeps the system from
   * suspending the app and asks before the window closes.
   */
  setExportActivity(activity: ExportActivity | null): Promise<void>;
}

/** IPC channels between preload and main. */
export const IPC_CHANNELS = {
  getCoreConnection: 'core:get-connection',
  chooseFolder: 'dialog:choose-folder',
  chooseFiles: 'dialog:choose-files',
  chooseFile: 'dialog:choose-file',
  chooseSaveFile: 'dialog:choose-save-file',
  openExternal: 'shell:open-external',
  showInFolder: 'shell:show-in-folder',
  showNotification: 'notification:show',
  notificationClicked: 'notification:clicked',
  resolveFilePath: 'fs:resolve-file-path',
  setExportActivity: 'export:set-activity',
} as const;
