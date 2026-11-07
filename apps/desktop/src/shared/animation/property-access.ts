import type {
  Animatable,
  AnimatableProperty,
  Color,
  ItemKind,
  PropertyAnimation,
  Vec2,
} from '@se/protocol';
import {
  interpolateColor,
  interpolateNumber,
  interpolateVec2,
  type Interpolator,
} from '@se/engine/animation';

/** The value type of each animatable property (mirror of `PropertyAnimation`'s variants). */
export interface PropertyValues {
  readonly position: Vec2;
  readonly scale: Vec2;
  readonly rotation: number;
  readonly anchor: Vec2;
  readonly opacity: number;
  readonly blur: number;
  readonly volumeDb: number;
  readonly fontSize: number;
  readonly fillColor: Color;
  readonly letterSpacing: number;
  readonly reveal: number;
  readonly strokeColor: Color;
  readonly strokeWidth: number;
  readonly shadowColor: Color;
  readonly shadowBlur: number;
  readonly shadowOffset: Vec2;
  readonly backgroundColor: Color;
}

type ProtocolValues = {
  [A in PropertyAnimation as A['property']]: A['value'] extends Animatable<infer T> ? T : never;
};
type Assert<T extends true> = T;
/** Compile-time check: {@link PropertyValues} names every property with the protocol's type. */
export type PropertyValuesMatchProtocol = Assert<
  [PropertyValues, ProtocolValues] extends [ProtocolValues, PropertyValues] ? true : false
>;

/** Every animatable property, in inspector order (`AnimatableProperty::ALL` in Rust). */
export const ANIMATABLE_PROPERTIES: readonly AnimatableProperty[] = [
  'position',
  'scale',
  'rotation',
  'anchor',
  'opacity',
  'blur',
  'volumeDb',
  'fontSize',
  'fillColor',
  'letterSpacing',
  'reveal',
  'strokeColor',
  'strokeWidth',
  'shadowColor',
  'shadowBlur',
  'shadowOffset',
  'backgroundColor',
];

type AnyAnimatable = Animatable<number> | Animatable<Vec2> | Animatable<Color>;

/** The field of `property` on `kind`, or `null` when the kind has none (mirror of Rust). */
function slot(kind: ItemKind, property: AnimatableProperty): AnyAnimatable | null {
  const transform = 'transform' in kind ? kind.transform : null;
  const text = kind.kind === 'text' ? kind : null;
  const shape = kind.kind === 'shape' ? kind : null;
  switch (property) {
    case 'position':
    case 'scale':
    case 'rotation':
    case 'anchor':
    case 'opacity':
    case 'blur':
      return transform?.[property] ?? null;
    case 'volumeDb':
      return kind.kind === 'mediaClip' ? kind.volumeDb : null;
    case 'fontSize':
      return text?.style.fontSize ?? null;
    case 'letterSpacing':
      return text?.style.letterSpacing ?? null;
    case 'reveal':
      return text?.reveal ?? null;
    case 'shadowColor':
      return text?.style.shadow.color ?? null;
    case 'shadowBlur':
      return text?.style.shadow.blur ?? null;
    case 'shadowOffset':
      return text?.style.shadow.offset ?? null;
    case 'backgroundColor':
      return text?.style.background.color ?? null;
    case 'fillColor':
      return text?.style.color ?? shape?.fill ?? null;
    case 'strokeColor':
      return text?.style.stroke.color ?? shape?.stroke.color ?? null;
    case 'strokeWidth':
      return text?.style.stroke.width ?? shape?.stroke.width ?? null;
  }
}

/** The animatable `property` of an item of `kind`, or `null` when the kind has none. */
export function propertyOf<P extends AnimatableProperty>(
  kind: ItemKind,
  property: P,
): Animatable<PropertyValues[P]> | null {
  // `slot` returns the field named by `property`, whose values have type `PropertyValues[P]`.
  return slot(kind, property) as Animatable<PropertyValues[P]> | null;
}

/** Whether items of `kind` have `property`. */
export function hasProperty(kind: ItemKind, property: AnimatableProperty): boolean {
  return slot(kind, property) !== null;
}

/** The `PropertyAnimation` setting `property` to `value`. */
export function animationOf<P extends AnimatableProperty>(
  property: P,
  value: Animatable<PropertyValues[P]>,
): PropertyAnimation {
  // `PropertyValues` matches the protocol's variants (checked above).
  return { property, value } as PropertyAnimation;
}

/** How values of `property` blend between keyframes (the engine's interpolators). */
export function interpolatorOf<P extends AnimatableProperty>(
  property: P,
): Interpolator<PropertyValues[P]> {
  const interpolator = (() => {
    switch (property) {
      case 'position':
      case 'scale':
      case 'anchor':
      case 'shadowOffset':
        return interpolateVec2;
      case 'fillColor':
      case 'strokeColor':
      case 'shadowColor':
      case 'backgroundColor':
        return interpolateColor;
      default:
        return interpolateNumber;
    }
  })();
  // The switch picks the interpolator of the property's value type.
  return interpolator as unknown as Interpolator<PropertyValues[P]>;
}

/** `kind` with `animation` applied (a copy; `kind` is returned when it lacks the property). */
export function withAnimation(kind: ItemKind, animation: PropertyAnimation): ItemKind {
  if (!hasProperty(kind, animation.property)) return kind;
  const { property, value } = animation;
  const transformProperty =
    property === 'position' ||
    property === 'scale' ||
    property === 'rotation' ||
    property === 'anchor' ||
    property === 'opacity' ||
    property === 'blur';
  if (transformProperty && 'transform' in kind) {
    return { ...kind, transform: { ...kind.transform, [property]: value } };
  }
  if (kind.kind === 'mediaClip' && property === 'volumeDb')
    return { ...kind, volumeDb: animation.value };
  if (kind.kind === 'shape') {
    if (property === 'fillColor') return { ...kind, fill: animation.value };
    const field = property === 'strokeColor' ? 'color' : 'width';
    return { ...kind, stroke: { ...kind.stroke, [field]: value } };
  }
  if (kind.kind !== 'text') return kind;
  return withTextAnimation(kind, animation);
}

function withTextAnimation(
  text: Extract<ItemKind, { kind: 'text' }>,
  { property, value }: PropertyAnimation,
): ItemKind {
  const style = text.style;
  switch (property) {
    case 'reveal':
      return { ...text, reveal: value };
    case 'fontSize':
    case 'letterSpacing':
      return { ...text, style: { ...style, [property]: value } };
    case 'fillColor':
      return { ...text, style: { ...style, color: value } };
    case 'strokeColor':
    case 'strokeWidth': {
      const field = property === 'strokeColor' ? 'color' : 'width';
      return { ...text, style: { ...style, stroke: { ...style.stroke, [field]: value } } };
    }
    case 'shadowColor':
    case 'shadowBlur':
    case 'shadowOffset': {
      const field =
        property === 'shadowColor' ? 'color' : property === 'shadowBlur' ? 'blur' : 'offset';
      return { ...text, style: { ...style, shadow: { ...style.shadow, [field]: value } } };
    }
    case 'backgroundColor':
      return {
        ...text,
        style: { ...style, background: { ...style.background, color: value } },
      };
    default:
      return text;
  }
}
