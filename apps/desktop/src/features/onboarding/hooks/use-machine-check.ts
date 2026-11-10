import type { CodecSupport, MediaToolsStatus } from '@se/protocol';
import { useEffect, useState } from 'react';

import { useCoreClient } from '../../../shared/core';
import { browserProbes, probeCodecs } from '../../about';
import { getMediaToolsStatus } from '../api';

/** What the « Binaires & GPU » step shows; each part is `null` while being checked. */
export interface MachineCheck {
  /** ffmpeg/ffprobe and the hardware encoders; `'failed'` when the core could not tell. */
  readonly tools: MediaToolsStatus | 'failed' | null;
  /** The GPU as WebGL names it (`undefined` while checking, `null` when hidden). */
  readonly gpu: string | null | undefined;
  readonly codecs: readonly CodecSupport[] | null;
}

/** Checks the media tools, the GPU and WebCodecs whenever `active` becomes true. */
export function useMachineCheck(active: boolean): MachineCheck {
  const client = useCoreClient();
  const [check, setCheck] = useState<MachineCheck>({ tools: null, gpu: undefined, codecs: null });
  useEffect(() => {
    if (!active) return;
    let current = true;
    const probes = browserProbes();
    // Asynchronously, like the other checks: the effect itself only starts them.
    void Promise.resolve().then(() => {
      if (current) setCheck((previous) => ({ ...previous, gpu: probes.gpu() }));
    });
    getMediaToolsStatus(client)
      .then((tools) => {
        if (current) setCheck((previous) => ({ ...previous, tools }));
      })
      .catch((error: unknown) => {
        console.warn('[onboarding] media tools status unavailable', error);
        if (current) setCheck((previous) => ({ ...previous, tools: 'failed' }));
      });
    void probeCodecs(probes).then((codecs) => {
      if (current) setCheck((previous) => ({ ...previous, codecs }));
    });
    return () => {
      current = false;
    };
  }, [active, client]);
  return check;
}
