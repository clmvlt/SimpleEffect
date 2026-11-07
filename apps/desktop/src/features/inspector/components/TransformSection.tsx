import { PropertyGroup } from '@se/ui';
import { useTranslation } from 'react-i18next';

import type { Inspection } from '../hooks/use-inspection';
import { NumberRow, PointRow } from './PropertyFields';

const percent = {
  toDisplay: (value: number) => value * 100,
  fromDisplay: (value: number) => value / 100,
  unit: '%',
};

/**
 * « Transformation » and « Opacité » of visual items: position in pixels of the sequence frame,
 * scale and anchor in percent, rotation in degrees, opacity in percent and blur in pixels, each
 * with its keyframe button.
 */
export function TransformSection({ inspection }: { readonly inspection: Inspection }) {
  const { t } = useTranslation();
  const { editor, sequence } = inspection;
  const { width, height } = sequence.format;
  return (
    <>
      <PropertyGroup title={t('editor.inspector.transform.title')}>
        <PointRow
          editor={editor}
          property="position"
          label={t('editor.inspector.transform.position')}
          display={{
            x: { toDisplay: (x) => x * width, fromDisplay: (x) => x / width, unit: 'px' },
            y: { toDisplay: (y) => y * height, fromDisplay: (y) => y / height, unit: 'px' },
          }}
        />
        <PointRow
          editor={editor}
          property="scale"
          label={t('editor.inspector.transform.scale')}
          display={{ x: percent, y: percent }}
        />
        <NumberRow
          editor={editor}
          property="rotation"
          label={t('editor.inspector.transform.rotation')}
          display={{ unit: '°' }}
        />
        <PointRow
          editor={editor}
          property="anchor"
          label={t('editor.inspector.transform.anchor')}
          display={{ x: percent, y: percent }}
        />
      </PropertyGroup>
      <PropertyGroup title={t('editor.inspector.opacity.title')}>
        <NumberRow
          editor={editor}
          property="opacity"
          label={t('editor.inspector.opacity.opacity')}
          display={{ ...percent, min: 0, max: 100 }}
        />
        <NumberRow
          editor={editor}
          property="blur"
          label={t('editor.inspector.opacity.blur')}
          display={{ unit: 'px', min: 0, step: 0.5 }}
        />
      </PropertyGroup>
    </>
  );
}
