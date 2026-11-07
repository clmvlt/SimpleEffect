import { useEffect, type RefObject } from 'react';

/** Calls `onWidth` with the element's width now and whenever it is resized. */
export function useElementWidth(
  element: RefObject<HTMLElement | null>,
  onWidth: (width: number) => void,
): void {
  useEffect(() => {
    const node = element.current;
    if (node === null) return undefined;
    onWidth(Math.round(node.getBoundingClientRect().width));
    if (typeof ResizeObserver === 'undefined') return undefined;
    const observer = new ResizeObserver(([entry]) => {
      if (entry !== undefined) onWidth(Math.round(entry.contentRect.width));
    });
    observer.observe(node);
    return () => {
      observer.disconnect();
    };
  }, [element, onWidth]);
}
