import { EmptyState } from '@se/ui';
import { SlidersHorizontal } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useAudioClip } from '../hooks/use-audio-clip';
import { AudioSection } from './AudioSection';

/**
 * Content of the « Inspecteur » tab: the « Audio » section when the selection holds one audio
 * clip, else an empty state.
 * TODO(prompt-15): property groups of every selection (transform, text, keyframes).
 */
export function InspectorView() {
  const { t } = useTranslation();
  const audio = useAudioClip();
  if (audio !== null) return <AudioSection editor={audio} />;
  return (
    <EmptyState
      icon={SlidersHorizontal}
      title={t('editor.inspector.empty.title')}
      description={t('editor.inspector.empty.description')}
    />
  );
}
