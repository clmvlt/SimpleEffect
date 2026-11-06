import tailwindcss from '@tailwindcss/vite';
import type { StorybookConfig } from '@storybook/react-vite';
import react from '@vitejs/plugin-react';

/**
 * Storybook for @se/ui: the living documentation of the design system (`pnpm storybook`).
 * Stories sit next to their components; foundation pages (principles, tokens, theme) live in
 * `src/docs/`. Telemetry and update checks are off: the project never phones home (golden rule 1).
 */
const config: StorybookConfig = {
  framework: '@storybook/react-vite',
  stories: ['../src/docs/*.mdx', '../src/**/*.stories.tsx'],
  addons: ['@storybook/addon-docs', '@storybook/addon-a11y'],
  core: { disableTelemetry: true, disableWhatsNewNotifications: true },
  viteFinal: (viteConfig) => ({
    ...viteConfig,
    plugins: [...(viteConfig.plugins ?? []), react(), tailwindcss()],
  }),
};

export default config;
