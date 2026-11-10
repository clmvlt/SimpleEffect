import type { Flicks } from '@se/protocol';
import { Button, Field, Input, Label, RadioGroup, RadioGroupItem } from '@se/ui';
import { useId } from 'react';
import { useTranslation } from 'react-i18next';

import { useDesktopBridge } from '../../../shared/desktop';
import { formatDuration } from '../../../shared/time';
import type { ExportForm, RangeKind } from '../hooks/use-export-form';
import { FILE_NAME_FIELDS } from '../services/file-name';

/** The exported part of the sequence: all of it, or its in/out span. */
export function RangeField({ form }: { readonly form: ExportForm }) {
  const { t } = useTranslation();
  const id = useId();
  return (
    <div className="flex flex-col gap-2">
      <span id={id} className="text-sm font-medium text-text">
        {t('export.dialog.range')}
      </span>
      <RadioGroup
        aria-labelledby={id}
        value={form.rangeKind}
        onValueChange={(value) => {
          form.setRangeKind(value as RangeKind);
        }}
        className="flex gap-4"
      >
        <div className="flex items-center gap-2">
          <RadioGroupItem id={`${id}-sequence`} value="sequence" />
          <Label htmlFor={`${id}-sequence`}>
            {t('export.dialog.wholeSequence', { duration: formatDuration(form.contentEnd) })}
          </Label>
        </div>
        <div className="flex items-center gap-2">
          <RadioGroupItem id={`${id}-inOut`} value="inOut" disabled={form.inOut === null} />
          <Label htmlFor={`${id}-inOut`}>
            {form.inOut === null
              ? t('export.dialog.noInOut')
              : t('export.dialog.inOut', {
                  duration: formatDuration((form.inOut.end - form.inOut.start) as Flicks),
                })}
          </Label>
        </div>
      </RadioGroup>
    </div>
  );
}

/** Where the file goes: the folder (native picker) and the file name template. */
export function DestinationFields({ form }: { readonly form: ExportForm }) {
  const { t } = useTranslation();
  const bridge = useDesktopBridge();
  const fields = FILE_NAME_FIELDS.map((field) => `{${field}}`).join(' ');
  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-end gap-2">
        <Field label={t('export.dialog.folder')} className="min-w-0 flex-1">
          <Input readOnly value={form.directory} title={form.directory} />
        </Field>
        <Button
          type="button"
          variant="secondary"
          onClick={() => {
            void bridge
              .chooseFolder({
                title: t('export.dialog.chooseFolder'),
                buttonLabel: t('export.dialog.chooseFolderButton'),
                ...(form.directory === '' ? {} : { defaultPath: form.directory }),
              })
              .then((folder) => {
                if (folder !== null) form.setDirectory(folder);
              });
          }}
        >
          {t('export.dialog.changeFolder')}
        </Button>
      </div>
      <Field
        label={t('export.dialog.fileName')}
        description={t('export.dialog.fileNameHint', { name: form.plan.fileName, fields })}
        {...(form.templateValid ? {} : { error: t('export.dialog.invalidTemplate') })}
      >
        <Input
          value={form.template}
          maxLength={200}
          onChange={(event) => {
            form.setTemplate(event.target.value);
          }}
        />
      </Field>
    </div>
  );
}
