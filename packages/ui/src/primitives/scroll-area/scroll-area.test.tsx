import { render, screen } from '@testing-library/react';
import { createRef } from 'react';
import { describe, expect, it } from 'vitest';

import { ScrollArea } from './scroll-area';

describe('ScrollArea', () => {
  it('renders its children inside the scrolling viewport', () => {
    const viewportRef = createRef<HTMLDivElement>();
    render(
      <ScrollArea className="h-32" viewportRef={viewportRef}>
        <p>Journal de rendu</p>
      </ScrollArea>,
    );
    const viewport = viewportRef.current;
    expect(viewport).toHaveAttribute('data-radix-scroll-area-viewport');
    expect(viewport).toContainElement(screen.getByText('Journal de rendu'));
    expect(viewport).not.toHaveAttribute('tabindex');
  });

  it('makes the viewport reachable with the keyboard when asked', () => {
    const viewportRef = createRef<HTMLDivElement>();
    render(
      <ScrollArea keyboardScrollable viewportRef={viewportRef} aria-label="Journal">
        <p>Ligne 1</p>
      </ScrollArea>,
    );
    expect(viewportRef.current).toHaveAttribute('tabindex', '0');
  });
});
