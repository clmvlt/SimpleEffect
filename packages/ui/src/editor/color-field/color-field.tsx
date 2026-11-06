import { useState } from 'react';

import { cn } from '../../lib/cn';
import { textFieldClasses } from '../../primitives/input';
import { Popover, PopoverContent, PopoverTrigger } from '../../primitives/popover';
import { ColorPickerPanel, type ColorPickerLabels } from './color-picker-panel';
import { colorFromHex, hexFromColor, normalizeHex, type PickerColor } from './color-model';

/** Default (English) names of the picker controls. */
export const DEFAULT_COLOR_FIELD_LABELS: ColorPickerLabels = {
  hex: 'Hex',
  lightness: 'Lightness',
  chroma: 'Chroma',
  hue: 'Hue',
  eyeDropper: 'Pick a colour on screen',
  recent: 'Recent colours',
};

/** Props of {@link ColorField}. */
export interface ColorFieldProps {
  /** The colour as `#rrggbb`. */
  value: string;
  /** Live updates while a slider or field is dragged. */
  onChange: (hex: string) => void;
  /** End of a gesture (slider released, field accepted, swatch or eyedropper pick): one undo step. */
  onCommit?: ((hex: string) => void) | undefined;
  /** Accessible name of the swatch button, e.g. « Couleur du texte ». */
  label: string;
  /** Recently used colours (`#rrggbb`), offered as one-click swatches. */
  recentColors?: readonly string[] | undefined;
  /** Names of the picker controls; the app passes localised ones. */
  labels?: Partial<ColorPickerLabels> | undefined;
  id?: string | undefined;
  disabled?: boolean | undefined;
  className?: string | undefined;
}

const FALLBACK_COLOR: PickerColor = { lightness: 0, chroma: 0, hue: 0 };

/**
 * A colour swatch that opens a picker: lightness, chroma and hue sliders (OKLCH, perceptually
 * even), hex and OKLCH fields, the screen eyedropper when the platform has one, and recent
 * colours. The value stays a plain `#rrggbb` string.
 */
export function ColorField({
  value,
  onChange,
  onCommit,
  label,
  recentColors = [],
  labels,
  id,
  disabled = false,
  className,
}: ColorFieldProps) {
  const [color, setColor] = useState(() => colorFromHex(value) ?? FALLBACK_COLOR);
  const [syncedValue, setSyncedValue] = useState(value);

  // Follow outside changes (undo, another selection) without overwriting the hue of a grey or the
  // precision of the colour being edited when the new value is just our own last emission.
  if (value !== syncedValue) {
    setSyncedValue(value);
    if (normalizeHex(value) !== hexFromColor(color)) {
      setColor(colorFromHex(value, color.hue) ?? color);
    }
  }

  const change = (next: PickerColor) => {
    setColor(next);
    onChange(hexFromColor(next));
  };
  const commit = (next: PickerColor) => {
    onCommit?.(hexFromColor(next));
  };
  const changeAndCommit = (next: PickerColor) => {
    change(next);
    commit(next);
  };

  return (
    <Popover>
      <PopoverTrigger
        id={id}
        disabled={disabled}
        aria-label={label}
        className={cn(
          textFieldClasses,
          'inline-flex h-control-sm w-auto items-center gap-2 px-1.5',
          className,
        )}
      >
        <span
          aria-hidden
          className="size-4 shrink-0 rounded-xs border border-border-strong"
          style={{ backgroundColor: hexFromColor(color) }}
        />
        <span className="timecode text-xs text-text-muted">{hexFromColor(color)}</span>
      </PopoverTrigger>
      <PopoverContent className="w-64" align="start">
        <ColorPickerPanel
          color={color}
          labels={{ ...DEFAULT_COLOR_FIELD_LABELS, ...labels }}
          recentColors={recentColors}
          onChange={change}
          onCommit={commit}
          onPick={changeAndCommit}
        />
      </PopoverContent>
    </Popover>
  );
}
