import type { ClipReframePolicy, FrameRate, SequenceEntity } from '@se/protocol';
import {
  Button,
  Checkbox,
  DialogClose,
  DialogFooter,
  Field,
  Input,
  Label,
  RadioGroup,
  RadioGroupItem,
} from '@se/ui';
import { useId, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { FormatFields, isDraftSizeValid } from '../../../shared/format';
import {
  effectiveClipPolicy,
  formatChanged,
  initialFormatValues,
  withFormat,
  type SequenceFormatValues,
} from '../services/sequence-format';

const POLICIES: readonly ClipReframePolicy[] = ['fill', 'fit', 'keep'];

/** Props of {@link SequenceFormatForm}. */
export interface SequenceFormatFormProps {
  /** `format`: change the sequence in place; `version`: create a copy in another format. */
  readonly mode: 'format' | 'version';
  readonly sequence: SequenceEntity;
  /** Whether the sequence holds items (its frame rate is then fixed). */
  readonly holdsItems: boolean;
  readonly frameRates: readonly FrameRate[];
  /** Sends the change; resolves `true` when done (the dialog closes). */
  readonly onSubmit: (values: SequenceFormatValues) => Promise<boolean>;
}

/** The fields of the « Format de la séquence » and « Créer une version… » dialogs. */
export function SequenceFormatForm({
  mode,
  sequence,
  holdsItems,
  frameRates,
  onSubmit,
}: SequenceFormatFormProps) {
  const { t } = useTranslation();
  const policyLabel = useId();
  const [values, setValues] = useState(() => initialFormatValues(sequence, mode));
  const [sending, setSending] = useState(false);
  const sizeValid = isDraftSizeValid(values);
  const nameValid = mode === 'format' || values.name.trim() !== '';
  const changed = mode === 'version' || formatChanged(values, sequence.format);
  const policy = effectiveClipPolicy(values, sequence.format);
  return (
    <form
      noValidate
      className="flex flex-col gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        if (!sizeValid || !nameValid || !changed) return;
        setSending(true);
        void onSubmit({ ...values, name: values.name.trim() }).finally(() => {
          setSending(false);
        });
      }}
    >
      {mode === 'version' && (
        <Field label={t('editor.sequences.dialog.name')}>
          <Input
            value={values.name}
            onChange={(event) => {
              setValues({ ...values, name: event.target.value, nameEdited: true });
            }}
          />
        </Field>
      )}
      <FormatFields
        value={values}
        onChange={(draft) => {
          setValues(withFormat(values, draft, sequence.name));
        }}
        frameRates={frameRates}
        frameRateLocked={holdsItems ? t('editor.sequences.dialog.frameRateLocked') : undefined}
        sizeError={sizeValid ? undefined : t('editor.sequences.dialog.invalidSize')}
      />
      <div className="flex flex-col gap-2">
        <span id={policyLabel} className="text-sm font-medium text-text">
          {t('editor.sequences.dialog.clips')}
        </span>
        <RadioGroup
          aria-labelledby={policyLabel}
          value={policy}
          onValueChange={(value) => {
            setValues({ ...values, clipPolicy: value as ClipReframePolicy });
          }}
        >
          {POLICIES.map((choice) => (
            <div key={choice} className="flex items-center gap-2">
              <RadioGroupItem id={`${policyLabel}-${choice}`} value={choice} />
              <Label htmlFor={`${policyLabel}-${choice}`}>
                {t(`editor.sequences.dialog.policies.${choice}`)}
              </Label>
            </div>
          ))}
        </RadioGroup>
      </div>
      <div className="flex items-center gap-2">
        <Checkbox
          id={`${policyLabel}-texts`}
          checked={values.fitTextWidths}
          onCheckedChange={(checked) => {
            setValues({ ...values, fitTextWidths: checked === true });
          }}
        />
        <Label htmlFor={`${policyLabel}-texts`}>{t('editor.sequences.dialog.fitTexts')}</Label>
      </div>
      <DialogFooter>
        <DialogClose asChild>
          <Button type="button" variant="ghost">
            {t('editor.sequences.dialog.cancel')}
          </Button>
        </DialogClose>
        <Button
          type="submit"
          variant="primary"
          disabled={sending || !sizeValid || !nameValid || !changed}
        >
          {t(`editor.sequences.dialog.confirm.${mode}`)}
        </Button>
      </DialogFooter>
    </form>
  );
}
