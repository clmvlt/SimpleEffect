import { Fragment } from 'react';

import { useConnectionStatus, useCoreInfo } from '../shared/core';
import { describeConnection, describeCoreInfo } from './connection-label';

/**
 * Temporary root screen showing the se-core connection state and `getCoreInfo`.
 * TODO(prompt-04): replace with the editor shell built on @se/ui.
 */
export function App() {
  const status = useConnectionStatus();
  const info = useCoreInfo();
  const rows = status === 'open' && info !== null ? describeCoreInfo(info) : [];
  return (
    <main className="flex h-screen flex-col items-center justify-center gap-6 bg-neutral-950 text-neutral-100">
      <p role="status" className="text-lg">
        {describeConnection(status, info)}
      </p>
      {rows.length > 0 && (
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm text-neutral-400">
          {rows.map((row) => (
            <Fragment key={row.label}>
              <dt>{row.label}</dt>
              <dd className="font-mono">{row.value}</dd>
            </Fragment>
          ))}
        </dl>
      )}
    </main>
  );
}
