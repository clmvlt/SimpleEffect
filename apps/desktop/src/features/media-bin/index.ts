/**
 * Media bin feature: import (dialog or drop), previews with thumbnails and waveforms, job
 * progress, details, missing media and relinking (prompt 08); assets drag to the timeline and
 * other panels can reveal one (`revealAsset`), read the selected one (`useSelectedAssetId`) and
 * the progress of an asset's transcription (`useTranscriptionJob`, prompt 21).
 */
export { MediaBinPanel } from './components/MediaBinPanel';
export { useAssetMediaUrl } from './hooks/use-asset-media';
export {
  useAssetJob,
  useTranscriptionJob,
  type AssetJobState,
  type TranscriptionJobState,
} from './hooks/use-transcription-job';
export { revealAsset, useSelectedAssetId } from './store/media-bin-store';
