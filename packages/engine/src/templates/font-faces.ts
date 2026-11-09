import type { CatalogFace } from '../fonts/font-registry';
import type { TemplateFontFace } from './contract';

/**
 * Gives template Workers the faces of a font family as bytes: Workers never fetch anything (their
 * scope has no network), so the main thread reads the catalogue's files and sends them.
 * Resolves with no face for a family the catalogue lacks.
 */
export type TemplateFontSource = (family: string) => Promise<readonly TemplateFontFace[]>;

/** Reads a font file (the `/media/fonts/{id}` URL with the session token). */
export type FontBytesLoader = (url: string) => Promise<ArrayBuffer>;

/** Reads a URL with `fetch`. */
export const fetchFontBytes: FontBytesLoader = async (url) => {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`a font file answered ${String(response.status)}`);
  return response.arrayBuffer();
};

/**
 * A {@link TemplateFontSource} over the font catalogue's faces: each file is read once (bytes
 * kept), and a copy of them is sent each time (a transfer would empty the cache).
 */
export function catalogFontSource(
  catalog: readonly CatalogFace[],
  load: FontBytesLoader = fetchFontBytes,
): TemplateFontSource {
  const files = new Map<string, Promise<ArrayBuffer | null>>();
  const bytes = (url: string) => {
    let file = files.get(url);
    if (file === undefined) {
      file = load(url).catch(() => null);
      files.set(url, file);
    }
    return file;
  };
  return async (family) => {
    const wanted = family.trim().toLowerCase();
    const faces = catalog.filter((face) => face.family.trim().toLowerCase() === wanted);
    const loaded = await Promise.all(
      faces.map(async (face): Promise<TemplateFontFace | null> => {
        const data = await bytes(face.url);
        if (data === null) return null;
        return {
          family: face.family,
          weight:
            face.weightMin === face.weightMax
              ? String(face.weightMin)
              : `${String(face.weightMin)} ${String(face.weightMax)}`,
          style: face.italic ? 'italic' : 'normal',
          unicodeRange: face.unicodeRange,
          data: data.slice(0),
        };
      }),
    );
    return loaded.filter((face) => face !== null);
  };
}
