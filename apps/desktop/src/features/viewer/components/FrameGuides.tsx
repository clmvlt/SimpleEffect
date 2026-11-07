import { useTranslation } from 'react-i18next';

import { frameGuideShapes, hasGuides } from '../services/frame-guides';
import type { ViewerGuides } from '../store/viewer-store';

/** Props of {@link FrameGuides}. */
export interface FrameGuidesProps {
  readonly guides: ViewerGuides;
  readonly frame: { readonly width: number; readonly height: number };
}

/**
 * The guides over the picture (prompt 16): a platform's safe areas (zones its interface covers
 * shaded, title and action safe outlined), the rule-of-thirds grid and the centre cross. Drawn
 * by the viewer only: never part of a frame or an export.
 */
export function FrameGuides({ guides, frame }: FrameGuidesProps) {
  const { t } = useTranslation();
  if (!hasGuides(guides)) return null;
  const shapes = frameGuideShapes(guides, frame);
  return (
    <div className="pointer-events-none absolute inset-0" data-frame-guides="">
      <svg className="size-full" viewBox="0 0 1 1" preserveAspectRatio="none" aria-hidden="true">
        {shapes.zones.map((zone) => {
          const { x, y, width, height } = zone.rect;
          const covered = zone.kind === 'covered';
          return (
            <rect
              key={zone.id}
              data-safe-zone={zone.id}
              x={x}
              y={y}
              width={width}
              height={height}
              vectorEffect="non-scaling-stroke"
              className={
                covered
                  ? 'fill-warning/25 stroke-warning/70'
                  : zone.kind === 'titleSafe'
                    ? 'fill-none stroke-info'
                    : 'fill-none stroke-info/60'
              }
              strokeWidth={1}
              strokeDasharray={covered ? undefined : '4 3'}
            />
          );
        })}
        {shapes.lines.map((line, index) => (
          <line
            key={index}
            {...line}
            vectorEffect="non-scaling-stroke"
            className="stroke-text/60"
            strokeWidth={1}
          />
        ))}
      </svg>
      {shapes.mismatchedPlatform !== null && (
        <span className="absolute top-1 left-1 rounded-sm bg-overlay/80 px-1.5 py-0.5 text-xs text-text-muted">
          {t('editor.viewer.guides.mismatch', { platform: shapes.mismatchedPlatform })}
        </span>
      )}
    </div>
  );
}
