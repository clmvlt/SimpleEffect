import type { Flicks, SequenceEntity } from '@se/protocol';
import { useState } from 'react';

import { useCoreEndpoint } from '../../../shared/core';
import { ShortcutRegion } from '../../../shared/shortcuts';
import { useInOutSpan } from '../../timeline';
import { usePlaybackSession } from '../hooks/use-playback-session';
import { usePlaybackShortcuts } from '../hooks/use-playback-shortcuts';
import { useViewerFonts } from '../hooks/use-viewer-fonts';
import { useViewerScene } from '../hooks/use-viewer-scene';
import type { AudioEngineFactory } from '../services/audio-engine-factory';
import { createMediaLocator } from '../services/media-locator';
import type { PreviewEngineFactory } from '../services/preview-engine-factory';
import { ViewerEngineController } from '../services/viewer-engine-controller';
import { sceneContentEnd } from '../services/viewer-scene';
import type { ViewerPreferences } from '../store/viewer-store';
import { PerformanceHud } from './PerformanceHud';
import { ViewerStage } from './ViewerStage';
import { ViewerTransport } from './ViewerTransport';

/** Props of {@link ViewerWorkspace}. */
export interface ViewerWorkspaceProps {
  readonly sequence: SequenceEntity;
  readonly preferences: ViewerPreferences;
  readonly createEngine: PreviewEngineFactory;
  readonly createAudio: AudioEngineFactory;
}

/**
 * The viewer of a sequence: the picture engine and the playback session (both outside React),
 * the picture, the transport bar and the playback shortcuts (context `viewer`).
 */
export function ViewerWorkspace({
  sequence,
  preferences,
  createEngine,
  createAudio,
}: ViewerWorkspaceProps) {
  const [engine] = useState(() => new ViewerEngineController(createEngine));
  useViewerFonts(engine);
  const scene = useViewerScene(sequence);
  const endpoint = useCoreEndpoint();
  const locator = createMediaLocator(endpoint, scene?.assets ?? {});
  const contentEnd = scene === null ? (0 as Flicks) : sceneContentEnd(scene);
  const loopRange = useInOutSpan(sequence.id, contentEnd);
  const session = usePlaybackSession({
    createAudio,
    engine,
    sequence,
    scene,
    contentEnd,
    locator,
    loopRange,
    looping: preferences.loop,
    scrubAudio: preferences.scrubAudio,
  });
  usePlaybackShortcuts(session);
  return (
    <ShortcutRegion context="viewer" className="flex size-full flex-col">
      <ViewerStage
        controller={engine}
        sequence={sequence}
        scene={scene}
        locator={locator}
        preferences={preferences}
      >
        {preferences.hud && (
          <PerformanceHud engine={engine} session={session} sequence={sequence} />
        )}
      </ViewerStage>
      <ViewerTransport
        session={session}
        sequence={sequence}
        contentEnd={contentEnd}
        preferences={preferences}
      />
    </ShortcutRegion>
  );
}
