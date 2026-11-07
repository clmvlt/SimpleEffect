import { assetMediaPath, decodeWaveformPeaks, mediaUrl } from '@se/protocol';
import type {
  Asset,
  AssetMediaFile,
  JobKind,
  PeakLevel,
  ThumbnailSheet,
  WaveformPeaks,
} from '@se/protocol';
import { useEffect, useState } from 'react';
import { useStore } from 'zustand';

import { useCoreEndpoint } from '../../../shared/core';
import { latestCompletion } from '../services/job-status';
import { jobsStore } from '../store/jobs-store';

/** Waveform level fetched for previews (samples per peak). */
const PREVIEW_PEAK_LEVEL: PeakLevel = 1024;

/**
 * URL of an asset's `/media` file, versioned by the latest completed job producing it so images
 * reload when it is generated again; `null` before the core is known.
 */
export function useAssetMediaUrl(asset: Asset, file: AssetMediaFile, job: JobKind): string | null {
  const endpoint = useCoreEndpoint();
  const version = useStore(jobsStore, (state) => latestCompletion(state.jobs, asset.id, job));
  if (endpoint === null) {
    return null;
  }
  const content = asset.fingerprint?.sampleHash ?? '';
  return mediaUrl(endpoint, assetMediaPath(asset.id, file), `${content}${version ?? ''}`);
}

/** Fetches `url` with `read` whenever it changes; `null` until loaded or when it fails (404). */
function useFetched<T>(url: string | null, read: (response: Response) => Promise<T>): T | null {
  const [loaded, setLoaded] = useState<{ url: string; value: T } | null>(null);
  useEffect(() => {
    if (url === null) {
      return undefined;
    }
    const controller = new AbortController();
    fetch(url, { signal: controller.signal })
      .then(async (response) => (response.ok ? read(response) : null))
      .then((value) => {
        if (value !== null) setLoaded({ url, value });
      })
      .catch(() => {
        // Not generated yet, or aborted: the next completed job changes the URL.
      });
    return () => {
      controller.abort();
    };
  }, [url, read]);
  return loaded?.url === url ? loaded.value : null;
}

const readSheet = (response: Response) => response.json() as Promise<ThumbnailSheet>;
const readPeaks = async (response: Response) => decodeWaveformPeaks(await response.arrayBuffer());

/**
 * The thumbnail sheet layout of a video or image, once generated (also for a missing original:
 * the cache keeps its pictures).
 */
export function useThumbnailSheet(asset: Asset): ThumbnailSheet | null {
  const url = useAssetMediaUrl(asset, 'thumbnails', 'thumbnails');
  return useFetched(asset.kind === 'audio' ? null : url, readSheet);
}

/** The waveform peaks of a sound file, once generated (cached ones of a missing original too). */
export function useWaveformPeaks(asset: Asset): WaveformPeaks | null {
  const url = useAssetMediaUrl(asset, { peaks: PREVIEW_PEAK_LEVEL }, 'waveformPeaks');
  return useFetched(asset.kind === 'audio' ? url : null, readPeaks);
}
