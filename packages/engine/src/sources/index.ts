/**
 * `@se/engine/sources`: which file the engine decodes for an asset (`docs/architecture.md` §7):
 * the original when this machine decodes it, else its mezzanine once ready (`playbackSource`,
 * `isPlayable`). Proxies for smooth preview arrive with prompts 12 and 13.
 */
export { isPlayable, playbackSource } from './playback-source';
export type { PlaybackSource } from './playback-source';
