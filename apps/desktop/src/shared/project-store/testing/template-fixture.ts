import type { Asset, Flicks, MotionTemplate, ProjectSession } from '@se/protocol';

import { SECOND, documentColor, graphicItem, transform } from './graphics-fixture';
import { IDS, sequence, session, videoTrack } from './session-fixture';

/** The fixture template `badge`: one parameter of every kind. */
export function badgeTemplate(): MotionTemplate {
  return {
    id: 'badge',
    name: 'Badge',
    description: 'A corner badge',
    version: 2,
    source: [
      "export const meta = { name: 'Badge', defaultDuration: 3 }; // a comment",
      'export const params = {};',
      'export function render(ctx, time, p) {',
      '  ctx.fillRect(0, 0, 10, 10);',
      '}',
    ].join('\n'),
    params: [
      {
        name: 'label',
        label: 'Texte',
        description: null,
        brand: null,
        spec: { kind: 'string', default: 'NEW', multiline: false, maxLength: 40 },
      },
      {
        name: 'size',
        label: 'Taille',
        description: null,
        brand: null,
        spec: { kind: 'number', default: 60, minimum: 10, maximum: 200, step: 1 },
      },
      {
        name: 'tint',
        label: 'Teinte',
        description: null,
        brand: null,
        spec: { kind: 'color', default: documentColor('ff3366') },
      },
      {
        name: 'shown',
        label: 'Visible',
        description: null,
        brand: null,
        spec: { kind: 'boolean', default: true },
      },
      {
        name: 'corner',
        label: 'Coin',
        description: null,
        brand: null,
        spec: {
          kind: 'enum',
          default: 'topRight',
          options: [
            { value: 'topLeft', label: 'En haut à gauche' },
            { value: 'topRight', label: 'En haut à droite' },
          ],
        },
      },
      {
        name: 'font',
        label: 'Police',
        description: null,
        brand: null,
        spec: { kind: 'font', default: 'Inter' },
      },
      { name: 'logo', label: 'Logo', description: null, brand: null, spec: { kind: 'asset' } },
    ],
    defaultDuration: (3 * SECOND) as Flicks,
    origin: { kind: 'user' },
    category: 'other',
    library: null,
  };
}

/** The image asset `logo`. */
export function logoAsset(): Asset {
  return {
    id: 'logo',
    originalPath: '/media/logo.png',
    relativeHint: null,
    kind: 'image',
    mediaInfo: null,
    fingerprint: null,
    derived: { mezzanine: null, proxy: null },
    analysis: { audio: null, scenes: null, transcript: null },
    decodability: { video: 'decodable', audio: 'noStream' },
    missing: false,
  };
}

/**
 * The fixture session with the template `badge`, its instance `badge-1` on V1 (label « PROMO »,
 * the rest at defaults) over the first four seconds, and the image `logo`.
 */
export function templateSession(): ProjectSession {
  const base = session();
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      project: { ...base.snapshot.project, templates: [badgeTemplate()] },
      sequences: [sequence([IDS.v1, IDS.v2])],
      tracks: [videoTrack(IDS.v1, ['badge-1']), videoTrack(IDS.v2, [])],
      items: [
        graphicItem('badge-1', {
          kind: 'motionTemplate',
          templateId: 'badge',
          params: { label: { kind: 'text', value: 'PROMO' } },
          transform: transform(),
        }),
      ],
      assets: [logoAsset()],
    },
  };
}
