/**
 * `@se/engine/export`: video exports (prompt 27, ADR-0038) — the export Worker drawing frames
 * with the preview's renderer and encoding them (WebCodecs + Mediabunny, uploads to the core),
 * sound mixed in blocks offline through the preview's audio graph, and the page-side session
 * driving both. Loads PixiJS and Mediabunny: import it lazily from UI code.
 */
export { ExportAudioMixer, type ExportAudioMixerOptions, type MixedBlock } from './audio-mixer';
export {
  audioCodecName,
  maximumAudioPackets,
  maximumVideoPackets,
  webCodecsVideoCodec,
  type ExportAudioCodecName,
  type WebCodecsVideoCodec,
} from './export-codecs';
export { ExportFrameDrawer, rgbaReader, thumbnailOf, type ExportDrawIssues } from './export-frames';
export { ExportJobRunner, withoutUrls } from './export-job-runner';
export type { ExportWorkerJob, FromExportWorker, ToExportWorker } from './export-messages';
export { UPLOAD_CHUNK_BYTES, openMuxedOutput, videoEncodingConfig } from './export-output';
export { createExportRenderer, type ExportRenderer } from './export-renderer';
export {
  createExportWorker,
  startExportSession,
  type ExportScene,
  type ExportSession,
  type ExportSessionOptions,
  type ExportSessionResult,
} from './export-session';
export {
  ExportStoppedError,
  exportMediaLocator,
  httpExportSink,
  sinkWritable,
  type ExportSink,
  type ExportStreamName,
} from './export-sink';
export {
  AUDIO_BLOCK_SECONDS,
  EXPORT_CHANNELS,
  EXPORT_SAMPLE_RATE,
  audioBlocks,
  exportFrames,
  sampleCount,
  type AudioBlock,
  type ExportFrames,
} from './export-timing';
export { renderExport, type ExportRenderContext, type ExportRenderJob } from './render-export';
