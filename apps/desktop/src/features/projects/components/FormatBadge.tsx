import type { Format } from '@se/protocol';
import { Badge } from '@se/ui';

import { formatBadgeLabel } from '../services/format-label';

/** The format of a project as a small badge (`16:9`, `9:16`, `1280×544`). */
export function FormatBadge({ format }: { readonly format: Format }) {
  return <Badge variant="neutral">{formatBadgeLabel(format)}</Badge>;
}
