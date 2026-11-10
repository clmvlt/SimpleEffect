import type { ExportSettings } from '@se/protocol';
import { useEffect, useEffectEvent, useState } from 'react';

import { isMasterCodec } from '../services/export-draft';
import type { EncoderSupport } from '../services/export-warnings';

/** Finds which encoders this machine offers for `settings` at `frameRate` frames per second. */
export type EncoderProbe = (settings: ExportSettings, frameRate: number) => Promise<EncoderSupport>;

/**
 * The real probe: `@se/engine/export` (imported lazily) asks WebCodecs, hardware first. Without
 * WebCodecs nothing can be encoded here, except masters (ffmpeg) and PCM.
 */
export const probeEncoders: EncoderProbe = async (settings, frameRate) => {
  if (typeof VideoEncoder === 'undefined') {
    const master = settings.video !== null && isMasterCodec(settings.video.codec);
    return {
      video: settings.video === null ? null : master ? 'ffmpeg' : 'none',
      audio: settings.audio === null ? null : settings.audio.codec === 'pcm' ? 'webCodecs' : 'none',
    };
  }
  const engine = await import('@se/engine/export');
  const [video, audio] = await Promise.all([
    engine.videoEncoderSupport(settings.video, frameRate),
    engine.audioEncoderSupport(settings.audio),
  ]);
  return { video, audio };
};

/** What decides the probe's answer (who encodes the sound does not: it follows the answer). */
function probeKey(settings: ExportSettings, frameRate: number): string {
  const audio = settings.audio === null ? null : { ...settings.audio, encoder: 'webCodecs' };
  return JSON.stringify([settings.container, settings.video, audio, frameRate]);
}

/**
 * The encoders found for `settings`, `null` while they are being probed (again after every
 * change that matters).
 */
export function useEncoderSupport(
  settings: ExportSettings,
  frameRate: number,
  probe: EncoderProbe,
): EncoderSupport | null {
  const key = probeKey(settings, frameRate);
  const [found, setFound] = useState<{
    readonly key: string;
    readonly support: EncoderSupport;
  } | null>(null);
  const run = useEffectEvent(() => probe(settings, frameRate));
  useEffect(() => {
    let current = true;
    run().then(
      (support) => {
        if (current) setFound({ key, support });
      },
      (error: unknown) => {
        console.warn('[export] probing encoders failed', error);
      },
    );
    return () => {
      current = false;
    };
  }, [key]);
  return found?.key === key ? found.support : null;
}
