import type { Plugin } from 'vite';

import { buildContentSecurityPolicy } from './content-security-policy';

/** Placeholder in `src/index.html` replaced by the policy. */
const PLACEHOLDER = '__CONTENT_SECURITY_POLICY__';

/**
 * Vite plugin writing the Content-Security-Policy meta tag of `index.html`.
 *
 * A meta tag (rather than an HTTP header) is used because the packaged app loads the renderer
 * from `file://`, where Electron cannot attach response headers.
 */
export function contentSecurityPolicyPlugin(): Plugin {
  let devServer = false;
  return {
    name: 'simple-effect:content-security-policy',
    configResolved(config) {
      devServer = config.command === 'serve';
    },
    transformIndexHtml: {
      order: 'pre',
      handler(html) {
        return html.replace(PLACEHOLDER, buildContentSecurityPolicy({ devServer }));
      },
    },
  };
}
