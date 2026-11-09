import type { CaptionAnimation, CaptionStyle, CaptionStylePatch } from '@se/protocol';
import {
  PropertyGroup,
  PropertyRow,
  ScrubInput,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
} from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { BrandColorField } from '../../../shared/brand-kit';
import { FontFamilySelect } from '../../../shared/fonts';

/** Props of {@link CaptionStyleFields}. */
export interface CaptionStyleFieldsProps {
  readonly style: CaptionStyle;
  /** Shows a change in the viewer before it is committed (drags, colour pickers). */
  readonly onPreview: (patch: CaptionStylePatch) => void;
  /** Commits a change (one undo step). */
  readonly onCommit: (patch: CaptionStylePatch) => void;
}

const ANIMATIONS: readonly CaptionAnimation[] = [
  'none',
  'wordHighlight',
  'popPerWord',
  'boxPerLine',
  'revealPerWord',
];

/** The `#rrggbb` part of a colour (the picker has no alpha). */
const opaque = (color: string) => color.slice(0, 7);
/** `hex` with the alpha of `original`. */
const withAlpha = (hex: string, original: string) => `${hex}${original.slice(7)}`;

/** A number committed on release, previewed while scrubbed. */
function NumberField({
  id,
  value,
  min,
  max,
  step = 1,
  unit,
  onPreview,
  onCommit,
}: {
  readonly id: string;
  readonly value: number;
  readonly min: number;
  readonly max: number;
  readonly step?: number;
  readonly unit?: string;
  readonly onPreview?: (value: number) => void;
  readonly onCommit: (value: number) => void;
}) {
  const [draft, setDraft] = useState<number | null>(null);
  return (
    <ScrubInput
      id={id}
      size="xs"
      className="w-full"
      value={draft ?? value}
      min={min}
      max={max}
      step={step}
      unit={unit}
      onChange={(next) => {
        setDraft(next);
        onPreview?.(next);
      }}
      onCommit={(next) => {
        setDraft(null);
        onCommit(next);
      }}
    />
  );
}

/** Every field of a caption style, grouped like the inspector's sections. */
export function CaptionStyleFields({ style, onPreview, onCommit }: CaptionStyleFieldsProps) {
  const { t } = useTranslation();
  const colour = (field: 'textColor' | 'activeColor' | 'emphasisColor', label: string) => (
    <PropertyRow label={label} htmlFor={`caption-${field}`}>
      <BrandColorField
        id={`caption-${field}`}
        label={label}
        value={opaque(style[field])}
        onChange={(hex) => {
          onPreview({ [field]: withAlpha(hex, style[field]) });
        }}
        onCommit={(hex) => {
          onCommit({ [field]: withAlpha(hex, style[field]) });
        }}
      />
    </PropertyRow>
  );
  const percent = (value: number) => Math.round(value * 1000) / 10;
  return (
    <div className="flex flex-col">
      <PropertyGroup title={t('editor.captions.style.text')}>
        <PropertyRow label={t('editor.captions.style.font')} htmlFor="caption-font">
          <FontFamilySelect
            id="caption-font"
            value={style.fontFamily}
            onChange={(fontFamily) => {
              onCommit({ fontFamily });
            }}
          />
        </PropertyRow>
        <PropertyRow label={t('editor.captions.style.size')} htmlFor="caption-size">
          <NumberField
            id="caption-size"
            value={style.fontSize}
            min={8}
            max={400}
            unit={t('editor.captions.style.pixels')}
            onPreview={(fontSize) => {
              onPreview({ fontSize });
            }}
            onCommit={(fontSize) => {
              onCommit({ fontSize });
            }}
          />
        </PropertyRow>
        <PropertyRow label={t('editor.captions.style.weight')} htmlFor="caption-weight">
          <NumberField
            id="caption-weight"
            value={style.fontWeight}
            min={100}
            max={900}
            step={100}
            onCommit={(fontWeight) => {
              onCommit({ fontWeight });
            }}
          />
        </PropertyRow>
        <PropertyRow label={t('editor.captions.style.uppercase')} htmlFor="caption-uppercase">
          <Switch
            id="caption-uppercase"
            checked={style.textCase === 'uppercase'}
            onCheckedChange={(upper) => {
              onCommit({ textCase: upper ? 'uppercase' : 'normal' });
            }}
          />
        </PropertyRow>
      </PropertyGroup>
      <PropertyGroup title={t('editor.captions.style.colors')}>
        {colour('textColor', t('editor.captions.style.textColor'))}
        {colour('activeColor', t('editor.captions.style.activeColor'))}
        {colour('emphasisColor', t('editor.captions.style.emphasisColor'))}
      </PropertyGroup>
      <CaptionEffectFields style={style} onPreview={onPreview} onCommit={onCommit} />
      <PropertyGroup title={t('editor.captions.style.layout')}>
        <PropertyRow label={t('editor.captions.style.positionY')} htmlFor="caption-y">
          <NumberField
            id="caption-y"
            value={percent(style.position.y)}
            min={0}
            max={100}
            unit="%"
            onPreview={(y) => {
              onPreview({ position: { x: style.position.x, y: y / 100 } });
            }}
            onCommit={(y) => {
              onCommit({ position: { x: style.position.x, y: y / 100 } });
            }}
          />
        </PropertyRow>
        <PropertyRow label={t('editor.captions.style.maxChars')} htmlFor="caption-chars">
          <NumberField
            id="caption-chars"
            value={style.maxCharsPerLine}
            min={4}
            max={80}
            onCommit={(maxCharsPerLine) => {
              onCommit({ maxCharsPerLine });
            }}
          />
        </PropertyRow>
        <PropertyRow label={t('editor.captions.style.maxLines')} htmlFor="caption-lines">
          <NumberField
            id="caption-lines"
            value={style.maxLines}
            min={1}
            max={4}
            onCommit={(maxLines) => {
              onCommit({ maxLines });
            }}
          />
        </PropertyRow>
      </PropertyGroup>
    </div>
  );
}

/** Animation, outline, shadow and plate. */
function CaptionEffectFields({ style, onPreview, onCommit }: CaptionStyleFieldsProps) {
  const { t } = useTranslation();
  return (
    <PropertyGroup title={t('editor.captions.style.effects')}>
      <PropertyRow label={t('editor.captions.style.animation')} htmlFor="caption-animation">
        <Select
          value={style.animation}
          onValueChange={(value) => {
            const animation = ANIMATIONS.find((candidate) => candidate === value);
            if (animation !== undefined) onCommit({ animation });
          }}
        >
          <SelectTrigger id="caption-animation" size="xs" className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {ANIMATIONS.map((animation) => (
              <SelectItem key={animation} value={animation}>
                {t(`editor.captions.animations.${animation}`)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </PropertyRow>
      <PropertyRow label={t('editor.captions.style.highlightSpoken')} htmlFor="caption-karaoke">
        <Switch
          id="caption-karaoke"
          checked={style.highlightSpoken}
          onCheckedChange={(highlightSpoken) => {
            onCommit({ highlightSpoken });
          }}
        />
      </PropertyRow>
      <PropertyRow label={t('editor.captions.style.outline')} htmlFor="caption-outline">
        <Switch
          id="caption-outline"
          checked={style.outline.enabled}
          onCheckedChange={(enabled) => {
            const width = enabled && style.outline.width === 0 ? 4 : style.outline.width;
            onCommit({ outline: { ...style.outline, enabled, width } });
          }}
        />
      </PropertyRow>
      <PropertyRow label={t('editor.captions.style.shadow')} htmlFor="caption-shadow">
        <Switch
          id="caption-shadow"
          checked={style.shadow.enabled}
          onCheckedChange={(enabled) => {
            onCommit({ shadow: { ...style.shadow, enabled } });
          }}
        />
      </PropertyRow>
      <PropertyRow label={t('editor.captions.style.background')} htmlFor="caption-background">
        <Switch
          id="caption-background"
          checked={style.background.enabled}
          onCheckedChange={(enabled) => {
            onCommit({ background: { ...style.background, enabled } });
          }}
        />
      </PropertyRow>
      <PropertyRow label={t('editor.captions.style.backgroundColor')} htmlFor="caption-plate">
        <BrandColorField
          id="caption-plate"
          label={t('editor.captions.style.backgroundColor')}
          value={opaque(style.background.color)}
          onChange={(hex) => {
            onPreview({
              background: { ...style.background, color: withAlpha(hex, style.background.color) },
            });
          }}
          onCommit={(hex) => {
            onCommit({
              background: { ...style.background, color: withAlpha(hex, style.background.color) },
            });
          }}
        />
      </PropertyRow>
    </PropertyGroup>
  );
}
