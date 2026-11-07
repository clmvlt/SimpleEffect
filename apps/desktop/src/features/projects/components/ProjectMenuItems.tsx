import type { RecentProject } from '@se/protocol';
import {
  ContextMenuItem,
  ContextMenuSeparator,
  DropdownMenuItem,
  DropdownMenuSeparator,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

/** What a project card's menus can do. */
export interface ProjectMenuHandlers {
  readonly onOpen: (project: RecentProject) => void;
  readonly onRename: (project: RecentProject) => void;
  readonly onDuplicate: (project: RecentProject) => void;
  readonly onShowInFolder: (project: RecentProject) => void;
  readonly onTrash: (project: RecentProject) => void;
}

/** Props of {@link ProjectMenuItems}. */
export interface ProjectMenuItemsProps extends ProjectMenuHandlers {
  readonly project: RecentProject;
  /** The context menu (right click) or the dropdown (« ⋯ » button) renders the same items. */
  readonly menu: 'context' | 'dropdown';
}

/** The items of a project's menus. A missing folder can only be trashed (forgotten). */
export function ProjectMenuItems({ project, menu, ...handlers }: ProjectMenuItemsProps) {
  const { t } = useTranslation();
  const Item = menu === 'context' ? ContextMenuItem : DropdownMenuItem;
  const Separator = menu === 'context' ? ContextMenuSeparator : DropdownMenuSeparator;
  const missing = !project.available;
  return (
    <>
      <Item
        disabled={missing}
        onSelect={() => {
          handlers.onOpen(project);
        }}
      >
        {t('projects.actions.open')}
      </Item>
      <Item
        disabled={missing}
        onSelect={() => {
          handlers.onRename(project);
        }}
      >
        {t('projects.actions.rename')}
      </Item>
      <Item
        disabled={missing}
        onSelect={() => {
          handlers.onDuplicate(project);
        }}
      >
        {t('projects.actions.duplicate')}
      </Item>
      <Item
        disabled={missing}
        onSelect={() => {
          handlers.onShowInFolder(project);
        }}
      >
        {t('projects.actions.showInFolder')}
      </Item>
      <Separator />
      <Item
        variant="danger"
        onSelect={() => {
          handlers.onTrash(project);
        }}
      >
        {t('projects.actions.trash')}
      </Item>
    </>
  );
}
