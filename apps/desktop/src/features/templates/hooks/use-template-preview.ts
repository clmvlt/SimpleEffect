import { FLICKS_PER_SECOND, templateValues } from '@se/engine';
import type { TemplateLayer } from '@se/engine/evaluate';
import type { MotionTemplate } from '@se/protocol';
import { useEffect, useRef, type RefObject } from 'react';

import { useCoreEndpoint } from '../../../shared/core';
import { useFontFamilies } from '../../../shared/fonts';
import { templateRuntimeFor, templatesSupported } from '../../../shared/templates';

/** Size of a preview, in pixels (16:9). */
export const PREVIEW_WIDTH = 192;
export const PREVIEW_HEIGHT = 108;

/** The design frame previews draw in. */
const DESIGN = { width: 1920, height: 1080 };

/** A layer of `template` at `seconds`, with its default values. */
function previewLayer(template: MotionTemplate, seconds: number): TemplateLayer {
  const duration = template.defaultDuration / FLICKS_PER_SECOND;
  return {
    kind: 'template',
    itemId: `preview:${template.id}`,
    template: { id: template.id, version: template.version, source: template.source },
    params: templateValues(template, {}, 0 as MotionTemplate['defaultDuration']),
    time: seconds,
    duration,
    box: DESIGN,
    transform: {
      position: { x: 0.5, y: 0.5 },
      scale: { x: 1, y: 1 },
      rotation: 0,
      anchor: { x: 0.5, y: 0.5 },
      opacity: 1,
      blur: 0,
    },
  };
}

/**
 * Draws `template` with its default values on `canvas` through the app's template runtime: its
 * middle at rest, its whole length looping while `playing` (hovered). Does nothing where
 * templates cannot run (jsdom).
 */
export function useTemplatePreview(
  canvas: RefObject<HTMLCanvasElement | null>,
  template: MotionTemplate,
  playing: boolean,
): void {
  const endpoint = useCoreEndpoint();
  const families = useFontFamilies();
  const busy = useRef(false);
  useEffect(() => {
    if (endpoint === null || !templatesSupported()) return;
    const duration = template.defaultDuration / FLICKS_PER_SECOND;
    let stopped = false;
    let frame = 0;
    const started = performance.now();
    const draw = (seconds: number) => {
      if (busy.current) return;
      busy.current = true;
      void templateRuntimeFor({ endpoint, families })
        .then(async (runtime) => {
          const result = await runtime.render({
            layer: previewLayer(template, seconds),
            width: PREVIEW_WIDTH,
            height: PREVIEW_HEIGHT,
            designWidth: DESIGN.width,
            designHeight: DESIGN.height,
          });
          const context = canvas.current?.getContext('2d');
          if (!stopped && context) {
            context.clearRect(0, 0, PREVIEW_WIDTH, PREVIEW_HEIGHT);
            context.drawImage(result.lease.picture.source, 0, 0, PREVIEW_WIDTH, PREVIEW_HEIGHT);
          }
          result.lease.release();
        })
        .catch(() => undefined)
        .finally(() => {
          busy.current = false;
        });
    };
    const tick = () => {
      if (stopped) return;
      draw(((performance.now() - started) / 1000) % duration);
      frame = requestAnimationFrame(tick);
    };
    if (playing) tick();
    else draw(duration / 2);
    return () => {
      stopped = true;
      cancelAnimationFrame(frame);
    };
  }, [canvas, template, playing, endpoint, families]);
}
