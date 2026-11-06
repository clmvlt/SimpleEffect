import { Slider as SliderPrimitive } from 'radix-ui';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';
import {
  CHANNEL_RANGES,
  channelGradient,
  type ColorChannel,
  type PickerColor,
} from './color-model';

/** Props of {@link ColorChannelSlider}. */
export interface ColorChannelSliderProps {
  color: PickerColor;
  channel: ColorChannel;
  label: string;
  /** Live value while sliding. */
  onChange: (value: number) => void;
  /** Final value at the end of the gesture (pointer release, each key press). */
  onCommit: (value: number) => void;
  disabled?: boolean | undefined;
}

/**
 * A slider for one OKLCH channel whose track previews the colours along it. Arrow keys step,
 * Page Up/Down jump ten steps, Home/End go to the ends.
 */
export function ColorChannelSlider({
  color,
  channel,
  label,
  onChange,
  onCommit,
  disabled = false,
}: ColorChannelSliderProps) {
  const { min, max, step } = CHANNEL_RANGES[channel];
  return (
    <SliderPrimitive.Root
      value={[color[channel]]}
      min={min}
      max={max}
      step={step}
      disabled={disabled}
      onValueChange={([value]) => {
        if (value !== undefined) {
          onChange(value);
        }
      }}
      onValueCommit={([value]) => {
        if (value !== undefined) {
          onCommit(value);
        }
      }}
      className="relative flex h-4 w-full touch-none items-center select-none data-[disabled]:opacity-50"
    >
      <SliderPrimitive.Track
        className="relative h-2.5 grow rounded-full border border-border"
        style={{ backgroundImage: channelGradient(color, channel) }}
      />
      <SliderPrimitive.Thumb
        aria-label={label}
        className={cn(
          'block size-3.5 rounded-full border-2 border-text shadow-overlay',
          'data-[disabled]:pointer-events-none',
          focusRing,
        )}
      />
    </SliderPrimitive.Root>
  );
}
