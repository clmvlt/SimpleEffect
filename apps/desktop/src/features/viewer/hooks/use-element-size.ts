import { useEffect, useState, type RefObject } from 'react';

import type { Size } from '../services/viewer-layout';

const NO_SIZE: Size = { width: 0, height: 0 };

/** The element's content size, updated whenever it is resized (zero before it is measured). */
export function useElementSize(element: RefObject<HTMLElement | null>): Size {
  const [size, setSize] = useState<Size>(NO_SIZE);
  useEffect(() => {
    const node = element.current;
    if (node === null || typeof ResizeObserver === 'undefined') return undefined;
    const observer = new ResizeObserver(([entry]) => {
      if (entry === undefined) return;
      const { width, height } = entry.contentRect;
      setSize((current) =>
        current.width === width && current.height === height ? current : { width, height },
      );
    });
    observer.observe(node);
    return () => {
      observer.disconnect();
    };
  }, [element]);
  return size;
}
