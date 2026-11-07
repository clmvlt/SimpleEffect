import type { FontRequest } from '../evaluate/frame-graph';

/** A face of the font catalogue (`listFonts`) with the URL serving its file. */
export interface CatalogFace {
  readonly family: string;
  readonly url: string;
  readonly weightMin: number;
  readonly weightMax: number;
  readonly italic: boolean;
  /** CSS `unicode-range` of a subset, or `null` for a whole font. */
  readonly unicodeRange: string | null;
  /** CSS format (`woff2`, `truetype`, `opentype`, `collection`). */
  readonly format: string;
}

/**
 * What the frame renderer needs from fonts: which requested families the catalogue lacks, and a
 * promise when some must still load (`null` when every one is ready).
 */
export interface FontLoader {
  missing(requests: readonly FontRequest[]): string[];
  ensure(requests: readonly FontRequest[]): Promise<void> | null;
}

/** The part of a `FontFace` the registry uses. */
export interface FontFaceLike {
  readonly family: string;
  load(): Promise<unknown>;
}

/** The part of a `FontFaceSet` (`document.fonts`, `self.fonts` in Workers) the registry uses. */
export interface FontFaceSetLike<Face extends FontFaceLike> {
  add(face: Face): unknown;
  delete(face: Face): boolean;
}

/** Builds a font face (the browser's `FontFace` constructor). */
export type FontFaceFactory<Face extends FontFaceLike> = (
  family: string,
  source: string,
  descriptors: FontFaceDescriptors,
) => Face;

/** Normalised key of a family name (CSS family matching ignores case). */
function familyKey(family: string): string {
  return family.trim().toLowerCase();
}

/** A CSS `url()` source with its format hint. */
function cssSource(face: CatalogFace): string {
  return `url(${JSON.stringify(face.url)}) format(${JSON.stringify(face.format)})`;
}

/**
 * Registers the faces of the font catalogue in a font set and loads a family's faces the first
 * time a text uses it (every weight, style and subset: a family is a few files). Loads that fail
 * still count as done: the text then falls back to the system's font of that name, or to the
 * fallback fonts. Families absent from the catalogue are reported as missing (unless no catalogue
 * was given at all).
 */
export class FontRegistry<Face extends FontFaceLike = FontFace> implements FontLoader {
  private faces = new Map<string, Face[]>();
  private loads = new Map<string, 'ready' | Promise<void>>();
  private hasCatalog = false;

  constructor(
    private readonly fontSet: FontFaceSetLike<Face>,
    private readonly createFace: FontFaceFactory<Face>,
  ) {}

  /** Replaces the catalogue (faces are registered, not loaded). */
  setCatalog(catalog: readonly CatalogFace[]): void {
    for (const faces of this.faces.values()) for (const face of faces) this.fontSet.delete(face);
    this.faces = new Map();
    this.loads = new Map();
    this.hasCatalog = true;
    for (const entry of catalog) {
      const descriptors: FontFaceDescriptors = {
        weight:
          entry.weightMin === entry.weightMax
            ? String(entry.weightMin)
            : `${String(entry.weightMin)} ${String(entry.weightMax)}`,
        style: entry.italic ? 'italic' : 'normal',
        ...(entry.unicodeRange === null ? {} : { unicodeRange: entry.unicodeRange }),
      };
      const face = this.createFace(entry.family, cssSource(entry), descriptors);
      this.fontSet.add(face);
      const key = familyKey(entry.family);
      this.faces.set(key, [...(this.faces.get(key) ?? []), face]);
    }
  }

  /** Whether the catalogue has the family. */
  has(family: string): boolean {
    return this.faces.has(familyKey(family));
  }

  missing(requests: readonly FontRequest[]): string[] {
    if (!this.hasCatalog) return [];
    const names = new Set<string>();
    for (const request of requests) {
      if (!this.has(request.family)) names.add(request.family);
    }
    return [...names];
  }

  ensure(requests: readonly FontRequest[]): Promise<void> | null {
    const pending: Promise<void>[] = [];
    for (const request of requests) {
      const key = familyKey(request.family);
      const state = this.loads.get(key) ?? this.startLoading(key);
      if (state !== 'ready') pending.push(state);
    }
    return pending.length === 0 ? null : Promise.all(pending).then(() => undefined);
  }

  private startLoading(key: string): 'ready' | Promise<void> {
    const faces = this.faces.get(key);
    if (faces === undefined) {
      this.loads.set(key, 'ready');
      return 'ready';
    }
    const loading = Promise.allSettled(faces.map((face) => face.load())).then(() => {
      // A newer catalogue replaced this load's map: leave its state alone.
      if (this.loads.get(key) === loading) this.loads.set(key, 'ready');
    });
    this.loads.set(key, loading);
    return loading;
  }
}

/** A registry over the document's (or Worker's) font set and the browser's `FontFace`. */
export function createFontRegistry(): FontRegistry {
  const fontSet: FontFaceSet =
    typeof document === 'undefined'
      ? (globalThis as unknown as { fonts: FontFaceSet }).fonts
      : document.fonts;
  return new FontRegistry(
    fontSet,
    (family, source, descriptors) => new FontFace(family, source, descriptors),
  );
}
