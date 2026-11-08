import { EmptyState } from '@se/ui';
import { Sparkles } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useAiStatus } from '../hooks/use-ai-status';
import { aiAvailability } from '../services/ai-indicator';
import { aiTextValues } from '../services/ai-texts';
import { AgentTestPanel } from './dev/AgentTestPanel';

/** Props of {@link AiView}. */
export interface AiViewProps {
  /** Shows the « Tester une tâche » panel (development builds). */
  readonly devTools?: boolean | undefined;
}

/**
 * Content of the « IA » tab: what the AI provider allows (not installed, logged out, ready) and,
 * in development builds, the « Tester une tâche » panel.
 * TODO(prompt-20): prompt composer, scope chips and the list of AI tasks.
 */
export function AiView({ devTools = import.meta.env.DEV }: AiViewProps) {
  const { t } = useTranslation();
  const status = useAiStatus();
  const availability = aiAvailability(status);
  const values = aiTextValues(status, t('statusBar.ai.fallbackName'));
  return (
    <div className="flex h-full min-h-0 flex-col overflow-auto">
      <EmptyState
        icon={Sparkles}
        title={t(`editor.ai.state.${availability}.title`, values)}
        description={t(`editor.ai.state.${availability}.description`, values)}
      />
      {devTools && <AgentTestPanel />}
    </div>
  );
}
