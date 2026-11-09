import type { FaceBox, ItemId, SequenceEntity, Vec2 } from '@se/protocol';
import { clipSourceTime } from '@se/engine/evaluate';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { facesAt, focusKeyframes, focusTrajectory, useSubjectDetection } from '../../reframing';
import { usePlayheadTime } from '../../timeline';
import { picturePoint, shownFocus, type CroppedLayer } from '../services/reframe-view';

/** Points of the trajectory drawn over a clip's duration. */
const TRAJECTORY_STEPS = 160;

const svgPoints = (list: readonly Vec2[], scale: number) =>
  list.map((point) => `${String(point.x * scale)},${String(point.y * scale)}`).join(' ');

/** The corners of a face box, normalised in the picture. */
function faceCorners(face: FaceBox): readonly Vec2[] {
  return [
    { x: face.x, y: face.y },
    { x: face.x + face.width, y: face.y },
    { x: face.x + face.width, y: face.y + face.height },
    { x: face.x, y: face.y + face.height },
  ];
}

/** Props of {@link ReframeTrajectory}. */
export interface ReframeTrajectoryProps {
  readonly layer: CroppedLayer;
  readonly sequence: SequenceEntity;
  readonly itemId: ItemId;
  /** CSS pixels per frame pixel. */
  readonly displayScale: number;
}

/**
 * Inside the reframing overlay's SVG (prompt 26): the path the frame's centre takes across the
 * picture over the clip (its focus keyframes as dots), the centre kept now, and the faces the
 * detector found in the picture under the playhead — to see why a tracked clip frames what it
 * frames, and where to drag.
 */
export function ReframeTrajectory({
  layer,
  sequence,
  itemId,
  displayScale,
}: ReframeTrajectoryProps) {
  const { t } = useTranslation();
  const item = useProjectSelector((state) => state.mirror?.items[itemId]);
  const clip = item?.kind.kind === 'mediaClip' ? item.kind : null;
  const detection = useSubjectDetection(clip?.assetId ?? null);
  const playhead = usePlayheadTime(sequence.id);
  if (item === undefined || clip === null) return null;
  const frame = sequence.format;
  const place = (point: Vec2) => picturePoint(layer, frame, point);
  const focus =
    clip.reframe.kind === 'fill' || clip.reframe.kind === 'tracked' ? clip.reframe.focus : null;
  const path =
    focus?.kind === 'keyframed'
      ? focusTrajectory(focus, item.timelineRange.duration, TRAJECTORY_STEPS).map(place)
      : [];
  const keyframes = focus === null ? [] : focusKeyframes(focus).map(place);
  const centre = place(shownFocus(layer.crop));
  const faces =
    detection === null
      ? []
      : facesAt(detection, clipSourceTime(clip, item.timelineRange, playhead));
  return (
    <g data-reframe-trajectory="">
      {path.length > 1 && (
        <polyline
          points={svgPoints(path, displayScale)}
          className="fill-none stroke-accent opacity-70"
          strokeWidth={1.5}
        >
          <title>{t('editor.reframing.viewer.trajectory')}</title>
        </polyline>
      )}
      {keyframes.map((point, index) => (
        <circle
          key={index}
          cx={point.x * displayScale}
          cy={point.y * displayScale}
          r={2.5}
          className="fill-accent"
        />
      ))}
      <circle
        cx={centre.x * displayScale}
        cy={centre.y * displayScale}
        r={4}
        className="fill-none stroke-text"
        strokeWidth={1.5}
      />
      {faces.map((face, index) => (
        <polygon
          key={index}
          data-detected-face=""
          points={svgPoints(faceCorners(face).map(place), displayScale)}
          className="fill-none stroke-success"
          strokeWidth={1.5}
        >
          <title>
            {t('editor.reframing.viewer.face', { percent: Math.round(face.score * 100) })}
          </title>
        </polygon>
      ))}
    </g>
  );
}
