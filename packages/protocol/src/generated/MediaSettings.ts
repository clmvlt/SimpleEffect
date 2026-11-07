// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ProxyResolution } from "./ProxyResolution";

/**
 * Media settings of the app (stored in `library.db`, edited in the settings screen).
 */
export type MediaSettings = { 
/**
 * Quality of new mezzanines as an x264/x265 CRF (lower is better and larger):
 * [`MediaSettings::MEZZANINE_QUALITY_RANGE`], default 14 (visually lossless).
 */
mezzanineQuality: number, 
/**
 * Height (short side) of new proxies.
 */
proxyResolution: ProxyResolution, 
/**
 * Whether proxies are generated automatically for footage larger than 1080p or that
 * WebCodecs cannot decode (otherwise only on request).
 */
autoProxies: boolean, 
/**
 * Folder holding ffmpeg and ffprobe, searched before the bundled ones; `null` uses the
 * bundled tools.
 */
toolsDir: string | null, };
