import { Pipette } from 'lucide-react';
import { useState, type KeyboardEvent } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';
import { IconButton } from '../../primitives/icon-button';
import { Input } from '../../primitives/input';
import { ScrubInput } from '../scrub-input';
import { ColorChannelSlider } from './color-channel-slider';
import {
  CHANNEL_RANGES,
  colorFromHex,
  hexFromColor,
  type ColorChannel,
  type PickerColor,
} from './color-model';
import { isEyeDropperSupported, pickScreenColor } from './eye-dropper';

/** Accessible names of the picker controls. */
export interface ColorPickerLabels {
  hex: string;
  lightness: string;
  chroma: string;
  hue: string;
  eyeDropper: string;
  recent: string;
  /** Name of the group of named swatches (the app's brand palette). */
  palette: string;
}

/** A named colour offered as a one-click swatch (a brand kit's palette). */
export interface ColorSwatch {
  /** `#rrggbb`. */
  readonly color: string;
  /** What it is, e.g. « Accent ». */
  readonly label: string;
}

/** Props of {@link ColorPickerPanel}. */
export interface ColorPickerPanelProps {
  color: PickerColor;
  labels: ColorPickerLabels;
  recentColors: readonly string[];
  /** Named swatches shown first (empty for none). */
  palette: readonly ColorSwatch[];
  /** Live change (slider or field being dragged). */
  onChange: (color: PickerColor) => void;
  /** End of a slider or field gesture. */
  onCommit: (color: PickerColor) => void;
  /** A complete pick (hex typed, recent swatch, eyedropper): change and commit at once. */
  onPick: (color: PickerColor) => void;
}

/** One OKLCH field: the channel shown multiplied by `scale` (lightness in %), and its format. */
interface ChannelField {
  readonly channel: ColorChannel;
  readonly scale: number;
  readonly unit?: string | undefined;
  readonly format: (scaledValue: number) => string;
}

const CHANNEL_FIELDS: readonly ChannelField[] = [
  {
    channel: 'lightness',
    scale: 100,
    unit: '%',
    format: (value) => String(Math.round(value * 10) / 10),
  },
  { channel: 'chroma', scale: 1, format: (value) => value.toFixed(3) },
  { channel: 'hue', scale: 1, unit: '°', format: (value) => String(Math.round(value)) },
];

/** The content of the `ColorField` popover. */
export function ColorPickerPanel({
  color,
  labels,
  recentColors,
  palette,
  onChange,
  onCommit,
  onPick,
}: ColorPickerPanelProps) {
  const [hexDraft, setHexDraft] = useState<string | null>(null);
  const [hexInvalid, setHexInvalid] = useState(false);
  const hex = hexFromColor(color);
  const withChannel = (channel: ColorChannel, value: number): PickerColor => ({
    ...color,
    [channel]: value,
  });

  /** Applies the typed hex; returns false when it is not a colour. */
  const acceptHex = (): boolean => {
    if (hexDraft === null) {
      return true;
    }
    const picked = colorFromHex(hexDraft, color.hue);
    if (picked === null) {
      setHexInvalid(true);
      return false;
    }
    setHexDraft(null);
    setHexInvalid(false);
    if (hexFromColor(picked) !== hex) {
      onPick(picked);
    }
    return true;
  };

  const handleHexKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      acceptHex();
    }
  };

  const pickFromScreen = async () => {
    const picked = await pickScreenColor();
    const pickedColor = picked === null ? null : colorFromHex(picked, color.hue);
    if (pickedColor !== null) {
      onPick(pickedColor);
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div
        aria-hidden
        className="h-8 rounded-md border border-border"
        style={{ backgroundColor: hex }}
      />
      <div className="flex flex-col gap-2">
        {CHANNEL_FIELDS.map(({ channel }) => (
          <ColorChannelSlider
            key={channel}
            color={color}
            channel={channel}
            label={labels[channel]}
            onChange={(value) => {
              onChange(withChannel(channel, value));
            }}
            onCommit={(value) => {
              onCommit(withChannel(channel, value));
            }}
          />
        ))}
      </div>
      <div className="grid grid-cols-3 gap-1">
        {CHANNEL_FIELDS.map(({ channel, unit, format, scale }) => (
          <ScrubInput
            key={channel}
            label={labels[channel]}
            size="xs"
            value={color[channel] * scale}
            min={CHANNEL_RANGES[channel].min * scale}
            max={CHANNEL_RANGES[channel].max * scale}
            step={CHANNEL_RANGES[channel].step * scale}
            unit={unit}
            format={format}
            onChange={(value) => {
              onChange(withChannel(channel, value / scale));
            }}
            onCommit={(value) => {
              onCommit(withChannel(channel, value / scale));
            }}
          />
        ))}
      </div>
      <div className="flex items-center gap-1">
        <Input
          aria-label={labels.hex}
          aria-invalid={hexInvalid || undefined}
          size="xs"
          spellCheck={false}
          className="timecode flex-1"
          value={hexDraft ?? hex}
          onChange={(event) => {
            setHexDraft(event.target.value);
            setHexInvalid(false);
          }}
          onKeyDown={handleHexKeyDown}
          onBlur={() => {
            if (!acceptHex()) {
              setHexDraft(null);
              setHexInvalid(false);
            }
          }}
        />
        {isEyeDropperSupported() && (
          <IconButton
            icon={Pipette}
            label={labels.eyeDropper}
            size="xs"
            onClick={() => void pickFromScreen()}
          />
        )}
      </div>
      {palette.length > 0 && (
        <div role="group" aria-label={labels.palette} className="flex flex-wrap gap-1">
          {palette.map((swatch) => (
            <button
              key={`${swatch.label}-${swatch.color}`}
              type="button"
              aria-label={`${swatch.label} ${swatch.color}`}
              title={`${swatch.label} ${swatch.color}`}
              onClick={() => {
                const picked = colorFromHex(swatch.color, color.hue);
                if (picked !== null) {
                  onPick(picked);
                }
              }}
              className={cn('size-5 rounded-full border border-border-strong', focusRing)}
              style={{ backgroundColor: swatch.color }}
            />
          ))}
        </div>
      )}
      {recentColors.length > 0 && (
        <div role="group" aria-label={labels.recent} className="flex flex-wrap gap-1">
          {recentColors.map((recent) => (
            <button
              key={recent}
              type="button"
              aria-label={recent}
              title={recent}
              onClick={() => {
                const picked = colorFromHex(recent, color.hue);
                if (picked !== null) {
                  onPick(picked);
                }
              }}
              className={cn('size-5 rounded-xs border border-border-strong', focusRing)}
              style={{ backgroundColor: recent }}
            />
          ))}
        </div>
      )}
    </div>
  );
}
