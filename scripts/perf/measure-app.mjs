// Measures the built app over the DevTools protocol (prompt 29, docs/performance.md). Start the
// app first (docs/performance.md « Measuring the built app »), then:
//
//   node scripts/perf/measure-app.mjs --scenario 4k        [--port 9223]
//   node scripts/perf/measure-app.mjs --scenario timeline  --project "<folder>.seproj"
//
// - `4k`: a 4K/30 project with the H.264 and HEVC 10-bit files of make-media.mjs on V1 and V2:
//   the in-app measure without proxies, then with proxies, then exports in 1080p and 2160p.
// - `timeline`: opens a large project (`pnpm generate-test-project --minutes 60`), measures
//   main-thread frames while zooming and scrolling the timeline, then runs the in-app measure.
// Results are printed as JSON.
import { mkdirSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { clickText, connectPage, core, evaluate, installCoreHelper, waitFor } from './cdp.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const french = JSON.parse(
  readFileSync(join(root, 'apps', 'desktop', 'src', 'i18n', 'locales', 'fr.json'), 'utf8'),
);
const SECOND = 705_600_000;

function argument(name, fallback) {
  const index = process.argv.indexOf(`--${name}`);
  return index === -1 ? fallback : process.argv[index + 1];
}

async function openInEditor(page, path) {
  const session = await core(page, { kind: 'openProject', path });
  const id = session.snapshot.project.id;
  await evaluate(page, `location.hash = '#/project/${id}'; true`);
  await waitFor(page, `!!document.querySelector('[data-timeline-lanes]')`);
  return session;
}

/** A hidden or occluded window draws nothing (no animation frames): measures would be wrong. */
async function ensureVisible(page) {
  const visible = await evaluate(page, `document.visibilityState === 'visible'`);
  if (!visible)
    throw new Error(
      'the app window is hidden: start the app with --disable-features=CalculateNativeWinOcclusion (docs/performance.md) or keep it visible',
    );
}

/** Runs « Réglages → Performances → Mesurer les performances » and reads the table. */
async function inAppMeasure(page) {
  await ensureVisible(page);
  const settingsLabel = french.settings.open;
  await evaluate(
    page,
    `document.querySelector('button[aria-label="${settingsLabel}"]').click(); true`,
  );
  await waitFor(page, `!!document.querySelector('[role=dialog]')`);
  // Radix tabs switch on pointer down: click again until the panel shows (the dialog animates in).
  const measureButton = `[...document.querySelectorAll('[role=dialog] button')].some((button) => button.textContent.includes(${JSON.stringify(french.performance.measure)}))`;
  for (let attempt = 0; attempt < 10 && !(await evaluate(page, measureButton)); attempt += 1) {
    await clickText(page, '[role=tab]', french.settings.sections.performance);
    await new Promise((done) => setTimeout(done, 300));
  }
  await clickText(page, 'button', french.performance.measure);
  await waitFor(page, `document.querySelectorAll('[data-budget]').length > 0`, 120_000);
  const rows = await evaluate(
    page,
    `[...document.querySelectorAll('[data-budget]')].map((row) => ({
      measure: row.dataset.budget,
      status: row.dataset.status,
      value: row.children[1].textContent,
    }))`,
  );
  const details = await evaluate(
    page,
    `document.querySelector('[data-budget]').closest('section').querySelector('table + p')?.textContent ?? null`,
  );
  await page.send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape' });
  await page.send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape' });
  return { rows, details };
}

/** Records main-thread frame intervals while `drive` runs. */
async function framesDuring(page, drive) {
  await evaluate(
    page,
    `window.__perfFrames = []; window.__perfLast = null;
     window.__perfTick = requestAnimationFrame(function tick(now) {
       if (window.__perfLast !== null) window.__perfFrames.push(now - window.__perfLast);
       window.__perfLast = now;
       window.__perfTick = requestAnimationFrame(tick);
     }); true`,
  );
  await drive();
  const intervals = await evaluate(
    page,
    `cancelAnimationFrame(window.__perfTick); window.__perfFrames`,
  );
  const sorted = [...intervals].sort((a, b) => a - b);
  const p95 = sorted[Math.max(0, Math.ceil(sorted.length * 0.95) - 1)] ?? 0;
  return {
    frames: sorted.length,
    p95Ms: Math.round(p95 * 10) / 10,
    maxMs: Math.round((sorted.at(-1) ?? 0) * 10) / 10,
    over50Ms: sorted.filter((interval) => interval > 50).length,
  };
}

async function wheelTimeline(page) {
  const box = await evaluate(
    page,
    `(() => { const b = document.querySelector('[data-timeline-lanes]').getBoundingClientRect();
      return { x: b.x + b.width / 2, y: b.y + 20 }; })()`,
  );
  const wheel = (deltaX, deltaY, modifiers) =>
    page.send('Input.dispatchMouseEvent', {
      type: 'mouseWheel',
      ...box,
      deltaX,
      deltaY,
      modifiers,
    });
  const pause = () => new Promise((done) => setTimeout(done, 16));
  for (let step = 0; step < 40; step += 1) {
    await wheel(0, step < 20 ? -120 : 120, 2); // Ctrl: zoom in, then out.
    await pause();
  }
  for (let step = 0; step < 60; step += 1) {
    await wheel(step < 30 ? 400 : -400, 0, 0);
    await pause();
  }
}

async function heapMegabytes(page) {
  const { metrics } = await page.send('Performance.getMetrics');
  const used = metrics.find((metric) => metric.name === 'JSHeapUsedSize')?.value ?? 0;
  return Math.round(used / 1_048_576);
}

async function exportOnce(page, session, height, folder) {
  const width = Math.round((height * 16) / 9);
  const sequence = session.snapshot.sequences[0];
  const started = await core(page, {
    kind: 'startExport',
    projectId: session.snapshot.project.id,
    sequenceId: sequence.id,
    settings: {
      container: 'mp4',
      video: {
        codec: 'h264',
        width,
        height,
        rate: { kind: 'bitrate', bitsPerSecond: height >= 2160 ? 45e6 : 12e6 },
      },
      audio: { codec: 'aac', bitsPerSecond: 192_000, encoder: 'ffmpeg' },
    },
    path: join(folder, `export-${height}p.mp4`),
    presetName: `perf ${height}p`,
  });
  const info = await waitFor(
    page,
    `window.__perfCore.then((send) => send({ kind: 'listExports' })).then((result) => {
      const info = result.exports.find((candidate) => candidate.id === '${started.export.id}');
      return ['completed', 'failed', 'cancelled'].includes(info.state.kind) ? info : null;
    })`,
    900_000,
  );
  return {
    height,
    state: info.state.kind,
    elapsedSeconds: info.state.elapsedSeconds ?? null,
    realtimeFactor:
      info.state.kind === 'completed'
        ? Math.round((info.duration / SECOND / info.state.elapsedSeconds) * 100) / 100
        : null,
    message: info.state.message ?? null,
  };
}

/** Sets the viewer's « Lire les proxys » toggle through its button, like a user. */
async function setProxyToggle(page, enabled) {
  const label = JSON.stringify(french.editor.viewer.toolbar.proxy);
  await evaluate(
    page,
    `(() => {
      const button = document.querySelector('[aria-label=' + JSON.stringify(${label}) + ']');
      if ((button.getAttribute('aria-pressed') === 'true') !== ${enabled}) button.click();
      return true;
    })()`,
  );
}

async function scenario4k(page) {
  // A new folder per run: the previous run's project may still be open (locked) in the app.
  const folder = join(root, 'target', 'perf-projects', String(Date.now()));
  mkdirSync(folder, { recursive: true });
  const created = await core(page, {
    kind: 'createProject',
    name: 'Perf 4K',
    parentDir: folder,
    format: {
      width: 3840,
      height: 2160,
      frameRate: { numerator: 30, denominator: 1 },
      sampleRate: 48_000,
    },
  });
  const projectId = created.snapshot.project.id;
  const media = ['4k-h264.mp4', '4k-hevc-10bit.mp4'].map((name) =>
    join(root, 'target', 'perf-media', name),
  );
  await core(page, { kind: 'importMedia', projectId, paths: media });
  const { snapshot } = await core(page, { kind: 'getProjectSnapshot', projectId });
  const assetIds = snapshot.assets.map((asset) => asset.id);
  const sequenceId = created.snapshot.sequences[0].id;
  for (const assetId of [...assetIds].reverse()) {
    await core(page, {
      kind: 'placeMedia',
      projectId,
      sequenceId,
      assetIds: [assetId],
      start: 0,
      videoTrack: { kind: 'new' },
      audioTrack: { kind: 'new' },
      mode: 'overwrite',
    });
  }
  const session = await openInEditor(page, created.path);
  await setProxyToggle(page, false);
  await new Promise((done) => setTimeout(done, 5000));
  const withoutProxies = await inAppMeasure(page);
  await waitFor(
    page,
    `window.__perfCore.then((send) => send({ kind: 'getProjectSnapshot', projectId: '${projectId}' }))
      .then((s) => s.snapshot.assets.every((asset) => asset.derived.proxy?.status.kind === 'ready'))`,
    900_000,
  );
  await setProxyToggle(page, true);
  await new Promise((done) => setTimeout(done, 5000));
  const withProxies = await inAppMeasure(page);
  const exports = [];
  for (const height of [1080, 2160]) exports.push(await exportOnce(page, session, height, folder));
  return { withoutProxies, withProxies, exports, heapMegabytes: await heapMegabytes(page) };
}

async function scenarioTimeline(page, path) {
  await openInEditor(page, path);
  await new Promise((done) => setTimeout(done, 5000));
  await ensureVisible(page);
  const timeline = await framesDuring(page, () => wheelTimeline(page));
  const measure = await inAppMeasure(page);
  return { timeline, measure, heapMegabytes: await heapMegabytes(page) };
}

const page = await connectPage(Number(argument('port', '9223')));
await page.send('Performance.enable');
await installCoreHelper(page);
const scenario = argument('scenario', '4k');
const result =
  scenario === 'timeline'
    ? await scenarioTimeline(page, resolve(argument('project', '')))
    : await scenario4k(page);
console.log(JSON.stringify({ scenario, ...result }, null, 2));
page.close();
