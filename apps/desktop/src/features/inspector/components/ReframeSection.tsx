import { Button, PropertyGroup, PropertyRow, ToggleGroup, ToggleGroupItem } from '@se/ui';
import { Crop } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { startReframing } from '../../../shared/reframe';
import { AutoReframeControls } from '../../reframing';
import type { Inspection } from '../hooks/use-inspection';
import {
  PICKABLE_REFRAME_MODES,
  reframeEdit,
  type PickableReframeMode,
} from '../services/reframe-switch';
import { PointRow } from './PropertyFields';

const percent = {
  toDisplay: (value: number) => value * 100,
  fromDisplay: (value: number) => value / 100,
  unit: '%',
  min: 0,
  max: 100,
};

function isPickable(mode: string): mode is PickableReframeMode {
  return (PICKABLE_REFRAME_MODES as readonly string[]).includes(mode);
}

/**
 * « Cadrage » of clips on video tracks (ADR-0026): how the picture fits the frame (fit, fill,
 * natural size; tracked clips show their mode), the point of the picture a fill keeps (in
 * percent, animatable), « Recadrer dans le viewer » for one filling clip, and automatic
 * reframing (prompt 26: « Recadrer automatiquement », the tracking settings).
 */
export function ReframeSection({ inspection }: { readonly inspection: Inspection }) {
  const { t } = useTranslation();
  const assets = useProjectSelector((state) => state.mirror?.assets);
  const { items, editor, sequence } = inspection;
  const modes = new Set(
    items.map((item) => (item.kind.kind === 'mediaClip' ? item.kind.reframe.kind : null)),
  );
  const [only] = [...modes];
  const shared = modes.size === 1 && only !== null && only !== undefined ? only : null;
  const filling = items.every(
    (item) =>
      item.kind.kind === 'mediaClip' &&
      (item.kind.reframe.kind === 'fill' || item.kind.reframe.kind === 'tracked'),
  );
  const [single] = items;
  return (
    <PropertyGroup title={t('editor.inspector.reframe.title')}>
      <PropertyRow label={t('editor.inspector.reframe.mode')}>
        <ToggleGroup
          type="single"
          size="xs"
          variant="segmented"
          aria-label={t('editor.inspector.reframe.modeLabel')}
          value={shared ?? ''}
          onValueChange={(value) => {
            if (!isPickable(value)) return;
            editor.edit((item) =>
              item.kind.kind === 'mediaClip'
                ? reframeEdit(item.kind, value, assets?.[item.kind.assetId], sequence.format)
                : null,
            );
          }}
        >
          {PICKABLE_REFRAME_MODES.map((mode) => (
            <ToggleGroupItem key={mode} value={mode}>
              {t(`editor.inspector.reframe.modes.${mode}`)}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </PropertyRow>
      {shared === 'tracked' && (
        <p className="px-3 text-xs text-text-muted">{t('editor.inspector.reframe.tracked')}</p>
      )}
      {shared === 'custom' && (
        <p className="px-3 text-xs text-text-muted">{t('editor.inspector.reframe.custom')}</p>
      )}
      {filling && (
        <PointRow
          editor={editor}
          property="reframeFocus"
          label={t('editor.inspector.reframe.focus')}
          display={{ x: percent, y: percent }}
        />
      )}
      {filling && items.length === 1 && single !== undefined && (
        <div className="px-3 pb-2">
          <Button
            size="xs"
            variant="secondary"
            onClick={() => {
              startReframing(single.id);
            }}
          >
            <Crop className="size-3.5" />
            {t('editor.inspector.reframe.inViewer')}
          </Button>
        </div>
      )}
      <AutoReframeControls key={items.map((item) => item.id).join(',')} clips={items} />
    </PropertyGroup>
  );
}
