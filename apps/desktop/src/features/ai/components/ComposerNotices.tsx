import { Button, Checkbox, Label } from '@se/ui';
import { useId } from 'react';
import { useTranslation } from 'react-i18next';

import { useTaskNumber } from '../../../shared/ai-tasks';
import { openSettings } from '../../settings';
import type { ComposerNoticeData } from '../hooks/use-composer-notices';
import type { ResumeOption } from '../hooks/use-composer-submit';
import { updateComposer } from '../store/composer-store';

/** Props of {@link ComposerNotices}. */
export interface ComposerNoticesProps {
  readonly notices: ComposerNoticeData;
}

/**
 * The lines above the composer's bottom bar: a scope taken from a small selection (with « Toute
 * la séquence »), media the agent cannot read because transcription cannot run (with
 * « Réglages »), and whether the prompt continues a task's session.
 */
export function ComposerNotices({ notices }: ComposerNoticesProps) {
  const { t } = useTranslation();
  const { smallSelection, speech, resume } = notices;
  return (
    <div className="flex flex-col gap-1.5">
      {smallSelection !== null && (
        <NoticeLine
          text={t('ai.composer.smallSelection', smallSelection)}
          action={t('ai.composer.useWholeSequence')}
          onAction={() => {
            updateComposer({ wholeSequence: true });
          }}
        />
      )}
      {speech !== null && (
        <NoticeLine
          text={t(`ai.composer.speech.${speech.readiness}`, {
            count: speech.count,
            name: speech.name,
          })}
          action={t('ai.composer.speech.openSettings')}
          onAction={() => {
            openSettings('transcription');
          }}
        />
      )}
      {resume !== null && <ResumeNotice resume={resume} />}
    </div>
  );
}

interface NoticeLineProps {
  readonly text: string;
  readonly action: string;
  readonly onAction: () => void;
}

function NoticeLine({ text, action, onAction }: NoticeLineProps) {
  return (
    <div role="status" className="flex items-center gap-2 text-warning">
      <span>{text}</span>
      <Button size="xs" variant="outline" className="shrink-0" onClick={onAction}>
        {action}
      </Button>
    </div>
  );
}

function ResumeNotice({ resume }: { readonly resume: ResumeOption }) {
  const { t } = useTranslation();
  const id = useId();
  const number = useTaskNumber(resume.task.id) ?? 0;
  return (
    <div className="flex items-center gap-2">
      <Checkbox
        id={id}
        checked={resume.checked}
        onCheckedChange={(checked) => {
          resume.setChecked(checked === true);
        }}
      />
      <Label htmlFor={id} className="text-xs font-normal">
        {t(resume.kind === 'redo' ? 'ai.composer.resume' : 'ai.composer.continue', { number })}
      </Label>
    </div>
  );
}
