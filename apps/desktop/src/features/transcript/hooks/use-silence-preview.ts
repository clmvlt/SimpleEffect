import type { CutSummary } from '@se/protocol';
import { useEffect, useState } from 'react';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey, type ErrorMessageKey } from '../../../shared/errors';
import { clearCutPreview, showCutPreview } from '../../timeline';
import { previewSilenceCuts, type SilenceCutRequest } from '../api';

/** Wait after the last change before asking the core (sliders move fast). */
const DEBOUNCE_MS = 200;

/** The preview of a silence removal. */
export interface SilencePreview {
  /** The cuts it would make; `null` before the first answer. */
  readonly summary: CutSummary | null;
  /** Whether the shown preview is older than the request (a newer one is on its way). */
  readonly loading: boolean;
  /** The i18n key of the last failure. */
  readonly errorKey: ErrorMessageKey | null;
}

/** The last answer and the request it answers. */
interface Answer {
  readonly key: string | null;
  readonly summary: CutSummary | null;
  readonly errorKey: ErrorMessageKey | null;
}

/**
 * Previews `request` (`null`: nothing to preview) a moment after it, or the project (`revision`:
 * a cut, an edit), last changed, and shows the cuts on the timeline as temporary bands; they
 * disappear with the panel.
 */
export function useSilencePreview(
  request: SilenceCutRequest | null,
  revision: number,
): SilencePreview {
  const client = useCoreClient();
  const [answer, setAnswer] = useState<Answer>({ key: null, summary: null, errorKey: null });
  // The request and revision as text: a stable dependency (the caller builds a new request
  // object at every render).
  const key = request === null ? null : JSON.stringify({ request, revision });
  useEffect(() => {
    if (key === null) {
      clearCutPreview();
      return undefined;
    }
    const asked = (JSON.parse(key) as { request: SilenceCutRequest }).request;
    let current = true;
    const timer = setTimeout(() => {
      previewSilenceCuts(client, asked).then(
        (summary) => {
          if (!current) return;
          showCutPreview(asked.sequenceId, summary.cuts);
          setAnswer({ key, summary, errorKey: null });
        },
        (error: unknown) => {
          if (!current) return;
          clearCutPreview();
          setAnswer({ key, summary: null, errorKey: errorMessageKey(error) });
        },
      );
    }, DEBOUNCE_MS);
    return () => {
      current = false;
      clearTimeout(timer);
    };
  }, [client, key]);
  useEffect(() => clearCutPreview, []);
  return {
    summary: answer.summary,
    loading: key !== null && key !== answer.key,
    errorKey: answer.errorKey,
  };
}
