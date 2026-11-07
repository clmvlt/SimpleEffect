import type { CreateProjectRequest, NewProjectDefaults } from '@se/protocol';
import { useState } from 'react';

import {
  initialNewProjectForm,
  isNewProjectFormValid,
  toCreateProjectRequest,
  validateNewProjectForm,
  type NewProjectFormErrors,
  type NewProjectFormValues,
} from '../services/new-project-form';

/**
 * State of the « new project » form. Errors appear once the user tried to submit, then follow
 * every change.
 */
export function useNewProjectForm(defaults: NewProjectDefaults) {
  const [values, setValues] = useState(() => initialNewProjectForm(defaults));
  const [showErrors, setShowErrors] = useState(false);
  const errors = validateNewProjectForm(values);
  const shownErrors: NewProjectFormErrors = showErrors ? errors : {};
  return {
    values,
    errors: shownErrors,
    setName: (name: string) => {
      setValues((current) => ({ ...current, name }));
    },
    setParentDir: (parentDir: string) => {
      setValues((current) => ({ ...current, parentDir }));
    },
    /** The format part of the form changed (`FormatFields`). */
    setFormat: (next: NewProjectFormValues) => {
      setValues((current) => ({ ...next, name: current.name, parentDir: current.parentDir }));
    },
    /** The request to send, or `null` (and the errors shown) when the form is invalid. */
    submit(): CreateProjectRequest | null {
      setShowErrors(true);
      return isNewProjectFormValid(errors) ? toCreateProjectRequest(values) : null;
    },
  };
}
