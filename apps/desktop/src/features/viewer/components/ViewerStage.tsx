import type { SequenceEntity } from '@se/protocol';
import type { MediaLocator } from '@se/engine/preview';
import { useRef, type CSSProperties, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { useElementSize } from '../hooks/use-element-size';
import { useViewerEngine } from '../hooks/use-viewer-engine';
import type { ViewerEngineController } from '../services/viewer-engine-controller';
import { viewerLayout } from '../services/viewer-layout';
import type { PlaybackScene } from '../services/viewer-scene';
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
  readonly controller: ViewerEngineController;
  readonly sequence: SequenceEntity;
  readonly scene: PlaybackScene | null;
  readonly locator: MediaLocator;
  readonly preferences: ViewerPreferences;
  /** Extra overlays over the picture (the performance HUD). */
  readonly children?: ReactNode;
}

/**
 * The picture area: the engine's canvas at the sequence ratio, fitted to the panel or at a fixed
 * zoom (scrolling when larger), with the loading and media indicators over it.
 */
export function ViewerStage({
  controller,
  sequence,
  scene,
  locator,
  preferences,
  children,
}: ViewerStageProps) {
  const { t } = useTranslation();
  const stage = useRef<HTMLDivElement | null>(null);
  const stageSize = useElementSize(stage);
  const layout = viewerLayout(
    stageSize,
    sequence.format,
    preferences.zoom,
    window.devicePixelRatio,
  );
  const { host, status } = useViewerEngine(controller, {
    sequenceId: sequence.id,
    scene,
    locator,
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
      {children}
    </div>
  );
}
