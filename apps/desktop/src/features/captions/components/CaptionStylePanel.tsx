import type { CaptionStyle, CaptionStylePatch, ItemId } from '@se/protocol';
import { CAPTION_STYLE_PRESETS, presetStyle } from '@se/engine/caption-styles';
import { ScrollArea, ToggleGroup, ToggleGroupItem, cn } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { clearCaptionPreview, previewCaptionStyle } from '../../../shared/caption-preview';
import type { CaptionCommands } from '../hooks/use-caption-commands';
import { styleOfRow, styleTargets, type CaptionRow } from '../services/caption-rows';
import { setStyleScope, useCaptionsPreferences } from '../store/captions-store';
import { CaptionStyleFields } from './CaptionStyleFields';
import { presetKey } from './CaptionsToolbar';

/** Props of {@link CaptionStylePanel}. */
export interface CaptionStylePanelProps {
  readonly rows: readonly CaptionRow[];
  readonly styles: readonly CaptionStyle[];
  readonly active: number;
  readonly selected: readonly ItemId[];
  readonly commands: CaptionCommands;
}

/**
 * The style of the captions: « Appliquer à » the selection (a style shared with other captions
 * is copied for it) or the whole track; predefined styles shown in the viewer while hovered and
 * applied on click; then every field of the style.
 */
export function CaptionStylePanel({
  rows,
  styles,
  active,
  selected,
  commands,
}: CaptionStylePanelProps) {
  const { t } = useTranslation();
  const { scope } = useCaptionsPreferences();
  const targets = styleTargets(rows, scope, selected, active);
  const first = rows.find((row) => row.itemId === targets[0]);
  const style = styleOfRow(first, styles);
  const preview = (patch: CaptionStylePatch) => {
    if (style === undefined) return;
    previewCaptionStyle(targets, { ...style, ...patch, preset: style.preset });
  };
  const commit = (patch: CaptionStylePatch) => {
    void commands.updateStyle(targets, patch).finally(clearCaptionPreview);
  };
  return (
    <ScrollArea className="min-h-0 flex-1">
      <div className="flex flex-col gap-3 p-2">
        <ToggleGroup
          type="single"
          variant="segmented"
          size="xs"
          value={scope}
          aria-label={t('editor.captions.scope.label')}
          className="self-start"
          onValueChange={(value) => {
            if (value === 'selection' || value === 'track') setStyleScope(value);
          }}
        >
          <ToggleGroupItem value="selection">
            {t('editor.captions.scope.selection')}
          </ToggleGroupItem>
          <ToggleGroupItem value="track">{t('editor.captions.scope.track')}</ToggleGroupItem>
        </ToggleGroup>
        <p className="text-xs text-text-muted">
          {t('editor.captions.scope.count', { count: targets.length })}
        </p>
        <div
          className="grid grid-cols-2 gap-1.5"
          role="group"
          aria-label={t('editor.captions.presetsLabel')}
        >
          {CAPTION_STYLE_PRESETS.map((preset) => (
            <button
              key={preset.id}
              type="button"
              data-caption-preset={preset.id}
              className={cn(
                'rounded-sm border border-border bg-surface px-2 py-2 text-left text-sm hover:border-accent',
                style?.preset === preset.id && 'border-accent',
              )}
              disabled={commands.busy || targets.length === 0}
              onPointerEnter={() => {
                previewCaptionStyle(targets, presetStyle(preset.id, 'preview'));
              }}
              onPointerLeave={clearCaptionPreview}
              onFocus={() => {
                previewCaptionStyle(targets, presetStyle(preset.id, 'preview'));
              }}
              onBlur={clearCaptionPreview}
              onClick={() => {
                void commands.applyPreset(targets, preset.id).finally(clearCaptionPreview);
              }}
            >
              {t(presetKey(preset.id))}
            </button>
          ))}
        </div>
        {style !== undefined && (
          <CaptionStyleFields style={style} onPreview={preview} onCommit={commit} />
        )}
      </div>
    </ScrollArea>
  );
}
