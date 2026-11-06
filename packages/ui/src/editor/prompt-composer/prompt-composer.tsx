import { ArrowUp, X } from 'lucide-react';
import type { KeyboardEvent, ReactNode } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';
import { Icon } from '../../primitives/icon';
import { IconButton } from '../../primitives/icon-button';

/** A scope attached to the prompt (selected clips, a time range, a track…), shown as a chip. */
export interface PromptScope {
  readonly id: string;
  readonly label: string;
}

/** Props of {@link PromptComposer}. */
export interface PromptComposerProps {
  value: string;
  onValueChange: (value: string) => void;
  /** Called with the text on Ctrl+Enter / ⌘+Enter or the send button, when it is not blank. */
  onSubmit: (value: string) => void;
  /** Scopes shown as removable chips above the text. */
  scopes?: readonly PromptScope[] | undefined;
  /** Removes a scope (chip button, or Backspace in an empty field for the last one). */
  onRemoveScope?: ((id: string) => void) | undefined;
  /** Accessible name of the text field (default English « Prompt »). */
  label?: string | undefined;
  placeholder?: string | undefined;
  /** Accessible name of the send button (default English « Send »). */
  submitLabel?: string | undefined;
  /** Accessible name of a chip's remove button (default English « Remove <label> »). */
  removeScopeLabel?: ((scopeLabel: string) => string) | undefined;
  /** Model selector slot, in the bottom bar. */
  modelSlot?: ReactNode | undefined;
  /** Effort selector slot, in the bottom bar. */
  effortSlot?: ReactNode | undefined;
  /** Hint next to the send button (usually a `KeyboardShortcut` for Mod+Enter). */
  submitHint?: ReactNode | undefined;
  /** A task is being started: sending is blocked and the button shows a spinner. */
  busy?: boolean | undefined;
  disabled?: boolean | undefined;
  className?: string | undefined;
}

/**
 * The AI request box (visual only): scope chips, an auto-growing text field, model and effort
 * slots and a send button. Ctrl+Enter (⌘+Enter on macOS) sends; Enter inserts a new line.
 */
export function PromptComposer({
  value,
  onValueChange,
  onSubmit,
  scopes = [],
  onRemoveScope,
  label = 'Prompt',
  placeholder,
  submitLabel = 'Send',
  removeScopeLabel = (scopeLabel) => `Remove ${scopeLabel}`,
  modelSlot,
  effortSlot,
  submitHint,
  busy = false,
  disabled = false,
  className,
}: PromptComposerProps) {
  const canSubmit = !disabled && !busy && value.trim() !== '';
  const submit = () => {
    if (canSubmit) {
      onSubmit(value);
    }
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      submit();
      return;
    }
    const lastScope = scopes.at(-1);
    if (
      event.key === 'Backspace' &&
      value === '' &&
      lastScope !== undefined &&
      onRemoveScope !== undefined
    ) {
      event.preventDefault();
      onRemoveScope(lastScope.id);
    }
  };

  return (
    <div
      className={cn(
        'flex flex-col gap-2 rounded-lg border border-border bg-input p-2',
        'transition-colors duration-(--duration-fast) focus-within:border-focus-ring',
        disabled && 'opacity-50',
        className,
      )}
    >
      {scopes.length > 0 && (
        <ul className="flex flex-wrap gap-1">
          {scopes.map((scope) => (
            <li
              key={scope.id}
              className="inline-flex h-5 items-center gap-0.5 rounded-sm bg-ai-muted pr-0.5 pl-1.5 text-xs text-ai-strong"
            >
              <span className="max-w-48 truncate">{scope.label}</span>
              {onRemoveScope !== undefined && (
                <button
                  type="button"
                  aria-label={removeScopeLabel(scope.label)}
                  disabled={disabled}
                  onClick={() => {
                    onRemoveScope(scope.id);
                  }}
                  className={cn(
                    'inline-flex size-4 items-center justify-center rounded-xs hover:bg-hover',
                    focusRing,
                  )}
                >
                  <Icon icon={X} size="xs" />
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
      <textarea
        aria-label={label}
        placeholder={placeholder}
        disabled={disabled}
        rows={2}
        value={value}
        onChange={(event) => {
          onValueChange(event.target.value);
        }}
        onKeyDown={handleKeyDown}
        className="max-h-48 min-h-10 w-full resize-none bg-transparent px-1 text-sm text-text outline-none [field-sizing:content] placeholder:text-text-subtle"
      />
      <div className="flex items-center gap-1">
        {modelSlot}
        {effortSlot}
        <div className="ml-auto flex items-center gap-2">
          {submitHint !== undefined && (
            <span className="text-2xs text-text-subtle">{submitHint}</span>
          )}
          <IconButton
            icon={ArrowUp}
            label={submitLabel}
            variant="primary"
            size="xs"
            loading={busy}
            disabled={!canSubmit}
            onClick={submit}
          />
        </div>
      </div>
    </div>
  );
}
