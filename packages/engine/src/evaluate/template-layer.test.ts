import type { Flicks, MotionTemplate } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { item, seconds, sequence, transform, videoTrack } from '../testing/document';
import { evaluate } from './evaluate';
import { templateAssets, templateValues } from './template-layer';

const template: MotionTemplate = {
  id: 'badge',
  name: 'Badge',
  description: '',
  version: 3,
  source: 'export function render() {}',
  params: [
    {
      name: 'label',
      label: 'Texte',
      description: null,
      spec: { kind: 'string', default: 'NEW', multiline: false, maxLength: 20 },
    },
    {
      name: 'size',
      label: 'Taille',
      description: null,
      spec: { kind: 'number', default: 10, minimum: null, maximum: null, step: null },
    },
    {
      name: 'tint',
      label: 'Teinte',
      description: null,
      spec: { kind: 'color', default: '#ff0000' },
    },
    { name: 'logo', label: 'Logo', description: null, spec: { kind: 'asset' } },
  ],
  defaultDuration: seconds(3),
  origin: { kind: 'user' },
};

describe('templateValues', () => {
  it('gives defaults, item values and interpolated keyframes', () => {
    const values = templateValues(
      template,
      {
        label: { kind: 'text', value: 'PROMO' },
        size: {
          kind: 'number',
          value: {
            kind: 'keyframed',
            keyframes: [
              { time: 0 as Flicks, value: 0, easing: { kind: 'linear' } },
              { time: seconds(2), value: 100, easing: { kind: 'linear' } },
            ],
          },
        },
        logo: { kind: 'asset', value: 'asset-1' },
        // A value of another kind than its parameter falls back to the default.
        tint: { kind: 'boolean', value: true },
      },
      seconds(0.5),
    );
    expect(values).toEqual({
      label: 'PROMO',
      size: 25,
      tint: '#ff0000',
      logo: { assetId: 'asset-1' },
    });
    expect(templateAssets(values)).toEqual(['asset-1']);
    expect(templateValues(template, {}, 0 as Flicks)).toEqual({
      label: 'NEW',
      size: 10,
      tint: '#ff0000',
      logo: null,
    });
  });
});

describe('evaluate: motion templates', () => {
  it('gives a frame-sized template layer with values, time and placement', () => {
    const instance = item(seconds(1), seconds(4), {
      kind: 'motionTemplate',
      templateId: 'badge',
      params: {},
      transform: transform({ opacity: 0.5 }),
    });
    const scene = { ...sequence(1920, 1080, [videoTrack([instance])]), templates: [template] };
    const [layer] = evaluate(scene, {}, seconds(2)).layers;
    expect(layer).toMatchObject({
      kind: 'template',
      template: { id: 'badge', version: 3, source: 'export function render() {}' },
      params: { label: 'NEW', size: 10 },
      time: 1,
      duration: 4,
      box: { width: 1920, height: 1080 },
      transform: { opacity: 0.5 },
    });
  });
});
