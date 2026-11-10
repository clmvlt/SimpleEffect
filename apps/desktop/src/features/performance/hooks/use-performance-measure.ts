import { useState } from 'react';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey, type ErrorMessageKey } from '../../../shared/errors';
import { browserDependencies } from '../services/browser-dependencies';
import {
  measurePerformance,
  type MeasureStep,
  type PerformanceReport,
} from '../services/measure-performance';

/** Where the measure stands. */
export type MeasureState =
  | { readonly kind: 'idle' }
  | { readonly kind: 'running'; readonly step: MeasureStep }
  | { readonly kind: 'done'; readonly report: PerformanceReport }
  | { readonly kind: 'failed'; readonly errorKey: ErrorMessageKey };

/** Runs the in-app performance measure (« Mesurer les performances ») and keeps its result. */
export function usePerformanceMeasure(): {
  readonly state: MeasureState;
  readonly measure: () => void;
} {
  const client = useCoreClient();
  const [state, setState] = useState<MeasureState>({ kind: 'idle' });
  const measure = () => {
    if (state.kind === 'running') return;
    setState({ kind: 'running', step: 'core' });
    const dependencies = browserDependencies(client, (step) => {
      setState({ kind: 'running', step });
    });
    measurePerformance(dependencies).then(
      (report) => {
        setState({ kind: 'done', report });
      },
      (error: unknown) => {
        setState({ kind: 'failed', errorKey: errorMessageKey(error) });
      },
    );
  };
  return { state, measure };
}
