import type { CreateProjectRequest, FrameRate, NewProjectDefaults } from '@se/protocol';
import type { AspectRatio } from '@se/ui';
import { useState } from 'react';

import {
  initialNewProjectForm,
  isNewProjectFormValid,
  toCreateProjectRequest,
  validateNewProjectForm,
  withRatio,
  type NewProjectFormErrors,
} from '../services/new-project-form';
import { resolutionsFor, type Resolution } from '../services/resolutions';

/**
 * State of the « new project » form. Errors appear once the user tried to submit, then follow
 * every change.
 */
export function useNewProjectForm(defaults: NewProjectDefaults) {
  const [values, setValues] = useState(() => initialNewProjectForm(defaults));
  const [customSize, setCustomSize] = useState(false);
  const [showErrors, setShowErrors] = useState(false);
  const errors = validateNewProjectForm(values);
  const resolutions = resolutionsFor(values.ratio);
  const listed = resolutions.some(
    (resolution) => resolution.width === values.width && resolution.height === values.height,
  );
  const shownErrors: NewProjectFormErrors = showErrors ? errors : {};
  return {
    values,
    errors: shownErrors,
    resolutions,
    customSize: customSize || !listed,
    setName: (name: string) => {
      setValues((current) => ({ ...current, name }));
    },
    setParentDir: (parentDir: string) => {
      setValues((current) => ({ ...current, parentDir }));
    },
    setRatio: (ratio: AspectRatio) => {
      setCustomSize(false);
      setValues((current) => withRatio(current, ratio));
    },
    chooseResolution: (resolution: Resolution | 'custom') => {
      setCustomSize(resolution === 'custom');
      if (resolution !== 'custom') setValues((current) => ({ ...current, ...resolution }));
    },
    setSize: (size: Partial<Resolution>) => {
      setValues((current) => ({ ...current, ...size }));
    },
    setFrameRate: (frameRate: FrameRate) => {
      setValues((current) => ({ ...current, frameRate }));
    },
    /** The request to send, or `null` (and the errors shown) when the form is invalid. */
    submit(): CreateProjectRequest | null {
      setShowErrors(true);
      return isNewProjectFormValid(errors) ? toCreateProjectRequest(values) : null;
    },
  };
}
