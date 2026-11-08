// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * The app's transcription preferences (stored in `library.db`, edited in « Réglages →
 * Transcription »). Per-project choices (language, vocabulary) live in the document.
 */
export type TranscriptionPreferences = { 
/**
 * The model to use (a [`TranscriptionModelInfo::id`]); `null` uses the best installed one
 * for this machine.
 */
model: string | null, 
/**
 * Whether to use the GPU when one is available.
 */
useGpu: boolean, 
/**
 * Whether media are transcribed when they are imported.
 */
autoTranscribe: boolean, 
/**
 * Whether speech is detected first (voice model): fewer invented words over silences or
 * music, faster, and word timings bounded by speech.
 */
voiceDetection: boolean, 
/**
 * A folder holding the user's own `whisper-cli` build, searched first; `null` uses the
 * bundled one (and the GPU pack when installed).
 */
whisperDir: string | null, };
