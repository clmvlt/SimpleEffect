import type { AiSettings, AiStatus, EffortLevel } from '@se/protocol';
import {
  Button,
  Field,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Slider,
  Spinner,
  StatusDot,
} from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useDesktopBridge } from '../../../shared/desktop';
import {
  aiAvailability,
  aiTextValues,
  useAiRefreshing,
  useAiStatus,
  useRefreshAiStatus,
} from '../../ai';

/** Value of the « CLI default » choice in the model and effort lists. */
const CLI_DEFAULT = 'cli-default';
/** Accepted numbers of tasks at once, as in `AiSettings::MAX_CONCURRENT_TASKS_RANGE`. */
const MAX_TASKS_MIN = 1;
const MAX_TASKS_MAX = 8;

/** Props of {@link AiSettingsSection}. */
export interface AiSettingsSectionProps {
  readonly settings: AiSettings;
  readonly onSave: (changes: Partial<AiSettings>) => void;
}

/**
 * « IA »: the CLI found (path, version), its login state with « Actualiser », a custom CLI path,
 * the default model and effort of new tasks, and how many tasks run at once.
 */
export function AiSettingsSection({ settings, onSave }: AiSettingsSectionProps) {
  const { t } = useTranslation();
  const status = useAiStatus();
  return (
    <div className="flex flex-col gap-4">
      <ProviderStatus status={status} />
      <ExecutablePath settings={settings} onSave={onSave} />
      <TaskDefaults settings={settings} status={status} onSave={onSave} />
      <p className="text-2xs text-text-muted">{t('settings.ai.privacy')}</p>
    </div>
  );
}

function ProviderStatus({ status }: { readonly status: AiStatus | null }) {
  const { t } = useTranslation();
  const bridge = useDesktopBridge();
  const refreshing = useAiRefreshing();
  const refresh = useRefreshAiStatus();
  const availability = aiAvailability(status);
  const values = aiTextValues(status, t('statusBar.ai.fallbackName'));
  const installation = status?.installation;
  return (
    <section aria-labelledby="settings-ai-provider" className="flex flex-col gap-2">
      <h3 id="settings-ai-provider" className="text-sm font-semibold text-text">
        {t('settings.ai.providerTitle', values)}
      </h3>
      <dl className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-1 text-xs">
        <dt className="text-text-muted">{t('settings.ai.cli')}</dt>
        <dd className="min-w-0 text-text" data-testid="ai-installation">
          {installation?.kind === 'installed' ? (
            <span className="block truncate select-text" title={installation.path}>
              {t('settings.ai.installed', values)} — {installation.path}
            </span>
          ) : (
            t(`settings.ai.installation.${installation?.kind ?? 'checking'}`, values)
          )}
        </dd>
        <dt className="text-text-muted">{t('settings.ai.login')}</dt>
        <dd className="flex items-center gap-1.5 text-text" role="status">
          <StatusDot
            size="xs"
            tone={
              availability === 'ready'
                ? 'success'
                : availability === 'loggedOut'
                  ? 'warning'
                  : 'neutral'
            }
          />
          {availability === 'ready' && values.account !== ''
            ? t('settings.ai.auth.readyAccount', values)
            : t(`settings.ai.auth.${availability}`, values)}
        </dd>
      </dl>
      {installation !== undefined && installation.kind !== 'installed' && (
        <p className="text-2xs break-words text-text-muted select-text">{installation.message}</p>
      )}
      <div className="flex flex-wrap gap-2">
        <Button size="xs" variant="secondary" disabled={refreshing} onClick={refresh}>
          {refreshing && <Spinner size="sm" label={t('settings.ai.refreshing')} />}
          {t('settings.ai.refresh')}
        </Button>
        {availability === 'notInstalled' && status !== null && status.provider.helpUrl !== '' && (
          <Button
            size="xs"
            variant="ghost"
            onClick={() => {
              void bridge.openExternal(status.provider.helpUrl);
            }}
          >
            {t('settings.ai.installHelp', values)}
          </Button>
        )}
      </div>
    </section>
  );
}

function ExecutablePath({ settings, onSave }: AiSettingsSectionProps) {
  const { t } = useTranslation();
  const bridge = useDesktopBridge();
  const choose = async () => {
    const path = await bridge.chooseFile({
      title: t('settings.ai.chooseCliTitle'),
      buttonLabel: t('settings.ai.chooseCliButton'),
    });
    if (path !== null) onSave({ executablePath: path });
  };
  return (
    <div className="flex flex-col gap-1.5">
      <span className="text-xs font-medium text-text">{t('settings.ai.executablePath')}</span>
      <div className="flex items-center gap-2">
        <span
          className="min-w-0 flex-1 truncate text-xs text-text-muted select-text"
          title={settings.executablePath ?? undefined}
        >
          {settings.executablePath ?? t('settings.ai.executableAuto')}
        </span>
        <Button size="xs" variant="secondary" onClick={() => void choose()}>
          {t('settings.ai.chooseCli')}
        </Button>
        {settings.executablePath !== null && (
          <Button
            size="xs"
            variant="ghost"
            onClick={() => {
              onSave({ executablePath: null });
            }}
          >
            {t('settings.ai.resetCli')}
          </Button>
        )}
      </div>
    </div>
  );
}

function TaskDefaults({
  settings,
  status,
  onSave,
}: AiSettingsSectionProps & { readonly status: AiStatus | null }) {
  const { t } = useTranslation();
  const [maxTasks, setMaxTasks] = useState<number | null>(null);
  const models = status?.capabilities.models ?? [];
  const efforts = status?.capabilities.effortLevels ?? [];
  const shownMax = maxTasks ?? settings.maxConcurrentTasks;
  return (
    <section aria-labelledby="settings-ai-tasks" className="flex flex-col gap-3">
      <h3 id="settings-ai-tasks" className="text-sm font-semibold text-text">
        {t('settings.ai.tasksTitle')}
      </h3>
      <Field label={t('settings.ai.defaultModel')}>
        <Select
          value={settings.defaultModel ?? CLI_DEFAULT}
          onValueChange={(value) => {
            onSave({ defaultModel: value === CLI_DEFAULT ? null : value });
          }}
        >
          <SelectTrigger size="xs" className="w-48">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={CLI_DEFAULT}>{t('settings.ai.cliDefault')}</SelectItem>
            {settings.defaultModel !== null &&
              !models.some((model) => model.id === settings.defaultModel) && (
                <SelectItem value={settings.defaultModel}>{settings.defaultModel}</SelectItem>
              )}
            {models.map((model) => (
              <SelectItem key={model.id} value={model.id}>
                {model.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </Field>
      <Field label={t('settings.ai.defaultEffort')}>
        <Select
          value={settings.defaultEffort ?? CLI_DEFAULT}
          onValueChange={(value) => {
            const effort = efforts.find((candidate) => candidate === value);
            onSave({ defaultEffort: effort ?? null });
          }}
        >
          <SelectTrigger size="xs" className="w-48">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={CLI_DEFAULT}>{t('settings.ai.cliDefault')}</SelectItem>
            {efforts.map((effort: EffortLevel) => (
              <SelectItem key={effort} value={effort}>
                {t(`settings.ai.effort.${effort}`)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </Field>
      <div className="flex flex-col gap-1.5">
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium text-text">{t('settings.ai.maxConcurrent')}</span>
          <span className="text-text-muted tabular-nums">{shownMax}</span>
        </div>
        <Slider
          thumbLabels={[t('settings.ai.maxConcurrent')]}
          min={MAX_TASKS_MIN}
          max={MAX_TASKS_MAX}
          step={1}
          value={[shownMax]}
          onValueChange={([value]) => {
            if (value !== undefined) setMaxTasks(value);
          }}
          onValueCommit={([value]) => {
            setMaxTasks(null);
            if (value !== undefined) onSave({ maxConcurrentTasks: value });
          }}
        />
        <p className="text-2xs text-text-muted">{t('settings.ai.maxConcurrentDescription')}</p>
      </div>
    </section>
  );
}
