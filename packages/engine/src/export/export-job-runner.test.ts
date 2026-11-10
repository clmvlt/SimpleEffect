import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { seconds, sequence } from '../testing/document';
import { ExportJobRunner } from './export-job-runner';
import type { ExportWorkerJob, FromExportWorker } from './export-messages';
import { ExportStoppedError, type ExportSink } from './export-sink';

/** A sound-only export: the runner needs no canvas, PixiJS or WebCodecs for it. */
const job: ExportWorkerJob = {
  exportId: 'export-1',
  endpoint: { url: 'http://127.0.0.1:9', token: 'secret' },
  scene: { sequence: sequence(1920, 1080, []), assets: {} },
  fonts: [],
  settings: {
    container: 'm4a',
    video: null,
    audio: { codec: 'aac', bitsPerSecond: 192_000, encoder: 'ffmpeg' },
  },
  pipeline: { kind: 'aacRemux', withVideo: false },
  start: 0 as Flicks,
  end: seconds(1),
  frameRate: { numerator: 25, denominator: 1 },
};

function sinkWith(write: ExportSink['write']): ExportSink {
  return { write, frame: () => Promise.resolve(), thumbnail: () => Promise.resolve() };
}

/** Runs `job` until the runner reports an outcome, answering its sound requests. */
function run(sink: ExportSink, answer: 'sound' | 'fail') {
  const messages: FromExportWorker[] = [];
  return new Promise<FromExportWorker[]>((resolve) => {
    const runner = new ExportJobRunner(
      (message) => {
        messages.push(message);
        if (message.kind === 'needSound') {
          queueMicrotask(() => {
            runner.receive(
              answer === 'sound'
                ? {
                    kind: 'sound',
                    index: message.index,
                    channels: [new Float32Array(48_000), new Float32Array(48_000)],
                  }
                : { kind: 'soundFailed', index: message.index, message: 'the mix failed' },
            );
          });
        }
        if (message.kind === 'finished' || message.kind === 'failed') resolve(messages);
      },
      () => sink,
    );
    runner.receive({ kind: 'start', job });
  });
}

describe('ExportJobRunner', () => {
  it('asks for the sound, uploads it and reports progress then the outcome', async () => {
    let written = 0;
    const messages = await run(
      sinkWith((stream, _position, bytes) => {
        expect(stream).toBe('audio');
        written += bytes.length;
        return Promise.resolve();
      }),
      'sound',
    );
    expect(messages.map((message) => message.kind)).toEqual(['needSound', 'progress', 'finished']);
    // 16-bit stereo for one second, plus the WAV header.
    expect(written).toBeGreaterThan(48_000 * 4);
  });

  it('reports a failed mix with its reason', async () => {
    const messages = await run(
      sinkWith(() => Promise.resolve()),
      'fail',
    );
    expect(messages.at(-1)).toEqual({ kind: 'failed', message: 'the mix failed', stopped: false });
  });

  it('tells a stopped export from a failed one', async () => {
    const messages = await run(
      sinkWith(() => Promise.reject(new ExportStoppedError())),
      'sound',
    );
    expect(messages.at(-1)).toMatchObject({ kind: 'failed', stopped: true });
  });
});
