import { captionText } from '@se/engine';
import type { Asset, ItemEntity, MotionTemplate, TrackKind } from '@se/protocol';

/** Colour family of a clip (`ClipShell` tone). */
export type ItemTone = 'video' | 'audio' | 'graphic' | 'caption';

/** What a clip draws behind its label. */
export type ItemPicture = 'thumbnails' | 'waveform' | 'none';

/** How an item looks in the timeline. */
export interface ItemDisplay {
  readonly tone: ItemTone;
  readonly picture: ItemPicture;
  /** The label, or `null` when the component must use a translated generic name. */
  readonly label: string | null;
  /** The asset whose thumbnails or peaks are drawn. */
  readonly asset: Asset | null;
  /** Generic name to translate when `label` is `null`. */
  readonly fallback: 'media' | 'shape' | 'template' | 'text';
}

/** File name of a path, whatever the separator. */
export function fileName(path: string): string {
  return path.split(/[\\/]/u).at(-1) ?? path;
}

/**
 * The tone, picture and label of an item: media clips show thumbnails on video tracks and a
 * waveform on audio tracks, named after their file; texts, shapes and templates use the graphic
 * colour, captions their own, labelled by their text or template name.
 */
export function itemDisplay(
  item: ItemEntity,
  trackKind: TrackKind,
  assets: Readonly<Record<string, Asset>>,
  templates: readonly MotionTemplate[],
): ItemDisplay {
  const kind = item.kind;
  switch (kind.kind) {
    case 'mediaClip': {
      const asset = assets[kind.assetId] ?? null;
      const audio = trackKind === 'audio' || asset?.kind === 'audio';
      return {
        tone: audio ? 'audio' : 'video',
        picture: audio ? 'waveform' : 'thumbnails',
        label: asset === null ? null : fileName(asset.originalPath),
        asset,
        fallback: 'media',
      };
    }
    case 'image': {
      const asset = assets[kind.assetId] ?? null;
      const label = asset === null ? null : fileName(asset.originalPath);
      return { tone: 'video', picture: 'thumbnails', label, asset, fallback: 'media' };
    }
    case 'text':
      return graphic(kind.content.trim() || null, 'text', 'graphic');
    case 'caption':
      return graphic(captionText(kind).trim() || null, 'text', 'caption');
    case 'shape':
      return graphic(null, 'shape', 'graphic');
    case 'motionTemplate': {
      const template = templates.find((candidate) => candidate.id === kind.templateId);
      return graphic(template?.name ?? null, 'template', 'graphic');
    }
  }
}

function graphic(
  label: string | null,
  fallback: ItemDisplay['fallback'],
  tone: ItemTone,
): ItemDisplay {
  return { tone, picture: 'none', label, asset: null, fallback };
}

/** Whether an item was made by an AI task (shown with the AI badge). */
export function isAiMade(item: ItemEntity): boolean {
  return item.origin.kind === 'aiTask';
}
