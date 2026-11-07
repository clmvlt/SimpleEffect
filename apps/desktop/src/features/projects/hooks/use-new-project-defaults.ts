import type { NewProjectDefaults } from '@se/protocol';
import { useEffect, useEffectEvent, useState } from 'react';

/** Loads the dialog's defaults each time it opens; `null` while loading or after a failure. */
export function useNewProjectDefaults(
  open: boolean,
  load: () => Promise<NewProjectDefaults>,
  onError: (error: unknown) => void,
): NewProjectDefaults | null {
  const [defaults, setDefaults] = useState<NewProjectDefaults | null>(null);
  const loadDefaults = useEffectEvent(load);
  const fail = useEffectEvent(onError);
  useEffect(() => {
    if (!open) return undefined;
    let current = true;
    loadDefaults().then(
      (loaded) => {
        if (current) setDefaults(loaded);
      },
      (error: unknown) => {
        if (current) fail(error);
      },
    );
    return () => {
      current = false;
      setDefaults(null);
    };
  }, [open]);
  return defaults;
}
