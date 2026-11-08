import { useTranslation } from 'react-i18next';

import { useAiStatus } from '../hooks/use-ai-status';
import { aiAvailability } from '../services/ai-indicator';
import { aiTextValues } from '../services/ai-texts';
import { TaskComposer } from './TaskComposer';
import { TaskList } from './TaskList';

/**
 * Content of the « IA » tab: the project's tasks above, the composer below. When the provider
 * cannot run tasks (not installed, logged out), a notice says why and how to fix it; the tasks
 * stay listed.
 */
export function AiView() {
  const { t } = useTranslation();
  const status = useAiStatus();
  const availability = aiAvailability(status);
  const values = aiTextValues(status, t('statusBar.ai.fallbackName'));
  return (
    <div className="flex h-full min-h-0 flex-col">
      {availability !== 'ready' && (
        <div role="status" className="border-b border-border px-3 py-2 text-xs">
          <p className="font-medium text-text">
            {t(`editor.ai.state.${availability}.title`, values)}
          </p>
          <p className="text-text-muted">
            {t(`editor.ai.state.${availability}.description`, values)}
          </p>
        </div>
      )}
      <TaskList />
      <TaskComposer />
    </div>
  );
}
