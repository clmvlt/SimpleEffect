import type { Preview } from '@storybook/react-vite';
import { themes } from 'storybook/theming';

import '../src/styles.css';

/**
 * Every story renders on the app background (`bg` token, set on `<body>` by `styles.css`), so
 * Storybook's own backgrounds are disabled. The a11y addon runs axe on each story; a violation
 * fails the story's accessibility check in the panel.
 */
const preview: Preview = {
  parameters: {
    layout: 'centered',
    backgrounds: { disable: true },
    controls: { expanded: true },
    docs: { theme: themes.dark },
    a11y: { test: 'error' },
    options: {
      storySort: { order: ['Foundations', ['Principles', 'Tokens', 'Theme'], 'Primitives'] },
    },
  },
  tags: ['autodocs'],
};

export default preview;
