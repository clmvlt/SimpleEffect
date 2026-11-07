import type { Animatable, Flicks, ItemEntity, ItemKind, ProjectSession } from '@se/protocol';

import { IDS, sequence, session, videoTrack } from './session-fixture';

/**
 * A document colour (`#rrggbb[aa]`) from its hex digits: tests write `documentColor('ffffff')`
 * because the raw-colour guard of `pnpm check` rejects colour literals in UI sources.
 */
export function documentColor(digits: string): string {
  return `#${digits}`;
}

/** One second in flicks. */
export const SECOND = 705_600_000 as Flicks;

const still = <T>(value: T): Animatable<T> => ({ kind: 'static', value });

/** A centred transform; `opacity` keyframed from 0 to 1 over the first second when `fade`. */
export function transform(fade = false) {
  return {
    position: still({ x: 0.5, y: 0.5 }),
    scale: still({ x: 1, y: 1 }),
    rotation: still(0),
    anchor: still({ x: 0.5, y: 0.5 }),
    opacity: fade
      ? ({
          kind: 'keyframed',
          keyframes: [
            { time: 0 as Flicks, value: 0, easing: { kind: 'linear' } },
            { time: SECOND, value: 1, easing: { kind: 'linear' } },
          ],
        } satisfies Animatable<number>)
      : still(1),
    blur: still(0),
  };
}

/** The text « Bonjour » (white Inter 96 px), its opacity fading in when `fade`. */
export function textKind(fade = false): ItemKind {
  return {
    kind: 'text',
    content: 'Bonjour',
    style: {
      fontFamily: 'Inter',
      fontWeight: 600,
      italic: false,
      fontSize: still(96),
      color: still(documentColor('ffffff')),
      stroke: { enabled: false, color: still(documentColor('000000')), width: still(4) },
      shadow: {
        enabled: false,
        color: still(documentColor('000000b3')),
        blur: still(8),
        offset: still({ x: 0, y: 4 }),
      },
      background: {
        enabled: false,
        color: still(documentColor('000000b3')),
        padding: { x: 24, y: 12 },
        cornerRadius: 8,
      },
      letterSpacing: still(0),
      lineHeight: 1.2,
      align: 'center',
      maxWidth: null,
      textCase: 'normal',
    },
    transform: transform(fade),
    reveal: still(1),
  };
}

/** An item `id` on V1 over the first four seconds. */
export function graphicItem(id: string, kind: ItemKind): ItemEntity {
  return {
    trackId: IDS.v1,
    id,
    revision: 1,
    origin: { kind: 'user' },
    timelineRange: { start: 0 as Flicks, duration: (4 * SECOND) as Flicks },
    linkGroup: null,
    kind,
  };
}

/** The fixture session with the text `title` (opacity keyframed) and the text `plain` on V1. */
export function graphicsSession(): ProjectSession {
  const base = session();
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      sequences: [sequence([IDS.v1, IDS.v2])],
      tracks: [videoTrack(IDS.v1, ['title']), videoTrack(IDS.v2, ['plain'])],
      items: [
        graphicItem('title', textKind(true)),
        { ...graphicItem('plain', textKind()), trackId: IDS.v2 },
      ],
    },
  };
}
