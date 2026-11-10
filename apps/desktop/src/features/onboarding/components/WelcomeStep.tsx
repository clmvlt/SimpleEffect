import { useTranslation } from 'react-i18next';

/** « Bienvenue »: what SimpleEffect is and what the next steps check. */
export function WelcomeStep() {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col gap-3 text-sm text-text">
      <p>{t('onboarding.welcome.intro')}</p>
      <ul className="flex list-disc flex-col gap-1 pl-5 text-text-muted">
        <li>{t('onboarding.welcome.local')}</li>
        <li>{t('onboarding.welcome.ai')}</li>
        <li>{t('onboarding.welcome.editable')}</li>
      </ul>
      <p className="text-text-muted">{t('onboarding.welcome.next')}</p>
    </div>
  );
}
