// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * The ffmpeg tools found and what they can do, for the settings screen.
 */
export type MediaToolsStatus = { 
/**
 * Path of the ffmpeg in use, `null` when none was found.
 */
ffmpegPath: string | null, 
/**
 * Path of the ffprobe in use, `null` when none was found.
 */
ffprobePath: string | null, 
/**
 * ffmpeg's version (`9.0.2`…), when it could be read.
 */
ffmpegVersion: string | null, 
/**
 * Hardware encoders that passed a test encode (`nvenc`, `videotoolbox`, `vaapi`…), in the
 * order they are tried; empty when encoding is software only.
 */
hardwareEncoders: Array<string>, 
/**
 * Why a tool is missing, when one is.
 */
problem: string | null, };
