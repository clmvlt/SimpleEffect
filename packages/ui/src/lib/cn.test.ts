import { describe, expect, it } from 'vitest';

import { cn } from './cn';

describe('cn', () => {
  it('drops falsy values and joins the rest', () => {
    expect(cn('px-2', false, undefined, null, 'py-1')).toBe('px-2 py-1');
  });

  it('lets the last conflicting utility win', () => {
    expect(cn('px-2 text-text', 'px-3')).toBe('text-text px-3');
  });

  it('knows the custom control heights', () => {
    expect(cn('h-8', 'h-control-md')).toBe('h-control-md');
    expect(cn('size-control-sm', 'size-control-xs')).toBe('size-control-xs');
  });

  it('tells the editor font sizes apart from text colours', () => {
    expect(cn('text-sm text-text-muted', 'text-2xs')).toBe('text-text-muted text-2xs');
    expect(cn('text-text', 'text-accent')).toBe('text-accent');
  });

  it('knows the custom shadow, easing and animations', () => {
    expect(cn('shadow-overlay', 'shadow-none')).toBe('shadow-none');
    expect(cn('ease-out', 'ease-standard')).toBe('ease-standard');
    expect(cn('animate-spin', 'animate-content-in')).toBe('animate-content-in');
  });
});
