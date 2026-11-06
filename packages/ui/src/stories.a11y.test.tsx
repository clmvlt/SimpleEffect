// Runs axe-core on every story, so an accessibility regression fails `pnpm test` and not only the
// Storybook a11y panel. jsdom has no layout engine: colour contrast is checked separately on the
// tokens (`tokens/contrast.test.ts`) and visually in Storybook.
import { composeStories, setProjectAnnotations } from '@storybook/react-vite';
import { render } from '@testing-library/react';
import axe from 'axe-core';
import type { FunctionComponent } from 'react';
import { describe, expect, it } from 'vitest';

import preview from '../.storybook/preview';

setProjectAnnotations(preview);

type StoryModule = Parameters<typeof composeStories>[0];

const storyModules = import.meta.glob<StoryModule>('./**/*.stories.tsx', { eager: true });

/** WCAG 2.2 A and AA rules, minus colour contrast (needs a real layout engine). */
const AXE_OPTIONS: axe.RunOptions = {
  runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'] },
  rules: { 'color-contrast': { enabled: false } },
};

function describeViolations(violations: axe.Result[]): string[] {
  return violations.map(
    (violation) =>
      `${violation.id} (${violation.impact ?? 'unknown'}): ${violation.help} — ${violation.nodes
        .map((node) => node.target.join(' '))
        .join(', ')}`,
  );
}

/** Composed stories are components; the generic module type only knows them as `unknown`. */
function isComposedStory(entry: [string, unknown]): entry is [string, FunctionComponent] {
  return typeof entry[1] === 'function';
}

describe.each(Object.entries(storyModules))('%s', (_path, storyModule) => {
  const stories = Object.entries(composeStories(storyModule)).filter(isComposedStory);

  it.each(stories)('%s has no accessibility violation', async (_name, Story) => {
    render(<Story />);
    const results = await axe.run(document.body, AXE_OPTIONS);
    expect(describeViolations(results.violations)).toEqual([]);
  });
});
