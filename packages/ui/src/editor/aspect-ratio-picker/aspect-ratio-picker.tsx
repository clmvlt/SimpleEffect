import { RadioGroup as RadioGroupPrimitive } from 'radix-ui';
import { useState } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';
import { ScrubInput } from '../scrub-input';

/** An aspect ratio as two positive integers, e.g. `{ width: 16, height: 9 }`. */
export interface AspectRatio {
  readonly width: number;
  readonly height: number;
}

/** The ratios offered as presets: landscape, vertical (shorts, reels), square, portrait feed. */
export const ASPECT_RATIO_PRESETS: readonly AspectRatio[] = [
  { width: 16, height: 9 },
  { width: 9, height: 16 },
  { width: 1, height: 1 },
  { width: 4, height: 5 },
];

const CUSTOM = 'custom';

/** `16:9`: the label and radio value of a ratio. */
export function formatAspectRatio({ width, height }: AspectRatio): string {
  return `${String(width)}:${String(height)}`;
}

/** Whether two ratios are the same shape (`32:18` equals `16:9`). */
export function sameAspectRatio(a: AspectRatio, b: AspectRatio): boolean {
  return a.width * b.height === b.width * a.height;
}

/** Props of {@link AspectRatioPicker}. */
export interface AspectRatioPickerProps {
  value: AspectRatio;
  onChange: (value: AspectRatio) => void;
  /** Accessible name of the choice (default English « Aspect ratio »). */
  label?: string | undefined;
  /** Label of the custom option (default English « Custom »). */
  customLabel?: string | undefined;
  /** Accessible names of the custom width and height fields. */
  customWidthLabel?: string | undefined;
  customHeightLabel?: string | undefined;
  presets?: readonly AspectRatio[] | undefined;
  disabled?: boolean | undefined;
  className?: string | undefined;
}

/** A frame drawn at the ratio's proportions inside a fixed 20 px box. */
function RatioThumbnail({ ratio }: { ratio: AspectRatio | null }) {
  const scale = ratio === null ? 1 : 1 / Math.max(ratio.width, ratio.height);
  return (
    <span aria-hidden className="flex size-5 items-center justify-center">
      <span
        className={cn('rounded-[2px] border border-current', ratio === null && 'border-dashed')}
        style={{
          width: `${String(ratio === null ? 70 : ratio.width * scale * 100)}%`,
          height: `${String(ratio === null ? 70 : ratio.height * scale * 100)}%`,
        }}
      />
    </span>
  );
}

const optionClasses = cn(
  'flex h-12 min-w-11 flex-col items-center justify-center gap-0.5 rounded-md border border-border px-1.5',
  'text-2xs text-text-muted transition-colors duration-(--duration-fast) hover:bg-hover hover:text-text',
  'data-[state=checked]:border-accent data-[state=checked]:bg-accent-muted data-[state=checked]:text-text',
  'disabled:pointer-events-none disabled:opacity-50',
  focusRing,
);

/**
 * Picks a sequence or crop format: 16:9, 9:16, 1:1, 4:5 or a custom ratio, each with a thumbnail
 * drawn to scale. A radio group (arrow keys move the choice). Choosing « custom » shows width and
 * height fields (it stays chosen while they are edited, even through a preset's shape); a value
 * that matches no preset always shows as custom.
 */
export function AspectRatioPicker({
  value,
  onChange,
  label = 'Aspect ratio',
  customLabel = 'Custom',
  customWidthLabel = 'Ratio width',
  customHeightLabel = 'Ratio height',
  presets = ASPECT_RATIO_PRESETS,
  disabled = false,
  className,
}: AspectRatioPickerProps) {
  const [customChosen, setCustomChosen] = useState(false);
  const matchingPreset = presets.find((preset) => sameAspectRatio(preset, value));
  const selected =
    customChosen || matchingPreset === undefined ? CUSTOM : formatAspectRatio(matchingPreset);
  return (
    <div className={cn('flex flex-col gap-2', className)}>
      <RadioGroupPrimitive.Root
        aria-label={label}
        value={selected}
        disabled={disabled}
        orientation="horizontal"
        onValueChange={(next) => {
          // Custom keeps the current shape; the fields below change it.
          setCustomChosen(next === CUSTOM);
          const preset = presets.find((candidate) => formatAspectRatio(candidate) === next);
          if (preset !== undefined) {
            onChange(preset);
          }
        }}
        className="flex flex-wrap gap-1"
      >
        {presets.map((preset) => (
          <RadioGroupPrimitive.Item
            key={formatAspectRatio(preset)}
            value={formatAspectRatio(preset)}
            className={optionClasses}
          >
            <RatioThumbnail ratio={preset} />
            {formatAspectRatio(preset)}
          </RadioGroupPrimitive.Item>
        ))}
        <RadioGroupPrimitive.Item value={CUSTOM} className={optionClasses}>
          <RatioThumbnail ratio={null} />
          {customLabel}
        </RadioGroupPrimitive.Item>
      </RadioGroupPrimitive.Root>
      {selected === CUSTOM && (
        <div className="flex items-center gap-1">
          <ScrubInput
            label={customWidthLabel}
            size="xs"
            className="w-16"
            value={value.width}
            min={1}
            max={100}
            disabled={disabled}
            onChange={(width) => {
              onChange({ width, height: value.height });
            }}
          />
          <span aria-hidden className="text-xs text-text-subtle">
            :
          </span>
          <ScrubInput
            label={customHeightLabel}
            size="xs"
            className="w-16"
            value={value.height}
            min={1}
            max={100}
            disabled={disabled}
            onChange={(height) => {
              onChange({ width: value.width, height });
            }}
          />
        </div>
      )}
    </div>
  );
}
