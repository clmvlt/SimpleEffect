import type { FrameRate } from '@se/protocol';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
  Textarea,
  cn,
} from '@se/ui';
import { useRef, useState, type KeyboardEvent } from 'react';
import { useTranslation } from 'react-i18next';

import { formatTimecode } from '../../../shared/time';
import type { CaptionCommands } from '../hooks/use-caption-commands';
import { captionKeyAction, splitWordIndex } from '../services/caption-editing';
import type { CaptionRow } from '../services/caption-rows';

/** Props of {@link CaptionRowItem}. */
export interface CaptionRowItemProps {
  readonly row: CaptionRow;
  readonly frameRate: FrameRate;
  /** Under the playhead. */
  readonly current: boolean;
  readonly selected: boolean;
  /** The caption before it on its track (Retour arrière at the start merges with it). */
  readonly previous: CaptionRow | undefined;
  readonly commands: CaptionCommands;
  /** A click (`additive`: Ctrl/⌘ or Maj held). */
  readonly onPick: (additive: boolean) => void;
}

/**
 * One caption: its start timecode and words (emphasised words marked; a word's menu puts it
 * forward or back). A double click edits the text: Entrée splits at the caret, Retour arrière at
 * the very start merges with the previous caption, Ctrl/⌘+Entrée or leaving the field commits,
 * Échap cancels.
 */
export function CaptionRowItem({
  row,
  frameRate,
  current,
  selected,
  previous,
  commands,
  onPick,
}: CaptionRowItemProps) {
  const { t } = useTranslation();
  const [draft, setDraftState] = useState<string | null>(null);
  // The field may blur after Entrée already committed: the ref commits a draft only once.
  const draftRef = useRef<string | null>(null);
  const setDraft = (text: string | null) => {
    draftRef.current = text;
    setDraftState(text);
  };
  const [wordMenu, setWordMenu] = useState<number | null>(null);
  /** Commits the draft when it changed; `false` when the core refused it. */
  const commitDraft = async (): Promise<boolean> => {
    const text = draftRef.current;
    setDraft(null);
    if (text === null || text.trim() === row.text || text.trim() === '') return true;
    return commands.editText(row.itemId, text.trim());
  };
  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    const field = event.currentTarget;
    const action = captionKeyAction(event, {
      start: field.selectionStart,
      end: field.selectionEnd,
    });
    if (action === null) return;
    event.preventDefault();
    if (action === 'cancel') {
      setDraft(null);
      return;
    }
    if (action === 'commit') {
      void commitDraft();
      return;
    }
    if (action === 'split') {
      const index = splitWordIndex(field.value, field.selectionStart);
      if (index === null) return;
      void commitDraft().then((ok) => (ok ? commands.split(row.itemId, index) : undefined));
      return;
    }
    if (previous === undefined) return;
    void commitDraft().then((ok) =>
      ok ? commands.merge([previous.itemId, row.itemId]) : undefined,
    );
  };
  return (
    <li
      data-caption-id={row.itemId}
      data-current={current ? '' : undefined}
      aria-selected={selected}
      className={cn(
        'flex gap-2 rounded-sm px-2 py-1 text-sm',
        current && 'bg-accent-subtle',
        selected && 'ring-1 ring-accent',
      )}
      onClick={(event) => {
        if (draft === null) onPick(event.ctrlKey || event.metaKey || event.shiftKey);
      }}
      onDoubleClick={() => {
        setDraft(row.text);
      }}
    >
      <span className="w-20 shrink-0 font-mono text-xs leading-5 text-text-muted tabular-nums">
        {formatTimecode(row.start, frameRate)}
      </span>
      {draft === null ? (
        <ContextMenu
          onOpenChange={(open) => {
            if (!open) setWordMenu(null);
          }}
        >
          <ContextMenuTrigger asChild>
            <span className="min-w-0 flex-1 leading-5">
              {row.words.map((word, index) => (
                <span
                  // Words of one caption keep their order: the index identifies them.
                  key={index}
                  data-word-index={index}
                  className={cn(word.emphasis && 'font-semibold text-accent underline')}
                  onContextMenu={() => {
                    setWordMenu(index);
                  }}
                >
                  {index > 0 ? ' ' : ''}
                  {word.text}
                </span>
              ))}
            </span>
          </ContextMenuTrigger>
          <ContextMenuContent>
            <ContextMenuItem
              disabled={wordMenu === null}
              onSelect={() => {
                if (wordMenu === null) return;
                const emphasised = row.words[wordMenu]?.emphasis ?? false;
                void commands.setEmphasis(row.itemId, [wordMenu], !emphasised);
              }}
            >
              {t(
                wordMenu !== null && row.words[wordMenu]?.emphasis === true
                  ? 'editor.captions.unemphasize'
                  : 'editor.captions.emphasize',
              )}
            </ContextMenuItem>
            <ContextMenuItem
              onSelect={() => {
                setDraft(row.text);
              }}
            >
              {t('editor.captions.editText')}
            </ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
      ) : (
        <Textarea
          autoFocus
          rows={2}
          value={draft}
          aria-label={t('editor.captions.textLabel')}
          className="min-h-0 flex-1 text-sm"
          onChange={(event) => {
            setDraft(event.currentTarget.value);
          }}
          onKeyDown={onKeyDown}
          onBlur={() => {
            void commitDraft();
          }}
        />
      )}
    </li>
  );
}
