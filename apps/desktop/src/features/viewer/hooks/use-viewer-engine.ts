import type { SequenceId } from '@se/protocol';
import type { CompositorBackground, MediaLocator, RenderScene } from '@se/engine/preview';
import { useEffect, useRef, useSyncExternalStore, type RefObject } from 'react';

import { playheadStore, playheadTimeOf } from '../../timeline';
import type {
  ViewerEngineController,
  ViewerEngineStatus,
} from '../services/viewer-engine-controller';
import type { Size } from '../services/viewer-layout';

/** What the viewer shows and how. */
export interface ViewerEngineInputs {
  readonly sequenceId: SequenceId;
  readonly scene: RenderScene | null;
  readonly locator: MediaLocator;
  readonly proxy: boolean;
  readonly background: CompositorBackground;
  readonly pixels: Size;
}

/** Window property exposing the engine's counters in development (open `VideoFrame`s). */
interface DevWindow {
  simpleEffectPreviewStats?: () => unknown;
}

/**
 * Thin bridge between React and the engine, which lives outside React (`controller`): it mounts
 * the engine in the returned host element, forwards the scene, options and size, and follows the
 * playhead store directly (no re-render per scrub step or played frame).
 */
export function useViewerEngine(
  controller: ViewerEngineController,
  inputs: ViewerEngineInputs,
): {
  readonly host: RefObject<HTMLDivElement | null>;
  readonly status: ViewerEngineStatus;
} {
  const host = useRef<HTMLDivElement | null>(null);
  const status = useSyncExternalStore(controller.subscribe, controller.getStatus);
  const { sequenceId, scene, locator, proxy, background, pixels } = inputs;

  useEffect(() => {
    const element = host.current;
    if (element === null) return undefined;
    controller.attach(element);
    if (!import.meta.env.DEV)
      return () => {
        controller.detach();
      };
    (window as DevWindow).simpleEffectPreviewStats = () => controller.stats();
    return () => {
      delete (window as DevWindow).simpleEffectPreviewStats;
      controller.detach();
    };
  }, [controller]);

  useEffect(() => {
    controller.setScene(scene);
  }, [controller, scene]);
  useEffect(() => {
    controller.setLocator(locator);
  }, [controller, locator]);
  useEffect(() => {
    controller.setProxy(proxy);
  }, [controller, proxy]);
  useEffect(() => {
    controller.setBackground(background);
  }, [controller, background]);
  useEffect(() => {
    controller.setPixels(pixels);
  }, [controller, pixels]);
  useEffect(() => {
    controller.setTime(playheadTimeOf(playheadStore.getState(), sequenceId));
    return playheadStore.subscribe((state) => {
      controller.setTime(playheadTimeOf(state, sequenceId));
    });
  }, [controller, sequenceId]);

  return { host, status };
}
