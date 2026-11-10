import type { ExportRenderDeps } from '../services/export-render';
import { useExportActivity } from './use-export-activity';
import { useExportRenders } from './use-export-renders';
import { useExportsSync } from './use-exports-sync';

/**
 * Everything exports need while the app runs (prompt 27): the mirrored list and its notices, the
 * renders the core asks for, and the anti-sleep / quit confirmation. Mounted once by `AppLayout`.
 */
export function useExportHost(renderDeps: Partial<ExportRenderDeps> = {}): void {
  useExportsSync();
  useExportRenders(renderDeps);
  useExportActivity();
}
