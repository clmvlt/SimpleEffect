/**
 * Safe areas of publishing platforms (prompt 16): the parts of the frame covered by an app's
 * interface (TikTok, Instagram Reels, YouTube Shorts) and the broadcast title and action safe
 * areas of 16:9. Data only, one JSON file per platform with its source and measurement date:
 * the viewer draws them over the picture; they are never rendered into a frame or an export.
 */
import instagramReels from './instagram-reels.json';
import tiktok from './tiktok.json';
import youtube16x9 from './youtube-16x9.json';
import youtubeShorts from './youtube-shorts.json';

/** Stable ids of the platforms (UI preferences and translations use them). */
export type SafeAreaPlatformId = 'tiktok' | 'instagramReels' | 'youtubeShorts' | 'youtube16x9';

/**
 * What a zone means: `covered` by the app's interface (keep text and faces out of it), or the
 * inner `actionSafe` / `titleSafe` rectangle important action or text should stay inside.
 */
export type SafeAreaZoneKind = 'covered' | 'actionSafe' | 'titleSafe';

/** A rectangle normalised in the frame (`(0, 0)` top-left, `(1, 1)` bottom-right). */
export interface NormalisedRect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/** One zone of a platform. */
export interface SafeAreaZone {
  /** `top`, `bottom`, `left`, `right`, `action`, `title`. */
  readonly id: string;
  readonly kind: SafeAreaZoneKind;
  readonly rect: NormalisedRect;
}

/** A platform's safe areas, normalised in the frame it was measured on. */
export interface SafeAreaPlatform {
  readonly id: SafeAreaPlatformId;
  /** Brand name, shown as is. */
  readonly name: string;
  /** The aspect ratio the zones were measured for (e.g. 9:16). */
  readonly aspect: { readonly width: number; readonly height: number };
  /** Where the figures come from, in English. */
  readonly source: string;
  readonly url: string;
  /** When the figures were taken (ISO date): interfaces change, check again from time to time. */
  readonly measuredOn: string;
  readonly zones: readonly SafeAreaZone[];
}

/** The shape of one JSON file (pixels of `reference`). */
interface SafeAreaFile {
  readonly id: string;
  readonly name: string;
  readonly aspect: { readonly width: number; readonly height: number };
  readonly reference: { readonly width: number; readonly height: number };
  readonly source: string;
  readonly url: string;
  readonly measuredOn: string;
  readonly zones: readonly {
    readonly id: string;
    readonly kind: string;
    readonly rect: NormalisedRect;
  }[];
}

const PLATFORM_IDS: readonly SafeAreaPlatformId[] = [
  'tiktok',
  'instagramReels',
  'youtubeShorts',
  'youtube16x9',
];
const ZONE_KINDS: readonly SafeAreaZoneKind[] = ['covered', 'actionSafe', 'titleSafe'];

function isPlatformId(id: string): id is SafeAreaPlatformId {
  return (PLATFORM_IDS as readonly string[]).includes(id);
}

function isZoneKind(kind: string): kind is SafeAreaZoneKind {
  return (ZONE_KINDS as readonly string[]).includes(kind);
}

/** A file's zones in frame fractions; throws on an unknown id or kind (a data bug, tested). */
export function normalisePlatform(file: SafeAreaFile): SafeAreaPlatform {
  if (!isPlatformId(file.id)) throw new Error(`unknown safe-area platform '${file.id}'`);
  const { width, height } = file.reference;
  const zones = file.zones.map((zone) => {
    if (!isZoneKind(zone.kind)) {
      throw new Error(`unknown safe-area zone kind '${zone.kind}' in ${file.id}`);
    }
    return {
      id: zone.id,
      kind: zone.kind,
      rect: {
        x: zone.rect.x / width,
        y: zone.rect.y / height,
        width: zone.rect.width / width,
        height: zone.rect.height / height,
      },
    };
  });
  return {
    id: file.id,
    name: file.name,
    aspect: file.aspect,
    source: file.source,
    url: file.url,
    measuredOn: file.measuredOn,
    zones,
  };
}

/** Every platform, vertical ones first. */
export const SAFE_AREA_PLATFORMS: readonly SafeAreaPlatform[] = [
  tiktok,
  instagramReels,
  youtubeShorts,
  youtube16x9,
].map(normalisePlatform);

/** The platform with this id. */
export function safeAreaPlatform(id: SafeAreaPlatformId): SafeAreaPlatform | undefined {
  return SAFE_AREA_PLATFORMS.find((platform) => platform.id === id);
}

/**
 * Whether a frame has the aspect a platform's zones were measured for (within 1 %): drawn on
 * another aspect they are stretched and only indicative.
 */
export function matchesAspect(
  platform: SafeAreaPlatform,
  frame: { readonly width: number; readonly height: number },
): boolean {
  const expected = platform.aspect.width / platform.aspect.height;
  const actual = frame.width / frame.height;
  return Math.abs(actual / expected - 1) <= 0.01;
}
