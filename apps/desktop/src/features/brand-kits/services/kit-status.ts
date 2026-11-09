import type { BrandKit, LibraryBrandKit } from '@se/protocol';

/** Where the open project's kit stands against the library. */
export type ProjectKitStatus =
  | { readonly kind: 'none' }
  /** A copy of a library kit; `newer` is the library's version when it is more recent. */
  | {
      readonly kind: 'linked';
      readonly kit: LibraryBrandKit;
      readonly newer: LibraryBrandKit | null;
    }
  /** A kit the library does not hold (made elsewhere, or removed since). */
  | { readonly kind: 'orphan' }
  /** The library is not listed yet. */
  | { readonly kind: 'unknown' };

/** The status of `copy` (the project's kit) given the library's `kits` (`null`: not listed). */
export function projectKitStatus(
  copy: BrandKit | null,
  kits: readonly LibraryBrandKit[] | null,
): ProjectKitStatus {
  if (copy === null) return { kind: 'none' };
  if (kits === null) return { kind: 'unknown' };
  const source = copy.source;
  const kit = source === null ? undefined : kits.find((candidate) => candidate.id === source.kitId);
  if (source === null || kit === undefined) return { kind: 'orphan' };
  return { kind: 'linked', kit, newer: kit.version > source.version ? kit : null };
}
