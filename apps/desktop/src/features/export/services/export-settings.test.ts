import type { ExportSettings, Format } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import {
  normalizedDraft,
  outputSize,
  settingsOf,
  type DraftVideo,
  type ExportDraft,
} from './export-draft';
import {
  BUILT_IN_PRESETS,
  DEFAULT_PRESET_CHOICE,
  choiceValue,
  hasAspect,
  parseChoice,
  savedPresetOf,
} from './export-presets';
import { expandFileName, fileNameWithExtension, isValidTemplate } from './file-name';
import { estimatedBytes, videoBitrate } from './size-estimate';

const HD: Format = {
  width: 1920,
  height: 1080,
  frameRate: { numerator: 25, denominator: 1 },
  sampleRate: 48_000,
};
const format = (width: number, height: number): Format => ({ ...HD, width, height });
const H264: DraftVideo = {
  codec: 'h264',
  shortSide: 1080,
  rate: { kind: 'quality', level: 'high' },
};

describe('normalizedDraft', () => {
  it('drops the picture of sound-only containers and gives them their codec', () => {
    const draft = normalizedDraft({
      container: 'wav',
      video: H264,
      audio: { codec: 'aac', bitsPerSecond: 192_000 },
    });
    expect(draft.video).toBeNull();
    expect(draft.audio?.codec).toBe('pcm');
  });

  it('switches codecs a container refuses to its first ones', () => {
    const draft = normalizedDraft({
      container: 'webm',
      video: H264,
      audio: { codec: 'aac', bitsPerSecond: 128_000 },
    });
    expect(draft.video?.codec).toBe('vp9');
    expect(draft.audio).toEqual({ codec: 'opus', bitsPerSecond: 128_000 });
  });

  it('gives masters PCM sound and keeps silent exports silent', () => {
    const master = normalizedDraft({
      container: 'mov',
      video: { ...H264, codec: 'prores422Hq' },
      audio: { codec: 'aac', bitsPerSecond: 192_000 },
    });
    expect(master.audio?.codec).toBe('pcm');
    const silent = normalizedDraft({ container: 'mp4', video: H264, audio: null });
    expect(silent.audio).toBeNull();
  });

  it('gives sound-only containers sound and video containers a picture', () => {
    expect(normalizedDraft({ container: 'm4a', video: null, audio: null }).audio).toEqual({
      codec: 'aac',
      bitsPerSecond: 192_000,
    });
    expect(normalizedDraft({ container: 'mp4', video: null, audio: null }).video?.codec).toBe(
      'h264',
    );
  });
});

describe('outputSize', () => {
  it('keeps the sequence aspect with the chosen shorter side', () => {
    expect(outputSize(HD, 720)).toEqual({ width: 1280, height: 720 });
    expect(outputSize(format(1080, 1920), 1080)).toEqual({ width: 1080, height: 1920 });
    expect(outputSize(format(1440, 1080), 2160)).toEqual({ width: 2880, height: 2160 });
  });

  it('keeps even sides and the longer side within 7680 px', () => {
    expect(outputSize(format(1919, 1081), 'sequence')).toEqual({ width: 1920, height: 1082 });
    expect(outputSize(format(5760, 1080), 2160)).toEqual({ width: 7680, height: 1440 });
    const { width, height } = outputSize(format(1000, 333), 720);
    expect(width % 2).toBe(0);
    expect(height % 2).toBe(0);
  });
});

describe('settingsOf', () => {
  it('sends PCM without a bitrate and only AAC to ffmpeg', () => {
    const wav = settingsOf(
      { container: 'wav', video: null, audio: { codec: 'pcm', bitsPerSecond: 192_000 } },
      HD,
      'ffmpeg',
    );
    expect(wav.audio).toEqual({ codec: 'pcm', bitsPerSecond: 0, encoder: 'webCodecs' });
    const aac: ExportDraft = {
      container: 'mp4',
      video: H264,
      audio: { codec: 'aac', bitsPerSecond: 192_000 },
    };
    expect(settingsOf(aac, HD, 'ffmpeg').audio?.encoder).toBe('ffmpeg');
    const opus: ExportDraft = { ...aac, audio: { codec: 'opus', bitsPerSecond: 128_000 } };
    expect(settingsOf(opus, HD, 'ffmpeg').audio?.encoder).toBe('webCodecs');
    expect(settingsOf(aac, HD, 'webCodecs').video).toMatchObject({ width: 1920, height: 1080 });
  });
});

describe('presets', () => {
  it('ships consistent drafts', () => {
    for (const preset of BUILT_IN_PRESETS) {
      expect(normalizedDraft(preset.draft), preset.id).toEqual(preset.draft);
    }
  });

  it('round-trips menu values and falls back to the default', () => {
    for (const choice of [
      DEFAULT_PRESET_CHOICE,
      { kind: 'builtIn', id: 'master' },
      { kind: 'saved', id: 'mine:1' },
      { kind: 'custom' },
    ] as const) {
      expect(parseChoice(choiceValue(choice))).toEqual(choice);
    }
    expect(parseChoice('builtIn:gone')).toEqual(DEFAULT_PRESET_CHOICE);
  });

  it('saves the sequence size as its shorter side', () => {
    const preset = savedPresetOf(
      'id-1',
      '  Mon master  ',
      { container: 'mov', video: { ...H264, shortSide: 'sequence' }, audio: null },
      format(1080, 1920),
    );
    expect(preset).toMatchObject({ id: 'id-1', name: 'Mon master', audio: null });
    expect(preset.video?.shortSide).toBe(1080);
  });

  it('tells the frame shapes apart', () => {
    expect(hasAspect(format(1080, 1920), { width: 9, height: 16 })).toBe(true);
    expect(hasAspect(HD, { width: 9, height: 16 })).toBe(false);
    expect(hasAspect(format(1080, 1080), { width: 1, height: 1 })).toBe(true);
  });
});

describe('file names', () => {
  const values = {
    project: 'Film',
    sequence: 'Montage',
    preset: 'YouTube',
    date: new Date(2026, 10, 10, 9, 5),
    width: 1920,
    height: 1080,
  };

  it('fills the placeholders and keeps unknown ones', () => {
    expect(
      expandFileName('{project} - {sequence} {date} {time} {width}x{height} {other}', values),
    ).toBe('Film - Montage 2026-11-10 09h05 1920x1080 {other}');
    const soundOnly = { project: 'Film', sequence: 'Montage', preset: 'WAV', date: values.date };
    expect(expandFileName('{width}', soundOnly)).toBe('export');
  });

  it('replaces characters file systems refuse and trims the ends', () => {
    expect(expandFileName('{sequence}', { ...values, sequence: 'a/b:c?' })).toBe('a_b_c_');
    expect(expandFileName('  nom.  ', values)).toBe('nom');
    expect(expandFileName('...', values)).toBe('export');
  });

  it('accepts the templates the core accepts', () => {
    expect(isValidTemplate('{sequence}')).toBe(true);
    expect(isValidTemplate('   ')).toBe(false);
    expect(isValidTemplate('a<b')).toBe(false);
    expect(isValidTemplate('x'.repeat(201))).toBe(false);
    expect(isValidTemplate('\u{1F3AC}'.repeat(200))).toBe(true);
    expect(fileNameWithExtension('Montage', 'mov')).toBe('Montage.mov');
  });
});

describe('size estimate', () => {
  const settings = (video: ExportSettings['video'], audio: ExportSettings['audio']) => ({
    container: 'mp4' as const,
    video,
    audio,
  });

  it('adds the picture, the sound and the container overhead', () => {
    const fixed = settings(
      { codec: 'h264', width: 1920, height: 1080, rate: { kind: 'bitrate', bitsPerSecond: 12e6 } },
      { codec: 'aac', bitsPerSecond: 192_000, encoder: 'webCodecs' },
    );
    expect(estimatedBytes(fixed, 10, 25)).toBe(15_392_400);
    const wav = settings(null, { codec: 'pcm', bitsPerSecond: 0, encoder: 'webCodecs' });
    expect(estimatedBytes(wav, 60, 25)).toBe(11_635_200);
  });

  it('scales masters and follows the codec efficiency for quality levels', () => {
    const master = {
      codec: 'prores422Hq' as const,
      width: 1920,
      height: 1080,
      rate: { kind: 'quality' as const, level: 'veryHigh' as const },
    };
    expect(videoBitrate(master, 29.97)).toBeCloseTo(220e6);
    expect(videoBitrate({ ...master, width: 3840, height: 2160 }, 29.97)).toBeCloseTo(880e6);
    const h264 = {
      ...master,
      codec: 'h264' as const,
      rate: { kind: 'quality' as const, level: 'high' as const },
    };
    const hevc = { ...h264, codec: 'hevc' as const };
    expect(videoBitrate(hevc, 25) / videoBitrate(h264, 25)).toBeCloseTo(0.6);
    expect(videoBitrate(h264, 25)).toBeGreaterThan(4e6);
  });
});
