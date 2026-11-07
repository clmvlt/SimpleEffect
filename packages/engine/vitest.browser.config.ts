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
    // One file at a time: the real-time A/V sync test must not share the CPU with SwiftShader.
    fileParallelism: false,
    provide: { updateGolden: process.env.SE_UPDATE_GOLDEN === '1' },
    browser: {
      enabled: true,
      headless: true,
      screenshotFailures: false,
      provider: playwright({
        launchOptions: {
          args: [
            '--use-gl=angle',
            '--use-angle=swiftshader',
            '--enable-unsafe-swiftshader',
            // The A/V sync test plays sound without a user gesture.
            '--autoplay-policy=no-user-gesture-required',
          ],
        },
      }),
      instances: [{ browser: 'chromium' }],
    },
  },
});
