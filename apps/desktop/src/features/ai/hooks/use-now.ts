import { useEffect, useState } from 'react';

/** The current time (ms since the epoch), refreshed every `intervalMs` (1 s) while `ticking`. */
export function useNow(ticking: boolean, intervalMs = 1000): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!ticking) return;
    const timer = setInterval(() => {
      setNow(Date.now());
    }, intervalMs);
    return () => {
      clearInterval(timer);
    };
  }, [ticking, intervalMs]);
  return now;
}
