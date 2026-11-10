import { useEffect, useState } from 'react';

import { useDesktopBridge, type RuntimeInfo } from '../../../shared/desktop';

/** Versions of Electron, Chromium, Node.js and V8 from the main process, `null` until known. */
export function useRuntimeInfo(): RuntimeInfo | null {
  const bridge = useDesktopBridge();
  const [info, setInfo] = useState<RuntimeInfo | null>(null);
  useEffect(() => {
    let current = true;
    bridge
      .getRuntimeInfo()
      .then((loaded) => {
        if (current) setInfo(loaded);
      })
      .catch((error: unknown) => {
        console.warn('[about] runtime versions unavailable', error);
      });
    return () => {
      current = false;
    };
  }, [bridge]);
  return info;
}
