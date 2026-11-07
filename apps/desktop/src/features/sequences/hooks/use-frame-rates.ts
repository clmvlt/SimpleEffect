import type { CoreClient, FrameRate } from '@se/protocol';
import { useEffect, useState } from 'react';

import { useCoreClient } from '../../../shared/core';

let cached: Promise<readonly FrameRate[]> | null = null;

/** The frame rates the core offers (`getNewProjectDefaults`), asked once per session. */
export function loadFrameRates(client: Pick<CoreClient, 'send'>): Promise<readonly FrameRate[]> {
  cached ??= client.send({ kind: 'getNewProjectDefaults' }).then(
    (defaults) => defaults.frameRates,
    (error: unknown) => {
      cached = null;
      throw error;
    },
  );
  return cached;
}

/** The frame rates offered in format dialogs; empty until the core answered. */
export function useFrameRates(): readonly FrameRate[] {
  const client = useCoreClient();
  const [rates, setRates] = useState<readonly FrameRate[]>([]);
  useEffect(() => {
    let live = true;
    loadFrameRates(client).then(
      (loaded) => {
        if (live) setRates(loaded);
      },
      (error: unknown) => {
        console.warn('[sequences] frame rates unavailable', error);
      },
    );
    return () => {
      live = false;
    };
  }, [client]);
  return rates;
}
