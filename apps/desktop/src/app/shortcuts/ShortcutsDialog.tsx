import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  KeyboardShortcut,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import {
  SHORTCUT_ACTION_IDS,
  SHORTCUT_CONTEXTS,
  useShortcutDisplayKeys,
  useShortcutKeyLabels,
  useShortcutRegistry,
  type ShortcutActionId,
  type ShortcutContext,
} from '../../shared/shortcuts';

/** Props of {@link ShortcutsDialog}. */
export interface ShortcutsDialogProps {
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
}

/** One action and its combinations, written the way this OS writes them. */
function ShortcutRow({ action }: { readonly action: ShortcutActionId }) {
  const { t } = useTranslation();
  const registry = useShortcutRegistry();
  const keyLabels = useShortcutKeyLabels();
  const combinations = useShortcutDisplayKeys(action);
  return (
    <div className="flex items-center justify-between gap-4 py-1">
      <dt className="text-sm text-text">{t(`shortcuts.actions.${action}`)}</dt>
      <dd className="flex items-center gap-1 text-xs text-text-subtle">
        {combinations.map((keys, index) => (
          <span key={keys.join('+')} className="flex items-center gap-1">
            {index > 0 && t('shortcuts.or')}
            <KeyboardShortcut keys={keys} platform={registry.platform} keyLabels={keyLabels} />
          </span>
        ))}
      </dd>
    </div>
  );
}

/** The actions listed under a context: those whose first context it is. */
function actionsOf(context: ShortcutContext, registry: ReturnType<typeof useShortcutRegistry>) {
  return SHORTCUT_ACTION_IDS.filter((id) => registry.actions[id].contexts[0] === context);
}

/**
 * « Raccourcis clavier »: every named action with its keys, grouped by the region where it
 * applies (generated from `SHORTCUT_ACTIONS`, so it never drifts from the real bindings).
 */
export function ShortcutsDialog({ open, onOpenChange }: ShortcutsDialogProps) {
  const { t } = useTranslation();
  const registry = useShortcutRegistry();
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="lg" closeLabel={t('common.close')}>
        <DialogHeader>
          <DialogTitle>{t('shortcuts.title')}</DialogTitle>
          <DialogDescription>{t('shortcuts.description')}</DialogDescription>
        </DialogHeader>
        <div className="flex max-h-[60vh] flex-col gap-4 overflow-auto pr-1">
          {SHORTCUT_CONTEXTS.map((context) => {
            const actions = actionsOf(context, registry);
            if (actions.length === 0) return null;
            return (
              <section key={context} aria-labelledby={`shortcuts-${context}`}>
                <h3
                  id={`shortcuts-${context}`}
                  className="mb-1 text-xs font-semibold text-text-muted uppercase"
                >
                  {t(`shortcuts.contexts.${context}`)}
                </h3>
                <dl className="divide-y divide-border">
                  {actions.map((action) => (
                    <ShortcutRow key={action} action={action} />
                  ))}
                </dl>
              </section>
            );
          })}
        </div>
      </DialogContent>
    </Dialog>
  );
}
