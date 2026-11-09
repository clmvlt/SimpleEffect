import type { ItemEntity, TextAlign, TextCase, TextItem, TextPatch } from '@se/protocol';
import {
  PropertyGroup,
  PropertyRow,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Textarea,
  ToggleGroup,
  ToggleGroupItem,
} from '@se/ui';
import { AlignCenter, AlignLeft, AlignRight } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import type { Inspection } from '../hooks/use-inspection';
import { FontFamilySelect, useKnownFamily } from '../../../shared/fonts';
import { NumberRow, StaticNumber } from './PropertyFields';

const WEIGHTS = ['100', '200', '300', '400', '500', '600', '700', '800', '900'] as const;
const ALIGNS: readonly TextAlign[] = ['left', 'center', 'right'];
const ALIGN_ICONS = { left: AlignLeft, center: AlignCenter, right: AlignRight } as const;

/** The text of the first item (the section shows only when every item is a text). */
function firstText(items: readonly ItemEntity[]): TextItem | null {
  const kind = items[0]?.kind;
  return kind?.kind === 'text' ? kind : null;
}

/**
 * « Texte » of selected texts: content, font (family from the catalogue, weight, italic), size,
 * colour, spacing, line height, alignment, wrapping, letter case and reveal. Animatable values
 * have keyframe buttons; the others are patches.
 */
export function TextSection({ inspection }: { readonly inspection: Inspection }) {
  const { t } = useTranslation();
  const { editor, items } = inspection;
  const text = firstText(items);
  const [content, setContent] = useState<string | null>(null);
  const known = useKnownFamily(text?.style.fontFamily ?? '');
  if (text === null) return null;
  const { style } = text;
  const patch = (change: TextPatch) => {
    editor.patch((item) => (item.kind.kind === 'text' ? { kind: 'text', ...change } : null));
  };
  return (
    <PropertyGroup title={t('editor.inspector.text.title')}>
      <div className="px-2 pb-1">
        <Textarea
          aria-label={t('editor.inspector.text.content')}
          value={content ?? text.content}
          rows={2}
          onChange={(event) => {
            setContent(event.target.value);
          }}
          onBlur={() => {
            if (content !== null && content !== text.content) patch({ content });
            setContent(null);
          }}
        />
      </div>
      <PropertyRow label={t('editor.inspector.text.font')} htmlFor="inspector-font">
        <FontFamilySelect
          id="inspector-font"
          value={style.fontFamily}
          onChange={(fontFamily) => {
            patch({ fontFamily });
          }}
        />
      </PropertyRow>
      {!known && (
        <p role="alert" className="px-2 pb-1 text-xs text-warning">
          {t('editor.inspector.text.missingFont', { family: style.fontFamily })}
        </p>
      )}
      <PropertyRow label={t('editor.inspector.text.weight')} htmlFor="inspector-weight">
        <div className="flex items-center gap-2">
          <Select
            value={String(style.fontWeight)}
            onValueChange={(value) => {
              patch({ fontWeight: Number(value) });
            }}
          >
            <SelectTrigger id="inspector-weight" size="xs" className="flex-1">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {WEIGHTS.map((weight) => (
                <SelectItem key={weight} value={weight}>
                  {t(`editor.inspector.text.weights.${weight}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Switch
            checked={style.italic}
            aria-label={t('editor.inspector.text.italic')}
            onCheckedChange={(italic) => {
              patch({ italic });
            }}
          />
        </div>
      </PropertyRow>
      <NumberRow
        editor={editor}
        property="fontSize"
        label={t('editor.inspector.text.size')}
        display={{ unit: 'px', min: 1 }}
      />
      <NumberRow
        editor={editor}
        property="letterSpacing"
        label={t('editor.inspector.text.letterSpacing')}
        display={{ unit: 'px', step: 0.5 }}
      />
      <PropertyRow label={t('editor.inspector.text.lineHeight')} htmlFor="inspector-lineHeight">
        <StaticNumber
          id="inspector-lineHeight"
          value={style.lineHeight}
          display={{ step: 0.05, min: 0.1, max: 10 }}
          onCommit={(lineHeight) => {
            patch({ lineHeight });
          }}
        />
      </PropertyRow>
      <PropertyRow label={t('editor.inspector.text.align')}>
        <ToggleGroup
          type="single"
          size="xs"
          value={style.align}
          onValueChange={(value) => {
            const align = ALIGNS.find((candidate) => candidate === value);
            if (align !== undefined) patch({ align });
          }}
        >
          {ALIGNS.map((align) => {
            const AlignIcon = ALIGN_ICONS[align];
            return (
              <ToggleGroupItem
                key={align}
                value={align}
                aria-label={t(`editor.inspector.text.aligns.${align}`)}
              >
                <AlignIcon className="size-3.5" />
              </ToggleGroupItem>
            );
          })}
        </ToggleGroup>
      </PropertyRow>
      <PropertyRow label={t('editor.inspector.text.wrap')}>
        <div className="flex items-center gap-2">
          <Switch
            checked={style.maxWidth !== null}
            aria-label={t('editor.inspector.text.wrap')}
            onCheckedChange={(wrap) => {
              patch({ maxWidth: { value: wrap ? 900 : null } });
            }}
          />
          {style.maxWidth !== null && (
            <StaticNumber
              value={style.maxWidth}
              display={{ unit: 'px', min: 10 }}
              onCommit={(width) => {
                patch({ maxWidth: { value: width } });
              }}
            />
          )}
        </div>
      </PropertyRow>
      <PropertyRow label={t('editor.inspector.text.case')}>
        <ToggleGroup
          type="single"
          size="xs"
          value={style.textCase}
          onValueChange={(value) => {
            if (value === 'normal' || value === 'uppercase')
              patch({ textCase: value satisfies TextCase });
          }}
        >
          <ToggleGroupItem value="normal">
            {t('editor.inspector.text.cases.normal')}
          </ToggleGroupItem>
          <ToggleGroupItem value="uppercase">
            {t('editor.inspector.text.cases.uppercase')}
          </ToggleGroupItem>
        </ToggleGroup>
      </PropertyRow>
      <NumberRow
        editor={editor}
        property="reveal"
        label={t('editor.inspector.text.reveal')}
        display={{
          toDisplay: (value) => value * 100,
          fromDisplay: (value) => value / 100,
          unit: '%',
          min: 0,
          max: 100,
        }}
      />
    </PropertyGroup>
  );
}
