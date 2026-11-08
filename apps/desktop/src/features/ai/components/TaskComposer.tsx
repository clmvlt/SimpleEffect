import { Checkbox, KeyboardShortcut, Label, PromptComposer } from '@se/ui';
import { useEffect, useId, useRef } from 'react';
import { useTranslation } from 'react-i18next';

import { useTaskNumber } from '../../../shared/ai-tasks';
import { selectProjectId, useProjectSelector } from '../../../shared/project-store';
import { useAiStatus } from '../hooks/use-ai-status';
import { useComposerScope } from '../hooks/use-composer-scope';
import { useComposerSubmit } from '../hooks/use-composer-submit';
import { useScopeChips } from '../hooks/use-scope-chips';
import { aiAvailability, canSubmitTasks } from '../services/ai-indicator';
import { aiTextValues } from '../services/ai-texts';
import {
  navigateHistory,
  setComposerText,
  takeFocusRequest,
  updateComposer,
  useComposer,
} from '../store/composer-store';
import { EffortSelect, ModelSelect } from './ModelEffortSelects';
import { ScopeMenu } from './ScopeMenu';

/**
 * The AI composer at the bottom of the « IA » panel: scope chips (automatic from the selection,
 * the in/out span or a range written in the text; adjustable in « Portée »), the prompt
 * (Ctrl/⌘+Entrée sends, ↑ recalls earlier prompts), model and effort, and — for « Refaire avec
 * l'IA » — whether to resume the session of the task that made the item. Disabled, with the
 * reason, while the AI cannot run tasks.
 */
export function TaskComposer() {
  const { t } = useTranslation();
  const ids = useId();
  const status = useAiStatus();
  const projectId = useProjectSelector(selectProjectId);
  const scope = useComposerScope();
  const { chips, remove } = useScopeChips(scope);
  const text = useComposer((state) => state.text);
  const resume = useComposer((state) => state.resume);
  const focusRequest = useComposer((state) => state.focusRequest);
  const { submit, busy, resumable } = useComposerSubmit(scope, projectId);
  const resumableNumber = useTaskNumber(resumable?.id ?? null);
  const field = useRef<HTMLTextAreaElement>(null);
  const ready = canSubmitTasks(status) && scope !== null;
  useEffect(() => {
    if (focusRequest > 0 && takeFocusRequest()) field.current?.focus();
  }, [focusRequest]);
  const availability = aiAvailability(status);
  const unavailable = canSubmitTasks(status)
    ? undefined
    : t(
        `ai.composer.unavailable.${availability}`,
        aiTextValues(status, t('statusBar.ai.fallbackName')),
      );
  const resumeNotice =
    resumable === null ? undefined : (
      <div className="flex items-center gap-2">
        <Checkbox
          id={`${ids}-resume`}
          checked={resume}
          onCheckedChange={(checked) => {
            updateComposer({ resume: checked === true });
          }}
        />
        <Label htmlFor={`${ids}-resume`} className="text-xs font-normal">
          {t('ai.composer.resume', { number: resumableNumber ?? 0 })}
        </Label>
      </div>
    );
  return (
    <PromptComposer
      className="m-2"
      label={t('ai.composer.label')}
      placeholder={t('ai.composer.placeholder')}
      submitLabel={t('ai.composer.submit')}
      removeScopeLabel={(scopeLabel) => t('ai.composer.removeScope', { scope: scopeLabel })}
      value={text}
      onValueChange={setComposerText}
      onSubmit={submit}
      scopes={chips}
      onRemoveScope={remove}
      scopeAction={scope === null ? undefined : <ScopeMenu scope={scope} disabled={!ready} />}
      onNavigateHistory={navigateHistory}
      textareaRef={field}
      notice={unavailable ?? resumeNotice}
      modelSlot={<ModelSelect capabilities={status?.capabilities ?? null} disabled={!ready} />}
      effortSlot={<EffortSelect capabilities={status?.capabilities ?? null} disabled={!ready} />}
      submitHint={
        <KeyboardShortcut
          keys={['Mod', 'Enter']}
          keyLabels={{ Enter: t('ai.composer.enterKey') }}
        />
      }
      busy={busy}
      submitDisabled={!ready}
    />
  );
}
