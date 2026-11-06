/** Options for {@link buildContentSecurityPolicy}. */
export interface ContentSecurityPolicyOptions {
  /**
   * True when the renderer is served by the Vite dev server: Fast Refresh needs inline scripts,
   * injected styles and a WebSocket to the dev server. Never true in a packaged app.
   */
  readonly devServer: boolean;
}

/** se-core listens on 127.0.0.1 with a random port, chosen at startup. */
const CORE_HTTP = 'http://127.0.0.1:*';
const CORE_WEBSOCKET = 'ws://127.0.0.1:*';

/**
 * Builds the renderer's Content-Security-Policy.
 *
 * Production allows network connections to the local core (127.0.0.1) only: no CDN, no remote
 * fonts, no telemetry (golden rule 1). Everything else is restricted to the app's own files.
 *
 * Inline styles are allowed in every mode: Radix (scroll areas, select viewports) and sonner insert
 * `<style>` elements at runtime. Styles cannot run code, and the app loads no remote resource a style
 * could exfiltrate data to; scripts stay restricted to the app's own files (ADR-0013).
 */
export function buildContentSecurityPolicy({ devServer }: ContentSecurityPolicyOptions): string {
  const directives: Record<string, readonly string[]> = {
    'default-src': ["'none'"],
    'script-src': devServer ? ["'self'", "'unsafe-inline'"] : ["'self'"],
    'style-src': ["'self'", "'unsafe-inline'"],
    'img-src': ["'self'", 'data:', 'blob:', CORE_HTTP],
    'media-src': ["'self'", 'blob:', CORE_HTTP],
    'font-src': ["'self'"],
    'connect-src': devServer
      ? ["'self'", CORE_WEBSOCKET, CORE_HTTP, 'ws://localhost:*']
      : [CORE_WEBSOCKET, CORE_HTTP],
    'worker-src': ["'self'", 'blob:'],
    'object-src': ["'none'"],
    'base-uri': ["'none'"],
    'form-action': ["'none'"],
  };
  return Object.entries(directives)
    .map(([directive, sources]) => `${directive} ${sources.join(' ')}`)
    .join('; ');
}
