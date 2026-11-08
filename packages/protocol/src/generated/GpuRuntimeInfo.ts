// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { DownloadProgress } from "./DownloadProgress";

/**
 * The GPU pack this platform can install (Windows: the CUDA build of whisper.cpp).
 */
export type GpuRuntimeInfo = { 
/**
 * What it is (`CUDA 12.4`).
 */
name: string, 
/**
 * Download size in bytes.
 */
downloadSize: number, 
/**
 * Whether it is installed.
 */
installed: boolean, 
/**
 * Its download, while one runs or after one failed.
 */
download: DownloadProgress | null, };
