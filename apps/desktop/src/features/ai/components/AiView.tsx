import { EmptyState } from '@se/ui';
import { Sparkles } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

/**
 * Content of the « IA » tab, empty until an AI provider is configured.
 * TODO(prompt-20): prompt composer, scope chips and the list of AI tasks.
 */
export function AiView() {
  const { t } = useTranslation();
  return (
    <EmptyState
      icon={Sparkles}
      title={t('editor.ai.empty.title')}
      description={t('editor.ai.empty.description')}
    />
  );
}
