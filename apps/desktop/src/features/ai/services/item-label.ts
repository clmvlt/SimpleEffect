import type { Asset, ItemEntity } from '@se/protocol';

/** How a prompt names an item: its own words, else a generic name to translate. */
export interface ItemLabel {
  /** Its text, caption or file name; `null` when it has none. */
  readonly label: string | null;
  /** Generic name (`editor.timeline.items.<fallback>`) when `label` is `null`. */
  readonly fallback: 'media' | 'text' | 'shape' | 'template';
}

const MAX_LABEL = 40;

function short(text: string): string | null {
  const flat = text.replace(/\s+/gu, ' ').trim();
  if (flat === '') return null;
  return flat.length > MAX_LABEL ? flat.slice(0, MAX_LABEL - 1) + '…' : flat;
}

function fileName(path: string): string {
  return path.split(/[\\/]/u).at(-1) ?? path;
}

/** The label of `item` (texts by their words, media by their file). */
export function itemLabel(item: ItemEntity, assets: Readonly<Record<string, Asset>>): ItemLabel {
  const kind = item.kind;
  switch (kind.kind) {
    case 'mediaClip':
    case 'image': {
      const asset = assets[kind.assetId];
      return {
        label: asset === undefined ? null : fileName(asset.originalPath),
        fallback: 'media',
      };
    }
    case 'text':
      return { label: short(kind.content), fallback: 'text' };
    case 'caption':
      return { label: short(kind.text), fallback: 'text' };
    case 'shape':
      return { label: null, fallback: 'shape' };
    case 'motionTemplate':
      return { label: null, fallback: 'template' };
  }
}
