import type { Asset, DerivedFile, DerivedKind, JobKind } from '@se/protocol';

import type { AssetJobStatus } from './job-status';

/** What the media bin shows about a mezzanine or proxy. */
export type DerivedDisplay =
  | { readonly kind: 'none' }
  | { readonly kind: 'queued' }
  | { readonly kind: 'running'; readonly progress: number | null }
  | { readonly kind: 'ready' }
  | { readonly kind: 'failed'; readonly message: string };

/** The job kind making a derived copy. */
const JOB_KINDS: Readonly<Record<DerivedKind, JobKind>> = {
  mezzanine: 'mezzanine',
  proxy: 'proxy',
};

/** The derived copy of `kind` recorded in the asset. */
export function derivedFile(asset: Asset, kind: DerivedKind): DerivedFile | null {
  return kind === 'mezzanine' ? asset.derived.mezzanine : asset.derived.proxy;
}

/**
 * The state of a derived copy: the document says pending, ready or failed (with the reason); a
 * pending copy shows its job's progress when it runs, « en attente » otherwise.
 */
export function derivedDisplay(
  asset: Asset,
  kind: DerivedKind,
  jobs: AssetJobStatus,
): DerivedDisplay {
  const file = derivedFile(asset, kind);
  switch (file?.status.kind) {
    case undefined:
      return { kind: 'none' };
    case 'ready':
      return { kind: 'ready' };
    case 'failed':
      return { kind: 'failed', message: file.status.message };
    case 'pending': {
      const running = jobs.running.find((job) => job.kind === JOB_KINDS[kind]);
      return running === undefined
        ? { kind: 'queued' }
        : { kind: 'running', progress: running.progress };
    }
  }
}

/** Whether « Générer un proxy » applies: a video that has no proxy, or whose proxy failed. */
export function canGenerateProxy(asset: Asset): boolean {
  const proxy = asset.derived.proxy;
  return (
    asset.kind === 'video' &&
    !asset.missing &&
    asset.mediaInfo?.videoStream != null &&
    (proxy === null || proxy.status.kind === 'failed')
  );
}
