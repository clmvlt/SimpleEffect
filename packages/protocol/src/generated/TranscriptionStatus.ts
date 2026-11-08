// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { GpuRuntimeInfo } from "./GpuRuntimeInfo";
import type { TranscriptionBackend } from "./TranscriptionBackend";
import type { TranscriptionModelInfo } from "./TranscriptionModelInfo";
import type { TranscriptionPreferences } from "./TranscriptionPreferences";
import type { VoiceModelInfo } from "./VoiceModelInfo";
import type { WhisperInfo } from "./WhisperInfo";

/**
 * Everything « Réglages → Transcription » shows.
 */
export type TranscriptionStatus = { 
/**
 * The program that transcribes.
 */
whisper: WhisperInfo, 
/**
 * The GPU detected (`NVIDIA GeForce RTX 3080`, `Apple M2`), if any.
 */
gpuName: string | null, 
/**
 * Where the next transcription should run.
 */
backend: TranscriptionBackend, 
/**
 * Where the last transcription of this session ran.
 */
lastBackend: TranscriptionBackend | null, 
/**
 * The GPU pack this platform offers, if any.
 */
gpuRuntime: GpuRuntimeInfo | null, 
/**
 * The catalogue models, then imported ones.
 */
models: Array<TranscriptionModelInfo>, 
/**
 * The voice detection model.
 */
voiceModel: VoiceModelInfo, 
/**
 * Absolute path of the models folder (`<app data>/models/whisper`).
 */
modelsDir: string, 
/**
 * The preferences.
 */
preferences: TranscriptionPreferences, };
