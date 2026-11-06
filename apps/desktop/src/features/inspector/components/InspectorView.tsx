import { EmptyState } from '@se/ui';
import { SlidersHorizontal } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

/**
 * Content of the « Inspecteur » tab, empty until items can be selected.
 * TODO(prompt-15): property groups of the selection (transform, audio, text, keyframes).
 */
export function InspectorView() {
  const { t } = useTranslation();
  return (
    <EmptyState
      icon={SlidersHorizontal}
      title={t('editor.inspector.empty.title')}
      description={t('editor.inspector.empty.description')}
    />
  );
}
