import { EmptyState, ScrollArea } from '@se/ui';
import { SlidersHorizontal } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useAudioClip } from '../hooks/use-audio-clip';
import { useInspection } from '../hooks/use-inspection';
import { AudioSection } from './AudioSection';
import { ReframeSection } from './ReframeSection';
import { ImageSection, ShapeSection, TextEffectsSection } from './StyleSections';
import { TextSection } from './TextSection';
import { TransformSection } from './TransformSection';

/**
 * Content of the « Inspecteur » tab: for selected visual items (texts, shapes, images, clips of
 * video tracks) the sections their kinds share — transform and opacity always, text, effects,
 * shape, image or reframing (clips) when every item is of that kind —; the « Audio » section for
 * one audio clip;
 * else an empty state. Values follow the playhead; animatable ones have keyframe buttons.
 */
export function InspectorView() {
  const { t } = useTranslation();
  const audio = useAudioClip();
  const inspection = useInspection();
  if (inspection !== null) {
    const { kind } = inspection;
    return (
      <ScrollArea className="h-full">
        {kind === 'text' && <TextSection inspection={inspection} />}
        {kind === 'text' && <TextEffectsSection inspection={inspection} />}
        {kind === 'shape' && <ShapeSection inspection={inspection} />}
        {kind === 'image' && <ImageSection inspection={inspection} />}
        {kind === 'mediaClip' && <ReframeSection inspection={inspection} />}
        <TransformSection inspection={inspection} />
        {audio !== null && <AudioSection editor={audio} />}
      </ScrollArea>
    );
  }
  if (audio !== null) return <AudioSection editor={audio} />;
  return (
    <EmptyState
      icon={SlidersHorizontal}
      title={t('editor.inspector.empty.title')}
      description={t('editor.inspector.empty.description')}
    />
  );
}
