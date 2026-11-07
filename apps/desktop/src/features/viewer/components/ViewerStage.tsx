import type { SequenceEntity } from '@se/protocol';
import type { MediaLocator } from '@se/engine/preview';
import { useEffect, useRef, type CSSProperties, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { usePlaybackRate } from '../../../shared/playback';
import { useProjectSelector } from '../../../shared/project-store';
import { stopReframing, useReframingItem } from '../../../shared/reframe';
import { useElementSize } from '../hooks/use-element-size';
import { useViewerEngine } from '../hooks/use-viewer-engine';
import type { ViewerEngineController } from '../services/viewer-engine-controller';
import { viewerLayout } from '../services/viewer-layout';
import type { PlaybackScene } from '../services/viewer-scene';
import { guidesOf, type ViewerPreferences } from '../store/viewer-store';
import { FrameGuides } from './FrameGuides';
import { ReframeBackdrop, ReframeOverlay } from './ReframeOverlay';
import { TransformGizmo } from './TransformGizmo';
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
 * The clip being reframed in the viewer when it is a clip of `sequence` that fills the frame and
 * nothing plays; the mode ends by itself otherwise.
 */
function useReframing(sequence: SequenceEntity): string | null {
  const itemId = useReframingItem();
  const playing = usePlaybackRate() !== 0;
  const valid = useProjectSelector((state) => {
    const item = itemId === null ? undefined : state.mirror?.items[itemId];
    const track = item === undefined ? undefined : state.mirror?.tracks[item.trackId];
    const reframe = item?.kind.kind === 'mediaClip' ? item.kind.reframe.kind : null;
    return track?.sequenceId === sequence.id && (reframe === 'fill' || reframe === 'tracked');
  });
  const active = itemId !== null && valid && !playing;
  useEffect(() => {
    if (itemId !== null && !active) stopReframing();
  }, [itemId, active]);
  return active ? itemId : null;
}

/**
 * The picture area: the engine's canvas at the sequence ratio, fitted to the panel or at a fixed
 * zoom (scrolling when larger), with the loading and media indicators over it, the guides, and
 * the transform gizmo — or the reframing overlay while a clip is reframed.
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
  const reframing = useReframing(sequence);
  const running = status.kind === 'running';
  return (
    <div className="relative min-h-0 flex-1">
      <div ref={stage} className="absolute inset-0 overflow-auto">
        <div className="flex h-max min-h-full w-max min-w-full items-center justify-center overflow-clip p-2">
          <div
            className="relative shrink-0"
            style={{ width: layout.display.width, height: layout.display.height }}
          >
            {running && reframing !== null && (
              <ReframeBackdrop
                controller={controller}
                sequence={sequence}
                itemId={reframing}
                displayScale={layout.scale}
              />
            )}
            <div
              ref={host}
              role="img"
              aria-label={t('editor.viewer.picture')}
              data-viewer-picture=""
              className="absolute inset-0"
              style={preferences.checkerboard ? CHECKERBOARD : undefined}
            />
            <FrameGuides guides={guidesOf(preferences, sequence.id)} frame={sequence.format} />
            {running && reframing === null && (
              <TransformGizmo
                controller={controller}
                sequence={sequence}
                displayScale={layout.scale}
              />
            )}
            {running && reframing !== null && (
              <ReframeOverlay
                controller={controller}
                sequence={sequence}
                itemId={reframing}
                displayScale={layout.scale}
              />
            )}
          </div>
        </div>
      </div>
      <ViewerOverlays status={status} />
      {children}
    </div>
  );
}
