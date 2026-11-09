import { useEffect } from 'react';

import { useCoreEndpoint } from '../../../shared/core';
import { useFontFamilies } from '../../../shared/fonts';
import { publishTemplateIssues, templateRuntimeFor } from '../../../shared/templates';
import type { ViewerEngineController } from '../services/viewer-engine-controller';

/**
 * Gives the viewer's engine the app's motion template runtime (prompt 24; again when the core
 * restarts or the font catalogue arrives) and publishes the failing template instances of each
 * drawn frame for the inspector.
 */
export function useViewerTemplates(controller: ViewerEngineController): void {
  const families = useFontFamilies();
  const endpoint = useCoreEndpoint();
  useEffect(() => {
    if (endpoint === null) return;
    let current = true;
    templateRuntimeFor({ endpoint, families }).then(
      (runtime) => {
        if (current) controller.setTemplates(runtime);
      },
      () => undefined,
    );
    return () => {
      current = false;
    };
  }, [controller, endpoint, families]);
  useEffect(
    () =>
      controller.subscribe(() => {
        const status = controller.getStatus();
        publishTemplateIssues(status.kind === 'running' ? status.preview.templateIssues : []);
      }),
    [controller],
  );
}
