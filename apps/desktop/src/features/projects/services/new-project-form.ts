import type { CreateProjectRequest, FrameRate, NewProjectDefaults } from '@se/protocol';
import type { AspectRatio } from '@se/ui';

import { aspectRatioOf } from './format-label';
import { defaultResolutionFor, isValidResolution } from './resolutions';

/** What the « new project » dialog edits. */
export interface NewProjectFormValues {
  readonly name: string;
  readonly parentDir: string;
  readonly ratio: AspectRatio;
  readonly width: number;
  readonly height: number;
  readonly frameRate: FrameRate;
  /** Audio sample rate, kept from the defaults (not edited yet). */
  readonly sampleRate: number;
}

/** Translation keys of the form's errors, by field. */
export interface NewProjectFormErrors {
  readonly name?: 'nameRequired';
  readonly parentDir?: 'locationRequired';
  readonly size?: 'invalidSize';
}

/** The form as it opens: empty name, the core's proposed folder and format. */
export function initialNewProjectForm(defaults: NewProjectDefaults): NewProjectFormValues {
  const { format } = defaults;
  return {
    name: '',
    parentDir: defaults.parentDir,
    ratio: aspectRatioOf(format.width, format.height),
    width: format.width,
    height: format.height,
    frameRate: format.frameRate,
    sampleRate: format.sampleRate,
  };
}

/** The form with another ratio and that ratio's default resolution. */
export function withRatio(values: NewProjectFormValues, ratio: AspectRatio): NewProjectFormValues {
  return { ...values, ratio, ...defaultResolutionFor(ratio) };
}

/** What is wrong with the form; empty when it can be submitted. */
export function validateNewProjectForm(values: NewProjectFormValues): NewProjectFormErrors {
  return {
    ...(values.name.trim() === '' ? { name: 'nameRequired' } : {}),
    ...(values.parentDir.trim() === '' ? { parentDir: 'locationRequired' } : {}),
    ...(isValidResolution(values) ? {} : { size: 'invalidSize' }),
  };
}

/** Whether {@link validateNewProjectForm} found nothing. */
export function isNewProjectFormValid(errors: NewProjectFormErrors): boolean {
  return Object.keys(errors).length === 0;
}

/** The `createProject` payload of a valid form. */
export function toCreateProjectRequest(values: NewProjectFormValues): CreateProjectRequest {
  return {
    name: values.name.trim(),
    parentDir: values.parentDir.trim(),
    format: {
      width: values.width,
      height: values.height,
      frameRate: values.frameRate,
      sampleRate: values.sampleRate,
    },
  };
}
