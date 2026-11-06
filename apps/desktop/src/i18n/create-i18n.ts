import i18next, { type i18n } from 'i18next';
import { initReactI18next } from 'react-i18next';

import fr from './locales/fr.json';

/** Language of the interface. French is the only locale so far. */
export const DEFAULT_LANGUAGE = 'fr';

/** Every bundled translation, by language. Keys are typed from `fr.json` (`i18next.d.ts`). */
export const resources = { fr: { translation: fr } } as const;

/**
 * Creates the app's i18next instance, initialised synchronously from the bundled resources (no
 * network, no lazy loading: golden rule 1), so the first render already has its strings. Tests
 * create their own instance the same way.
 */
export function createI18n(): i18n {
  const instance = i18next.createInstance();
  void instance.use(initReactI18next).init({
    lng: DEFAULT_LANGUAGE,
    fallbackLng: DEFAULT_LANGUAGE,
    resources,
    initAsync: false,
    // React already escapes text: i18next must not escape it a second time.
    interpolation: { escapeValue: false },
  });
  return instance;
}
