import type { ExportInfo } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

/** The exports of this core session as the app knows them, oldest first. */
export const exportsStore = createStore<{ readonly exports: readonly ExportInfo[] }>()(() => ({
  exports: [],
}));

/** Whether `info` still needs the app (queued, rendering or being finalised). */
export function isActiveExport(info: ExportInfo): boolean {
  const { kind } = info.state;
  return kind === 'queued' || kind === 'rendering' || kind === 'finalizing';
}

/** Replaces the list (after `listExports`). */
export function setExports(exports: readonly ExportInfo[]): void {
  exportsStore.setState({ exports });
}

/** Puts `info` in place of the export with its id, or after the others. */
export function upsertExport(info: ExportInfo): void {
  const { exports } = exportsStore.getState();
  const index = exports.findIndex((known) => known.id === info.id);
  exportsStore.setState({
    exports:
      index < 0 ? [...exports, info] : exports.map((known, at) => (at === index ? info : known)),
  });
}

/** Forgets finished, failed and cancelled exports (`exportsCleared`). */
export function dropFinishedExports(): void {
  exportsStore.setState({ exports: exportsStore.getState().exports.filter(isActiveExport) });
}

/** Forgets everything (tests). */
export function resetExports(): void {
  exportsStore.setState({ exports: [] });
}

/** The exports, oldest first. */
export function useExports(): readonly ExportInfo[] {
  return useStore(exportsStore, (state) => state.exports);
}
