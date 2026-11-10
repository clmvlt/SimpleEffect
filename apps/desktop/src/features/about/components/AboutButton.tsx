import { useTranslation } from 'react-i18next';

import { openAbout } from '../store/about-dialog-store';

/** The app version in the status bar; a click opens « À propos ». */
export function AboutButton() {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      onClick={() => {
        openAbout();
      }}
      title={t('about.open')}
      className="rounded-sm px-1 tabular-nums hover:bg-hover hover:text-text"
    >
      {t('statusBar.version', { version: __APP_VERSION__ })}
    </button>
  );
}
