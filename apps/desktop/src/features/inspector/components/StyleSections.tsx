import type { ImageFit, ItemEntity, ShapeGeometry, TextPatch } from '@se/protocol';
import {
  PropertyGroup,
  PropertyRow,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import type { Inspection } from '../hooks/use-inspection';
import { ColorRow, NumberRow, PointRow, StaticNumber } from './PropertyFields';

const IMAGE_FITS: readonly ImageFit[] = ['fit', 'fill', 'natural'];
const PIXELS = { unit: 'px', min: 0 } as const;
const AXES = ['x', 'y'] as const;

/** A switch row (outline, shadow, plate, covering the frame). */
function SwitchRow({
  label,
  checked,
  onChange,
}: {
  readonly label: string;
  readonly checked: boolean;
  readonly onChange: (checked: boolean) => void;
}) {
  return (
    <PropertyRow label={label}>
      <Switch checked={checked} aria-label={label} onCheckedChange={onChange} />
    </PropertyRow>
  );
}

function kindOf<K extends ItemEntity['kind']['kind']>(items: readonly ItemEntity[], kind: K) {
  const first = items[0]?.kind;
  return first?.kind === kind ? (first as Extract<ItemEntity['kind'], { kind: K }>) : null;
}

/** « Effets du texte »: outline, shadow and plate of selected texts. */
export function TextEffectsSection({ inspection }: { readonly inspection: Inspection }) {
  const { t } = useTranslation();
  const { editor, items } = inspection;
  const text = kindOf(items, 'text');
  if (text === null) return null;
  const { stroke, shadow, background } = text.style;
  const patch = (change: TextPatch) => {
    editor.patch((item) => (item.kind.kind === 'text' ? { kind: 'text', ...change } : null));
  };
  return (
    <PropertyGroup title={t('editor.inspector.effects.title')}>
      <SwitchRow
        label={t('editor.inspector.effects.stroke')}
        checked={stroke.enabled}
        onChange={(strokeEnabled) => {
          patch({ strokeEnabled });
        }}
      />
      {stroke.enabled && (
        <>
          <ColorRow
            editor={editor}
            property="strokeColor"
            label={t('editor.inspector.effects.strokeColor')}
          />
          <NumberRow
            editor={editor}
            property="strokeWidth"
            label={t('editor.inspector.effects.strokeWidth')}
            display={PIXELS}
          />
        </>
      )}
      <SwitchRow
        label={t('editor.inspector.effects.shadow')}
        checked={shadow.enabled}
        onChange={(shadowEnabled) => {
          patch({ shadowEnabled });
        }}
      />
      {shadow.enabled && (
        <>
          <ColorRow
            editor={editor}
            property="shadowColor"
            label={t('editor.inspector.effects.shadowColor')}
          />
          <NumberRow
            editor={editor}
            property="shadowBlur"
            label={t('editor.inspector.effects.shadowBlur')}
            display={PIXELS}
          />
          <PointRow
            editor={editor}
            property="shadowOffset"
            label={t('editor.inspector.effects.shadowOffset')}
            display={{ x: { unit: 'px' }, y: { unit: 'px' } }}
          />
        </>
      )}
      <SwitchRow
        label={t('editor.inspector.effects.background')}
        checked={background.enabled}
        onChange={(backgroundEnabled) => {
          patch({ backgroundEnabled });
        }}
      />
      {background.enabled && (
        <>
          <ColorRow
            editor={editor}
            property="backgroundColor"
            label={t('editor.inspector.effects.backgroundColor')}
          />
          <PropertyRow label={t('editor.inspector.effects.padding')}>
            <div className="grid grid-cols-2 gap-1">
              {AXES.map((axis) => (
                <StaticNumber
                  key={axis}
                  value={background.padding[axis]}
                  display={PIXELS}
                  onCommit={(value) => {
                    patch({ backgroundPadding: { ...background.padding, [axis]: value } });
                  }}
                />
              ))}
            </div>
          </PropertyRow>
          <PropertyRow label={t('editor.inspector.effects.cornerRadius')}>
            <StaticNumber
              value={background.cornerRadius}
              display={PIXELS}
              onCommit={(backgroundCornerRadius) => {
                patch({ backgroundCornerRadius });
              }}
            />
          </PropertyRow>
        </>
      )}
    </PropertyGroup>
  );
}

/** The geometry with one extent changed. */
function resized(geometry: ShapeGeometry, field: string, value: number): ShapeGeometry {
  return { ...geometry, [field]: Math.max(1, value) };
}

/** « Forme »: size, corners, fill, outline and frame covering of selected shapes. */
export function ShapeSection({ inspection }: { readonly inspection: Inspection }) {
  const { t } = useTranslation();
  const { editor, items } = inspection;
  const shape = kindOf(items, 'shape');
  if (shape === null) return null;
  const { geometry } = shape;
  const patchGeometry = (field: string, value: number) => {
    editor.patch((item) =>
      item.kind.kind === 'shape' && item.kind.geometry.kind === geometry.kind
        ? { kind: 'shape', geometry: resized(item.kind.geometry, field, value) }
        : null,
    );
  };
  const extents =
    geometry.kind === 'line'
      ? ([
          ['length', geometry.length],
          ['thickness', geometry.thickness],
        ] as const)
      : ([
          ['width', geometry.width],
          ['height', geometry.height],
        ] as const);
  return (
    <PropertyGroup title={t('editor.inspector.shape.title')}>
      {!shape.coverFrame || geometry.kind === 'line'
        ? extents.map(([field, value]) => (
            <PropertyRow key={field} label={t(`editor.inspector.shape.${field}`)}>
              <StaticNumber
                value={value}
                display={{ unit: 'px', min: 1 }}
                onCommit={(next) => {
                  patchGeometry(field, next);
                }}
              />
            </PropertyRow>
          ))
        : null}
      {geometry.kind === 'rectangle' && (
        <PropertyRow label={t('editor.inspector.shape.cornerRadius')}>
          <StaticNumber
            value={geometry.cornerRadius}
            display={PIXELS}
            onCommit={(next) => {
              patchGeometry('cornerRadius', next);
            }}
          />
        </PropertyRow>
      )}
      <ColorRow editor={editor} property="fillColor" label={t('editor.inspector.shape.fill')} />
      <SwitchRow
        label={t('editor.inspector.effects.stroke')}
        checked={shape.stroke.enabled}
        onChange={(strokeEnabled) => {
          editor.patch((item) =>
            item.kind.kind === 'shape' ? { kind: 'shape', strokeEnabled } : null,
          );
        }}
      />
      {shape.stroke.enabled && (
        <>
          <ColorRow
            editor={editor}
            property="strokeColor"
            label={t('editor.inspector.effects.strokeColor')}
          />
          <NumberRow
            editor={editor}
            property="strokeWidth"
            label={t('editor.inspector.effects.strokeWidth')}
            display={PIXELS}
          />
        </>
      )}
      {geometry.kind !== 'line' && (
        <SwitchRow
          label={t('editor.inspector.shape.coverFrame')}
          checked={shape.coverFrame}
          onChange={(coverFrame) => {
            editor.patch((item) =>
              item.kind.kind === 'shape' ? { kind: 'shape', coverFrame } : null,
            );
          }}
        />
      )}
    </PropertyGroup>
  );
}

/** « Image »: how selected images fill the frame. */
export function ImageSection({ inspection }: { readonly inspection: Inspection }) {
  const { t } = useTranslation();
  const { editor, items } = inspection;
  const image = kindOf(items, 'image');
  if (image === null) return null;
  return (
    <PropertyGroup title={t('editor.inspector.image.title')}>
      <PropertyRow label={t('editor.inspector.image.fit')} htmlFor="inspector-fit">
        <Select
          value={image.fit}
          onValueChange={(value) => {
            const fit = IMAGE_FITS.find((candidate) => candidate === value);
            if (fit !== undefined) {
              editor.patch((item) => (item.kind.kind === 'image' ? { kind: 'image', fit } : null));
            }
          }}
        >
          <SelectTrigger id="inspector-fit" size="xs" className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {IMAGE_FITS.map((fit) => (
              <SelectItem key={fit} value={fit}>
                {t(`editor.inspector.image.fits.${fit}`)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </PropertyRow>
    </PropertyGroup>
  );
}
