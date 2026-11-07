import { Input } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

/** Props of {@link TrackName}. */
export interface TrackNameProps {
  readonly name: string;
  /** Whether the name is being edited (double-click or the « Renommer » menu item). */
  readonly editing: boolean;
  readonly onEditingChange: (editing: boolean) => void;
  /** Called with a new, non-empty, trimmed name. */
  readonly onRename: (name: string) => void;
}

/**
 * A track's name, renamed inline: double-click (or « Renommer ») turns it into a field; Enter or
 * leaving the field renames, Escape cancels.
 */
export function TrackName({ name, editing, onEditingChange, onRename }: TrackNameProps) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState(name);
  if (!editing) {
    return (
      <span
        className="min-w-0 flex-1 truncate text-xs font-medium text-text"
        title={name}
        onDoubleClick={() => {
          setDraft(name);
          onEditingChange(true);
        }}
      >
        {name}
      </span>
    );
  }
  const finish = (save: boolean) => {
    const trimmed = draft.trim();
    if (save && trimmed !== '' && trimmed !== name) onRename(trimmed);
    onEditingChange(false);
  };
  return (
    <Input
      autoFocus
      size="xs"
      aria-label={t('editor.timeline.track.nameField')}
      className="min-w-0 flex-1"
      value={draft}
      onChange={(event) => {
        setDraft(event.target.value);
      }}
      onFocus={(event) => {
        event.target.select();
      }}
      onBlur={() => {
        finish(true);
      }}
      onKeyDown={(event) => {
        if (event.key === 'Enter') finish(true);
        if (event.key === 'Escape') finish(false);
      }}
    />
  );
}
