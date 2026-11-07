import { playwright } from '@vitest/browser-playwright';
import { defineProject } from 'vitest/config';

/**
 * Golden-frame tests of `@se/engine` in headless Chromium (Playwright): real WebCodecs decoding,
 * real PixiJS compositing, pixels compared with the reference PNGs of `test/golden/references/`.
 *
 * WebGL runs on SwiftShader (software) on every machine so references match across macOS,
 * Windows and Linux GPUs. `SE_UPDATE_GOLDEN=1` rewrites the references instead of comparing.
 * Needs Chromium once: `pnpm exec playwright install chromium`.
 */
export default defineProject({
  test: {
    name: 'engine-browser',
    include: ['test/**/*.browser.test.ts'],
    testTimeout: 60_000,
    provide: { updateGolden: process.env.SE_UPDATE_GOLDEN === '1' },
    browser: {
      enabled: true,
      headless: true,
      screenshotFailures: false,
      provider: playwright({
        launchOptions: {
          args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
        },
      }),
      instances: [{ browser: 'chromium' }],
    },
  },
});
