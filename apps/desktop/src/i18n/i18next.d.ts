import 'i18next';

import type { resources } from './create-i18n';

/**
 * Types translation keys from `locales/fr.json`: `t('home.newProject')` compiles, a misspelt or
 * missing key does not.
 */
declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'translation';
    resources: (typeof resources)['fr'];
  }
}
