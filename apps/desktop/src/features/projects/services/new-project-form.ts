import type { CreateProjectRequest, NewProjectDefaults } from '@se/protocol';
import type { AspectRatio } from '@se/ui';

import {
  draftFromFormat,
  draftToFormat,
  isDraftSizeValid,
  withDraftRatio,
  type FormatDraft,
} from '../../../shared/format';

/** What the « new project » dialog edits: a name, a folder and a format. */
export interface NewProjectFormValues extends FormatDraft {
  readonly name: string;
  readonly parentDir: string;
}

/** Translation keys of the form's errors, by field. */
export interface NewProjectFormErrors {
  readonly name?: 'nameRequired';
  readonly parentDir?: 'locationRequired';
  readonly size?: 'invalidSize';
}

/** The form as it opens: empty name, the core's proposed folder and format. */
export function initialNewProjectForm(defaults: NewProjectDefaults): NewProjectFormValues {
  return { name: '', parentDir: defaults.parentDir, ...draftFromFormat(defaults.format) };
}

/** The form with another ratio and that ratio's default resolution. */
export function withRatio(values: NewProjectFormValues, ratio: AspectRatio): NewProjectFormValues {
  return withDraftRatio(values, ratio);
}

/** What is wrong with the form; empty when it can be submitted. */
export function validateNewProjectForm(values: NewProjectFormValues): NewProjectFormErrors {
  return {
    ...(values.name.trim() === '' ? { name: 'nameRequired' } : {}),
    ...(values.parentDir.trim() === '' ? { parentDir: 'locationRequired' } : {}),
    ...(isDraftSizeValid(values) ? {} : { size: 'invalidSize' }),
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
    format: draftToFormat(values),
  };
}
