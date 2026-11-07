import { useEffect, useSyncExternalStore } from 'react';

/**
 * A small fetch cache shared by every clip: hundreds of clips use the same few thumbnail sheets
 * and waveform files, which are fetched once per URL (URLs are versioned when a file is
 * generated again). The least recently requested entries are dropped past {@link CAPACITY}.
 */

type Entry =
  | { readonly status: 'loading' }
  | { readonly status: 'ready'; readonly value: unknown }
  | { readonly status: 'failed' };

/** Entries kept (a waveform level of a long file can weigh a few megabytes). */
const CAPACITY = 64;

const entries = new Map<string, Entry>();
const listeners = new Set<() => void>();

function notify(): void {
  for (const listener of listeners) listener();
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** Starts fetching `url` unless it is cached or in flight; marks it recently used. */
function request<T>(url: string, read: (response: Response) => Promise<T>): void {
  const existing = entries.get(url);
  if (existing !== undefined) {
    entries.delete(url);
    entries.set(url, existing);
    return;
  }
  entries.set(url, { status: 'loading' });
  while (entries.size > CAPACITY) {
    const oldest = entries.keys().next().value;
    if (oldest === undefined) break;
    entries.delete(oldest);
  }
  fetch(url)
    .then(async (response) => (response.ok ? read(response) : Promise.reject(new Error('missing'))))
    .then((value) => {
      if (entries.has(url)) entries.set(url, { status: 'ready', value });
      notify();
    })
    .catch(() => {
      // Not generated yet: a completed job changes the URL, which starts a new request.
      if (entries.has(url)) entries.set(url, { status: 'failed' });
      notify();
    });
}

/** The resource at `url` read with `read`, or `null` while loading, missing or without URL. */
export function useCachedResource<T>(
  url: string | null,
  read: (response: Response) => Promise<T>,
): T | null {
  useEffect(() => {
    if (url !== null) request(url, read);
  }, [url, read]);
  const entry = useSyncExternalStore(subscribe, () =>
    url === null ? undefined : entries.get(url),
  );
  return entry?.status === 'ready' ? (entry.value as T) : null;
}
