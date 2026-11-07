import type { CreateProjectRequest, NewProjectDefaults } from '@se/protocol';
import { Button, DialogClose, DialogFooter, Field, Input } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { FormatFields } from '../../../shared/format';
import { useNewProjectForm } from '../hooks/use-new-project-form';

/** Props of {@link NewProjectForm}. */
export interface NewProjectFormProps {
  readonly defaults: NewProjectDefaults;
  readonly chooseFolder: (current: string) => Promise<string | null>;
  readonly onCreate: (request: CreateProjectRequest) => Promise<boolean>;
}

/** The fields and buttons of the « Nouveau projet » dialog. */
export function NewProjectForm({ defaults, chooseFolder, onCreate }: NewProjectFormProps) {
  const { t } = useTranslation();
  const form = useNewProjectForm(defaults);
  const [creating, setCreating] = useState(false);
  const { values, errors } = form;

  const submit = async () => {
    const request = form.submit();
    if (request === null) return;
    setCreating(true);
    await onCreate(request);
    setCreating(false);
  };

  return (
    <form
      noValidate
      className="flex flex-col gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <Field
        label={t('projects.newDialog.name')}
        error={errors.name && t(`projects.newDialog.errors.${errors.name}`)}
      >
        <Input
          autoFocus
          value={values.name}
          placeholder={t('projects.newDialog.namePlaceholder')}
          onChange={(event) => {
            form.setName(event.target.value);
          }}
        />
      </Field>
      <div className="flex items-end gap-2">
        <Field
          className="min-w-0 flex-1"
          label={t('projects.newDialog.location')}
          error={errors.parentDir && t(`projects.newDialog.errors.${errors.parentDir}`)}
        >
          <Input
            value={values.parentDir}
            onChange={(event) => {
              form.setParentDir(event.target.value);
            }}
          />
        </Field>
        <Button
          type="button"
          variant="secondary"
          onClick={() => {
            void chooseFolder(values.parentDir).then((folder) => {
              if (folder !== null) form.setParentDir(folder);
            });
          }}
        >
          {t('projects.newDialog.chooseLocation')}
        </Button>
      </div>
      <FormatFields
        value={values}
        onChange={form.setFormat}
        frameRates={defaults.frameRates}
        sizeError={errors.size && t(`projects.newDialog.errors.${errors.size}`)}
      />
      <DialogFooter>
        <DialogClose asChild>
          <Button type="button" variant="ghost">
            {t('projects.newDialog.cancel')}
          </Button>
        </DialogClose>
        <Button type="submit" variant="primary" disabled={creating}>
          {t('projects.newDialog.create')}
        </Button>
      </DialogFooter>
    </form>
  );
}
