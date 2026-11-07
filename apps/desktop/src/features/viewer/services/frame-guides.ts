import {
  matchesAspect,
  safeAreaPlatform,
  type NormalisedRect,
  type SafeAreaZoneKind,
} from '@se/engine/safe-areas';

import type { ViewerGuides } from '../store/viewer-store';

/** A line of a guide, normalised in the frame. */
export interface GuideLine {
  readonly x1: number;
  readonly y1: number;
  readonly x2: number;
  readonly y2: number;
}

/** What the guides overlay draws, normalised in the frame. */
export interface FrameGuideShapes {
  /** Safe-area zones: covered by an app (shaded) or title/action safe (outlined). */
  readonly zones: readonly {
    readonly id: string;
    readonly kind: SafeAreaZoneKind;
    readonly rect: NormalisedRect;
  }[];
  /** The platform's name when its zones were measured on another aspect (indicative only). */
  readonly mismatchedPlatform: string | null;
  readonly lines: readonly GuideLine[];
}

/** Half the arm of the centre cross, as a share of the frame's shorter side. */
const CROSS_ARM = 0.03;

/** The shapes of `guides` over a frame of `frame` size. */
export function frameGuideShapes(
  guides: ViewerGuides,
  frame: { readonly width: number; readonly height: number },
): FrameGuideShapes {
  const platform = guides.safeArea === null ? undefined : safeAreaPlatform(guides.safeArea);
  const lines: GuideLine[] = [];
  if (guides.thirds) {
    for (const third of [1 / 3, 2 / 3]) {
      lines.push({ x1: third, y1: 0, x2: third, y2: 1 });
      lines.push({ x1: 0, y1: third, x2: 1, y2: third });
    }
  }
  if (guides.centre) {
    const shorter = Math.min(frame.width, frame.height);
    const armX = (CROSS_ARM * shorter) / frame.width;
    const armY = (CROSS_ARM * shorter) / frame.height;
    lines.push({ x1: 0.5 - armX, y1: 0.5, x2: 0.5 + armX, y2: 0.5 });
    lines.push({ x1: 0.5, y1: 0.5 - armY, x2: 0.5, y2: 0.5 + armY });
  }
  return {
    zones: platform?.zones ?? [],
    mismatchedPlatform:
      platform === undefined || matchesAspect(platform, frame) ? null : platform.name,
    lines,
  };
}

/** Whether any guide is on. */
export function hasGuides(guides: ViewerGuides): boolean {
  return guides.safeArea !== null || guides.thirds || guides.centre;
}
