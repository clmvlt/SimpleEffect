import type { SequenceEntity, SequenceId } from '@se/protocol';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  IconButton,
  Input,
  Tabs,
  TabsList,
  TabsTrigger,
  Tooltip,
} from '@se/ui';
import { Clapperboard, Ellipsis } from '@se/ui/icons';
import { useState, type ComponentType, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

/** What the menu of a sequence tab does. */
export type SequenceAction = 'rename' | 'format' | 'version' | 'duplicate' | 'remove';

/** Props of {@link SequenceTabs}. */
export interface SequenceTabsProps {
  readonly sequences: readonly SequenceEntity[];
  readonly activeId: string;
  readonly onSelect: (sequenceId: string) => void;
  readonly onCreate: () => void;
  /** A menu action other than renaming (format, version, duplicate, remove). */
  readonly onAction: (action: Exclude<SequenceAction, 'rename'>, sequenceId: SequenceId) => void;
  /** Called with a new, non-empty, trimmed name. */
  readonly onRename: (sequenceId: SequenceId, name: string) => void;
}

interface MenuParts {
  readonly Item: ComponentType<{
    readonly onSelect: () => void;
    readonly variant?: 'danger';
    readonly disabled?: boolean;
    readonly children: ReactNode;
  }>;
  readonly Separator: ComponentType;
}

const CONTEXT_PARTS: MenuParts = { Item: ContextMenuItem, Separator: ContextMenuSeparator };
const DROPDOWN_PARTS: MenuParts = { Item: DropdownMenuItem, Separator: DropdownMenuSeparator };

/** The items of a sequence's menu (tab context menu and the active sequence's « … » button). */
function SequenceMenuItems({
  parts: { Item, Separator },
  canRemove,
  onAction,
}: {
  readonly parts: MenuParts;
  readonly canRemove: boolean;
  readonly onAction: (action: SequenceAction) => void;
}) {
  const { t } = useTranslation();
  const item = (action: SequenceAction) => (
    <Item
      onSelect={() => {
        onAction(action);
      }}
    >
      {t(`editor.sequences.menu.${action}`)}
    </Item>
  );
  return (
    <>
      {item('rename')}
      {item('format')}
      {item('version')}
      {item('duplicate')}
      <Separator />
      <Item
        variant="danger"
        disabled={!canRemove}
        onSelect={() => {
          onAction('remove');
        }}
      >
        {t('editor.sequences.menu.remove')}
      </Item>
    </>
  );
}

/** A sequence's name turned into a field: Enter or leaving renames, Escape cancels. */
function SequenceNameField({
  name,
  onDone,
}: {
  readonly name: string;
  readonly onDone: (name: string | null) => void;
}) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState(name);
  const finish = (save: boolean) => {
    const trimmed = draft.trim();
    onDone(save && trimmed !== '' && trimmed !== name ? trimmed : null);
  };
  return (
    <Input
      autoFocus
      size="xs"
      aria-label={t('editor.sequences.menu.renameField')}
      className="w-36"
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

/**
 * The sequences of the project as tabs, each with a context menu (rename — or double-click —,
 * format, version, duplicate, remove), the « … » menu of the active one and a « + » to create
 * a sequence. Choosing a tab shows that sequence in the timeline.
 */
export function SequenceTabs({
  sequences,
  activeId,
  onSelect,
  onCreate,
  onAction,
  onRename,
}: SequenceTabsProps) {
  const { t } = useTranslation();
  const [renaming, setRenaming] = useState<SequenceId | null>(null);
  const canRemove = sequences.length > 1;
  const act = (sequenceId: SequenceId) => (action: SequenceAction) => {
    if (action === 'rename') setRenaming(sequenceId);
    else onAction(action, sequenceId);
  };
  return (
    <div className="flex min-w-0 items-center gap-1">
      <Tabs value={activeId} onValueChange={onSelect} className="min-w-0">
        <TabsList variant="underline" size="sm" aria-label={t('editor.timeline.sequences')}>
          {sequences.map((sequence) =>
            renaming === sequence.id ? (
              <SequenceNameField
                key={sequence.id}
                name={sequence.name}
                onDone={(name) => {
                  setRenaming(null);
                  if (name !== null) onRename(sequence.id, name);
                }}
              />
            ) : (
              <ContextMenu key={sequence.id}>
                <ContextMenuTrigger asChild>
                  <TabsTrigger
                    value={sequence.id}
                    className="max-w-40 truncate"
                    onDoubleClick={() => {
                      setRenaming(sequence.id);
                    }}
                  >
                    {sequence.name}
                  </TabsTrigger>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <SequenceMenuItems
                    parts={CONTEXT_PARTS}
                    canRemove={canRemove}
                    onAction={act(sequence.id)}
                  />
                </ContextMenuContent>
              </ContextMenu>
            ),
          )}
        </TabsList>
      </Tabs>
      <DropdownMenu>
        <Tooltip content={t('editor.sequences.menu.label')}>
          <DropdownMenuTrigger asChild>
            <IconButton size="xs" icon={Ellipsis} label={t('editor.sequences.menu.label')} />
          </DropdownMenuTrigger>
        </Tooltip>
        <DropdownMenuContent align="start">
          <SequenceMenuItems
            parts={DROPDOWN_PARTS}
            canRemove={canRemove}
            onAction={act(activeId)}
          />
        </DropdownMenuContent>
      </DropdownMenu>
      <Tooltip content={t('editor.timeline.newSequence')}>
        <IconButton
          size="xs"
          icon={Clapperboard}
          label={t('editor.timeline.newSequence')}
          onClick={onCreate}
        />
      </Tooltip>
    </div>
  );
}
