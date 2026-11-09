import type { AutoReframeReport, ItemEntity, Reframe, TrackingSettings } from '@se/protocol';

/** The domain's default tracking settings (`TrackingSettings::default`, ADR-0037). */
export const DEFAULT_TRACKING_SETTINGS: TrackingSettings = {
  smoothing: 0.5,
  eyeLine: 0.33,
  groupFraming: true,
  deadZone: 0.15,
};

/** Ranges of the numeric settings (the domain's `TrackingSettings::problems`). */
export const TRACKING_RANGES = {
  smoothing: { min: 0, max: 1 },
  eyeLine: { min: 0.2, max: 0.8 },
  deadZone: { min: 0, max: 0.5 },
} as const;

/** The numeric settings. */
export type NumericTrackingSetting = keyof typeof TRACKING_RANGES;

/** The tracking settings of a tracked clip, `null` for any other item. */
export function trackingOf(item: ItemEntity): TrackingSettings | null {
  return item.kind.kind === 'mediaClip' && item.kind.reframe.kind === 'tracked'
    ? item.kind.reframe.tracking
    : null;
}

/** Whether `reframe` follows its subject. */
export function isTracked(reframe: Reframe): boolean {
  return reframe.kind === 'tracked';
}

/**
 * The settings to show for `items`: those of the first tracked clip (relaunching applies them to
 * every clip), else the defaults.
 */
export function shownTracking(items: readonly ItemEntity[]): TrackingSettings {
  for (const item of items) {
    const tracking = trackingOf(item);
    if (tracking !== null) return tracking;
  }
  return DEFAULT_TRACKING_SETTINGS;
}

/** `settings` with `setting` set to `value`, kept in its range. */
export function withSetting(
  settings: TrackingSettings,
  setting: NumericTrackingSetting,
  value: number,
): TrackingSettings {
  const { min, max } = TRACKING_RANGES[setting];
  const kept = Math.min(max, Math.max(min, Math.round(value * 100) / 100));
  return { ...settings, [setting]: kept };
}

/** What a reframing request did, as one toast. */
export type ReframeNotice =
  | { readonly kind: 'applied'; readonly count: number }
  | { readonly kind: 'analysing'; readonly count: number }
  | { readonly kind: 'nothing' };

/** The toast of `report`: clips reframed now, media being analysed, or nothing to follow. */
export function reportNotice(report: AutoReframeReport): ReframeNotice {
  if (report.waitingAssetIds.length > 0) {
    return { kind: 'analysing', count: report.waitingAssetIds.length };
  }
  if (report.appliedItemIds.length > 0) {
    return { kind: 'applied', count: report.appliedItemIds.length };
  }
  return { kind: 'nothing' };
}
