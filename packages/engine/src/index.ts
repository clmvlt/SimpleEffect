/**
 * `@se/engine`: the rendering engine shared by preview and export (WYSIWYG, ADR-0002).
 *
 * The root entry holds the pure, dependency-free parts: keyframe interpolation and sequence
 * evaluation. GPU and decoding parts live behind subpath entries that load PixiJS and Mediabunny:
 * `@se/engine/preview` (viewer engine), `@se/engine/composite`, `@se/engine/decode`.
 */
export * from './animation';
export * from './evaluate';
export { FLICKS_PER_SECOND } from './time/flicks';
