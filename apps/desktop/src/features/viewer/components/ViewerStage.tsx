import type { SequenceEntity } from '@se/protocol';
import { useRef, type CSSProperties } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreEndpoint } from '../../../shared/core';
import { useElementSize } from '../hooks/use-element-size';
import { useViewerEngine } from '../hooks/use-viewer-engine';
import { useViewerScene } from '../hooks/use-viewer-scene';
import { createMediaLocator } from '../services/media-locator';
import type { PreviewEngineFactory } from '../services/preview-engine-factory';
import { viewerLayout } from '../services/viewer-layout';
import type { ViewerPreferences } from '../store/viewer-store';
import { ViewerOverlays } from './ViewerOverlays';

/** Transparent areas over a checkerboard of two surface tokens. */
const CHECKERBOARD: CSSProperties = {
  backgroundImage:
    'repeating-conic-gradient(var(--color-control) 0% 25%, var(--color-panel-raised) 0% 50%)',
  backgroundSize: '16px 16px',
};

/** Props of {@link ViewerStage}. */
export interface ViewerStageProps {
  readonly sequence: SequenceEntity;
  readonly preferences: ViewerPreferences;
  readonly createEngine: PreviewEngineFactory;
}

/**
 * The picture area: the engine's canvas at the sequence ratio, fitted to the panel or at a fixed
 * zoom (scrolling when larger), with the loading and media indicators over it.
 */
export function ViewerStage({ sequence, preferences, createEngine }: ViewerStageProps) {
  const { t } = useTranslation();
  const stage = useRef<HTMLDivElement | null>(null);
  const stageSize = useElementSize(stage);
  const endpoint = useCoreEndpoint();
  const scene = useViewerScene(sequence);
  const layout = viewerLayout(
    stageSize,
    sequence.format,
    preferences.zoom,
    window.devicePixelRatio,
  );
  const { host, status } = useViewerEngine({
    createEngine,
    sequenceId: sequence.id,
    scene,
    locator: createMediaLocator(endpoint, scene?.assets ?? {}),
    proxy: preferences.proxy,
    background: preferences.checkerboard ? 'transparent' : 'black',
    pixels: layout.pixels,
  });
  return (
    <div className="relative min-h-0 flex-1">
      <div ref={stage} className="absolute inset-0 overflow-auto">
        <div className="flex h-max min-h-full w-max min-w-full items-center justify-center p-2">
          <div
            ref={host}
            role="img"
            aria-label={t('editor.viewer.picture')}
            data-viewer-picture=""
            className="shrink-0"
            style={{
              width: layout.display.width,
              height: layout.display.height,
              ...(preferences.checkerboard ? CHECKERBOARD : {}),
            }}
          />
        </div>
      </div>
      <ViewerOverlays status={status} />
    </div>
  );
}
