import type { AnimationPhase, AnimationPresetInfo, Flicks } from '@se/protocol';
import { cn, focusRing } from '@se/ui';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { FLICKS_PER_SECOND } from '../../../shared/time';
import { previewStyle } from '../services/preset-preview';

/** Props of {@link PresetCard}. */
export interface PresetCardProps {
  readonly preset: AnimationPresetInfo;
  readonly phase: AnimationPhase;
  readonly disabled: boolean;
  readonly onApply: () => void;
}

/** One second of animation then a short pause, looped while hovered. */
const LOOP_SECONDS = 1.4;
const SWATCH = { width: 96, height: 54 };

/**
 * A preset button: its name, and a « Aa » swatch playing the preset (from the core's sample)
 * while the pointer is over it.
 */
export function PresetCard({ preset, phase, disabled, onApply }: PresetCardProps) {
  const { t } = useTranslation();
  const swatch = useRef<HTMLSpanElement | null>(null);
  const [hovered, setHovered] = useState(false);
  const sample = preset.samples.find((candidate) => candidate.phase === phase);
  useEffect(() => {
    const element = swatch.current;
    if (!hovered || sample === undefined || element === null) return undefined;
    const started = performance.now();
    let frame = 0;
    const tick = (now: number) => {
      const seconds = Math.min(1, ((now - started) / 1000) % LOOP_SECONDS);
      const style = previewStyle(sample, Math.round(seconds * FLICKS_PER_SECOND) as Flicks, SWATCH);
      Object.assign(element.style, style);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
      element.removeAttribute('style');
    };
  }, [hovered, sample]);
  return (
    <button
      type="button"
      disabled={disabled}
      data-preset={preset.id}
      onClick={onApply}
      onPointerEnter={() => {
        setHovered(true);
      }}
      onPointerLeave={() => {
        setHovered(false);
      }}
      className={cn(
        'flex flex-col items-center gap-1 rounded-md border border-border bg-panel-raised p-1.5 text-xs text-text hover:border-border-strong disabled:opacity-40',
        focusRing,
      )}
    >
      <span
        aria-hidden
        className="flex items-center justify-center overflow-hidden rounded-sm bg-bg"
        style={SWATCH}
      >
        <span ref={swatch} className="inline-block text-lg font-semibold">
          {t('editor.animations.swatch')}
        </span>
      </span>
      {t(`editor.animations.presets.${preset.id}`, { defaultValue: preset.name })}
    </button>
  );
}
