import type { ProjectSession } from '@se/protocol';

import { captionRows, type CaptionRow } from '../services/caption-rows';

export {
  captionSeconds as seconds,
  captionSession,
} from '../../../shared/project-store/testing/caption-fixture';

/** The rows of a session's first sequence. */
export function captionRowsOf(project: ProjectSession): CaptionRow[] {
  const { snapshot } = project;
  const sequence = snapshot.sequences[0];
  if (sequence === undefined) return [];
  const tracks = Object.fromEntries(snapshot.tracks.map((track) => [track.id, track]));
  const items = Object.fromEntries(snapshot.items.map((item) => [item.id, item]));
  return captionRows(sequence, tracks, items);
}
