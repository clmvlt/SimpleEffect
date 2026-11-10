import { useEffect, useState } from 'react';

import { useDesktopBridge, type ThirdPartyNotices } from '../../../shared/desktop';

/** The licence notices while `active`. */
export type NoticesState =
  | { readonly kind: 'loading' }
  | { readonly kind: 'missing' }
  | { readonly kind: 'failed' }
  | { readonly kind: 'loaded'; readonly notices: ThirdPartyNotices };

/** Reads the licence notices the first time `active` becomes true (they weigh ~700 KB). */
export function useThirdPartyNotices(active: boolean): NoticesState {
  const bridge = useDesktopBridge();
  const [state, setState] = useState<NoticesState>({ kind: 'loading' });
  const loaded = state.kind !== 'loading';
  useEffect(() => {
    if (!active || loaded) return;
    let current = true;
    bridge
      .getThirdPartyNotices()
      .then((notices) => {
        if (current) setState(notices === null ? { kind: 'missing' } : { kind: 'loaded', notices });
      })
      .catch((error: unknown) => {
        console.warn('[about] licence notices unreadable', error);
        if (current) setState({ kind: 'failed' });
      });
    return () => {
      current = false;
    };
  }, [active, loaded, bridge]);
  return state;
}
