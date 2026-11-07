import type { NewProjectDefaults } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import {
  initialNewProjectForm,
  isNewProjectFormValid,
  toCreateProjectRequest,
  validateNewProjectForm,
  withRatio,
} from './new-project-form';

const DEFAULTS: NewProjectDefaults = {
  parentDir: '/home/me/Documents/SimpleEffect',
  format: {
    width: 1920,
    height: 1080,
    frameRate: { numerator: 30, denominator: 1 },
    sampleRate: 48_000,
  },
  presets: [],
  frameRates: [{ numerator: 30, denominator: 1 }],
};

describe('new project form', () => {
  it('starts from the core defaults with an empty, invalid name', () => {
    const values = initialNewProjectForm(DEFAULTS);
    expect(values.ratio).toEqual({ width: 16, height: 9 });
    expect(validateNewProjectForm(values)).toEqual({ name: 'nameRequired' });
  });

  it('reports a missing location and an invalid size', () => {
    const values = { ...initialNewProjectForm(DEFAULTS), name: 'Film', parentDir: ' ', width: 7 };
    expect(validateNewProjectForm(values)).toEqual({
      parentDir: 'locationRequired',
      size: 'invalidSize',
    });
  });

  it('switches to the default resolution of a new ratio', () => {
    const vertical = withRatio(initialNewProjectForm(DEFAULTS), { width: 9, height: 16 });
    expect([vertical.width, vertical.height]).toEqual([1080, 1920]);
  });

  it('builds a trimmed createProject request', () => {
    const values = { ...initialNewProjectForm(DEFAULTS), name: '  Film  ' };
    expect(isNewProjectFormValid(validateNewProjectForm(values))).toBe(true);
    expect(toCreateProjectRequest(values)).toEqual({
      name: 'Film',
      parentDir: DEFAULTS.parentDir,
      format: DEFAULTS.format,
    });
  });
});
