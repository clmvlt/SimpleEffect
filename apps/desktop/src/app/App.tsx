import { useCoreStatus } from '../shared/core-connection';
import { describeCoreStatus } from './core-status-label';

/**
 * Temporary root screen showing the se-core connection state.
 * TODO(prompt-04): replace with the editor shell built on @se/ui.
 */
export function App() {
  const status = useCoreStatus();
  return (
    <main className="flex h-screen items-center justify-center bg-neutral-950 text-neutral-100">
      <p role="status" className="text-lg">
        {describeCoreStatus(status)}
      </p>
    </main>
  );
}
