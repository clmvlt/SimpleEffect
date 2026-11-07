import type { AnimatableProperty, Vec2 } from '@se/protocol';
import { ColorField, KeyframeButton, PropertyRow, ScrubInput } from '@se/ui';
import { useState, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import type { PropertyValues } from '../../../shared/animation';
import type { PropertyEditor } from '../hooks/use-property-editor';
import { allSame, keepAlpha, opaquePart } from '../services/inspector-values';

/** Properties whose values are numbers. */
type NumberProperty = {
  [P in AnimatableProperty]: PropertyValues[P] extends number ? P : never;
}[AnimatableProperty];

/** Properties whose values are pairs. */
type PointProperty = {
  [P in AnimatableProperty]: PropertyValues[P] extends Vec2 ? P : never;
}[AnimatableProperty];

/** Properties whose values are colours. */
type ColorProperty = 'fillColor' | 'strokeColor' | 'shadowColor' | 'backgroundColor';

/** Display conversion of a number: the stored value ↔ the value typed and shown. */
export interface NumberDisplay {
  readonly toDisplay?: (value: number) => number;
  readonly fromDisplay?: (value: number) => number;
  readonly unit?: string;
  readonly step?: number;
  readonly min?: number;
  readonly max?: number;
}

const identity = (value: number) => value;

/** A value of a number or pair property. */
type Scalar = number | Vec2;

const readNumber = (value: Scalar) => (typeof value === 'number' ? value : 0);
const writeNumber = (_: Scalar, number: number): Scalar => number;

/** A property row with the keyframe button of `property` (diamond, previous, next). */
export function AnimatedRow({
  editor,
  property,
  label,
  htmlFor,
  children,
}: {
  readonly editor: PropertyEditor;
  readonly property: AnimatableProperty;
  readonly label: string;
  readonly htmlFor?: string;
  readonly children: ReactNode;
}) {
  const { t } = useTranslation();
  const state = editor.keyframe(property);
  return (
    <PropertyRow
      label={label}
      htmlFor={htmlFor}
      keyframe={
        <KeyframeButton
          state={state}
          onToggle={() => {
            editor.toggleKeyframe(property);
          }}
          onPrevious={editor.goToKeyframe(property, -1)}
          onNext={editor.goToKeyframe(property, 1)}
          label={t('editor.inspector.keyframes.toggle', { property: label })}
          previousLabel={t('editor.inspector.keyframes.previous')}
          nextLabel={t('editor.inspector.keyframes.next')}
        />
      }
    >
      {children}
    </PropertyRow>
  );
}

/** A number field of `property` (drag previews, release commits one step). */
export function NumberInput({
  editor,
  property,
  display = {},
  id,
  label,
  read = readNumber,
  write = writeNumber,
}: {
  readonly editor: PropertyEditor;
  readonly property: NumberProperty | PointProperty;
  readonly display?: NumberDisplay;
  readonly id?: string;
  readonly label?: string;
  /** The number of the property's value shown (a component of a pair). */
  readonly read?: (value: Scalar) => number;
  /** The property's value with the number changed. */
  readonly write?: (value: Scalar, number: number) => Scalar;
}) {
  const [draft, setDraft] = useState<number | null>(null);
  const toDisplay = display.toDisplay ?? identity;
  const fromDisplay = display.fromDisplay ?? identity;
  const values: Scalar[] = editor.values(property);
  const [first] = values;
  if (first === undefined) return null;
  const shown = toDisplay(read(first));
  const next = (number: number) => write(first, fromDisplay(number));
  return (
    <ScrubInput
      id={id}
      label={label}
      size="xs"
      className="w-full"
      value={draft ?? shown}
      unit={display.unit}
      step={display.step ?? 1}
      min={display.min}
      max={display.max}
      onChange={(value) => {
        setDraft(value);
        editor.preview(property, next(value));
      }}
      onCommit={(value) => {
        setDraft(null);
        editor.commit(property, next(value));
      }}
    />
  );
}

/** A row editing a number property. */
export function NumberRow({
  editor,
  property,
  label,
  display,
}: {
  readonly editor: PropertyEditor;
  readonly property: NumberProperty;
  readonly label: string;
  readonly display?: NumberDisplay;
}) {
  if (editor.value(property) === null) return null;
  const id = `inspector-${property}`;
  return (
    <AnimatedRow editor={editor} property={property} label={label} htmlFor={id}>
      <NumberInput editor={editor} property={property} display={display ?? {}} id={id} />
    </AnimatedRow>
  );
}

/** A row editing both numbers of a pair property (X and Y fields). */
export function PointRow({
  editor,
  property,
  label,
  display = {},
}: {
  readonly editor: PropertyEditor;
  readonly property: PointProperty;
  readonly label: string;
  readonly display?: { readonly x?: NumberDisplay; readonly y?: NumberDisplay };
}) {
  const { t } = useTranslation();
  if (editor.value(property) === null) return null;
  const axis = (key: 'x' | 'y') => (
    <NumberInput
      editor={editor}
      property={property}
      display={display[key] ?? {}}
      label={t(`editor.inspector.axis.${key}`, { property: label })}
      read={(value) => (typeof value === 'number' ? value : value[key])}
      write={(value, number) => (typeof value === 'number' ? value : { ...value, [key]: number })}
    />
  );
  return (
    <AnimatedRow editor={editor} property={property} label={label}>
      <div className="grid grid-cols-2 gap-1">
        {axis('x')}
        {axis('y')}
      </div>
    </AnimatedRow>
  );
}

/** A row editing a colour property (alpha kept). */
export function ColorRow({
  editor,
  property,
  label,
}: {
  readonly editor: PropertyEditor;
  readonly property: ColorProperty;
  readonly label: string;
}) {
  const values = editor.values(property);
  const [first] = values;
  if (first === undefined) return null;
  const id = `inspector-${property}`;
  const mixed = !allSame(values, (left, right) => left === right);
  return (
    <AnimatedRow editor={editor} property={property} label={label} htmlFor={id}>
      <ColorField
        id={id}
        label={label}
        value={opaquePart(first)}
        className={mixed ? 'opacity-70' : undefined}
        onChange={(hex) => {
          editor.preview(property, keepAlpha(hex, first));
        }}
        onCommit={(hex) => {
          editor.commit(property, keepAlpha(hex, first));
        }}
      />
    </AnimatedRow>
  );
}

/** A number field of a static (not animatable) value, committed as a patch. */
export function StaticNumber({
  id,
  value,
  display = {},
  onCommit,
}: {
  readonly id?: string;
  readonly value: number;
  readonly display?: NumberDisplay;
  readonly onCommit: (value: number) => void;
}) {
  const [draft, setDraft] = useState<number | null>(null);
  const toDisplay = display.toDisplay ?? identity;
  const fromDisplay = display.fromDisplay ?? identity;
  return (
    <ScrubInput
      id={id}
      size="xs"
      className="w-full"
      value={draft ?? toDisplay(value)}
      unit={display.unit}
      step={display.step ?? 1}
      min={display.min}
      max={display.max}
      onChange={setDraft}
      onCommit={(next) => {
        setDraft(null);
        onCommit(fromDisplay(next));
      }}
    />
  );
}
