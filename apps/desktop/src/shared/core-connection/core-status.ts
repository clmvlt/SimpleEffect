/** State of the renderer's connection to se-core. */
export type CoreStatus =
  | { readonly kind: 'connecting' }
  | { readonly kind: 'connected'; readonly version: string }
  | { readonly kind: 'reconnecting' };
