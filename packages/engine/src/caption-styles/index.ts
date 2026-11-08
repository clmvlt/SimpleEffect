/**
 * Predefined caption styles (prompt 23): one JSON file per preset, shared with the core
 * (`se_domain::caption_preset` embeds the same files), so the style panel, generation and agents
 * see the same values. Ids are stable (documents keep them in `CaptionStyle.preset`, the UI
 * translates `editor.captions.presets.<id>`): add presets, never rename one.
 */
import type { CaptionAnimation, CaptionStyle, CaptionStyleId, TextCase } from '@se/protocol';

import box from './box.json';
import karaoke from './karaoke.json';
import minimal from './minimal.json';
import tiktokBold from './tiktok-bold.json';

export { captionLines, lineChars, type LineRange } from './caption-lines';

/** Ids of the predefined styles, in the order the panel lists them. */
export type CaptionPresetId = 'tiktokBold' | 'minimal' | 'karaoke' | 'box';

/** A style's values without its id and origin. */
export type CaptionStyleValues = Omit<CaptionStyle, 'id' | 'preset'>;

/** A predefined caption style. */
export interface CaptionStylePreset {
  readonly id: CaptionPresetId;
  readonly style: CaptionStyleValues;
}

/** The preset used when none is chosen (the core's `DEFAULT_CAPTION_PRESET`). */
export const DEFAULT_CAPTION_PRESET: CaptionPresetId = 'minimal';

const ANIMATIONS: readonly CaptionAnimation[] = [
  'none',
  'wordHighlight',
  'popPerWord',
  'boxPerLine',
  'revealPerWord',
];
const CASES: readonly TextCase[] = ['normal', 'uppercase'];

/** The JSON shape of a preset file (enums read as strings). */
interface PresetFile {
  readonly id: string;
  readonly style: Omit<CaptionStyleValues, 'animation' | 'textCase'> & {
    readonly animation: string;
    readonly textCase: string;
  };
}

function isPresetId(id: string): id is CaptionPresetId {
  return id === 'tiktokBold' || id === 'minimal' || id === 'karaoke' || id === 'box';
}

/** A preset file with its enums checked (a broken file is a bug, caught by the tests). */
export function presetOf(file: PresetFile): CaptionStylePreset {
  const animation = ANIMATIONS.find((candidate) => candidate === file.style.animation);
  const textCase = CASES.find((candidate) => candidate === file.style.textCase);
  if (!isPresetId(file.id) || animation === undefined || textCase === undefined) {
    throw new Error(`caption style preset '${file.id}' is malformed`);
  }
  return { id: file.id, style: { ...file.style, animation, textCase } };
}

/** Every predefined style, in panel order. */
export const CAPTION_STYLE_PRESETS: readonly CaptionStylePreset[] = [
  tiktokBold,
  minimal,
  karaoke,
  box,
].map(presetOf);

/** The default preset (its file is embedded: a missing one is a bug, caught at load). */
const DEFAULT_PRESET: CaptionStylePreset = (() => {
  const found = CAPTION_STYLE_PRESETS.find((preset) => preset.id === DEFAULT_CAPTION_PRESET);
  if (found === undefined) throw new Error('the default caption style preset is missing');
  return found;
})();

/** The predefined style `id` (the default one for unknown ids). */
export function captionPreset(id: string): CaptionStylePreset {
  return CAPTION_STYLE_PRESETS.find((preset) => preset.id === id) ?? DEFAULT_PRESET;
}

/** A full style from preset `id` with the given style id (previews, fallbacks). */
export function presetStyle(id: CaptionPresetId, styleId: CaptionStyleId): CaptionStyle {
  return { id: styleId, preset: id, ...captionPreset(id).style };
}
