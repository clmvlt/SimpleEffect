import { KeyboardShortcut, PromptComposer } from '@se/ui';
import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';

import { selectProjectId, useProjectSelector } from '../../../shared/project-store';
import { useAiStatus } from '../hooks/use-ai-status';
import { useComposerScope } from '../hooks/use-composer-scope';
import { useComposerNotices } from '../hooks/use-composer-notices';
import { useComposerSubmit } from '../hooks/use-composer-submit';
import { useScopeChips } from '../hooks/use-scope-chips';
import { aiAvailability, canSubmitTasks } from '../services/ai-indicator';
import { aiTextValues } from '../services/ai-texts';
import {
  navigateHistory,
  setComposerText,
  takeFocusRequest,
  useComposer,
} from '../store/composer-store';
import { ComposerNotices } from './ComposerNotices';
import { EffortSelect, ModelSelect } from './ModelEffortSelects';
import { ScopeMenu } from './ScopeMenu';

/**
 * The AI composer at the bottom of the « IA » panel: scope chips (automatic from the selection,
 * the in/out span or a range written in the text; adjustable in « Portée »), the prompt
 * (Ctrl/⌘+Entrée sends, ↑ recalls earlier prompts), model and effort, and notices
 * (`ComposerNotices`): a small selection limiting the scope, media the agent cannot read, and
 * whether to resume a session — the task that made the item for « Refaire avec l'IA », else the
 * task that finished in the last 30 minutes. Disabled, with the reason, while the AI cannot run
 * tasks.
 */
export function TaskComposer() {
  const { t } = useTranslation();
  const status = useAiStatus();
  const projectId = useProjectSelector(selectProjectId);
  const scope = useComposerScope();
  const { chips, remove } = useScopeChips(scope);
  const text = useComposer((state) => state.text);
  const focusRequest = useComposer((state) => state.focusRequest);
  const { submit, busy, resume } = useComposerSubmit(scope, projectId);
  const notices = useComposerNotices(scope, resume);
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
      notice={unavailable ?? (notices === null ? undefined : <ComposerNotices notices={notices} />)}
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
