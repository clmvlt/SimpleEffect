import { useEffect, useState } from 'react';

/**
 * `flag`, but turning on only after it stayed on for `delayMs`: a loading indicator does not
 * flash for decodes that finish almost at once. It turns off immediately.
 */
export function useDelayedFlag(flag: boolean, delayMs: number): boolean {
  const [shown, setShown] = useState(false);
  useEffect(() => {
    if (!flag) return undefined;
    const timer = setTimeout(() => {
      setShown(true);
    }, delayMs);
    return () => {
      clearTimeout(timer);
      setShown(false);
    };
  }, [flag, delayMs]);
  return flag && shown;
}
