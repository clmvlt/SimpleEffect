import type { RecentProject } from '@se/protocol';
import {
  Badge,
  ContextMenu,
  ContextMenuContent,
  ContextMenuTrigger,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
  IconButton,
  cn,
  focusRing,
} from '@se/ui';
import { Ellipsis } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { FormatBadge } from './FormatBadge';
import { ProjectMenuItems, type ProjectMenuHandlers } from './ProjectMenuItems';

/** Props of {@link ProjectCard}. */
export interface ProjectCardProps extends ProjectMenuHandlers {
  readonly project: RecentProject;
}

/**
 * A recent project: click (or Enter) opens it; right click or the « ⋯ » button shows rename,
 * duplicate, reveal and trash.
 */
export function ProjectCard({ project, ...handlers }: ProjectCardProps) {
  const { t, i18n } = useTranslation();
  const openedAt = new Intl.DateTimeFormat(i18n.language, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(project.lastOpenedAt));
  return (
    <li className="relative">
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <button
            type="button"
            disabled={!project.available}
            onClick={() => {
              handlers.onOpen(project);
            }}
            className={cn(
              'flex w-full flex-col gap-1.5 rounded-lg border border-border bg-panel p-3 pr-10 text-left',
              'transition-colors duration-(--duration-fast) hover:border-border-strong hover:bg-panel-raised',
              'disabled:cursor-default disabled:opacity-60',
              focusRing,
            )}
          >
            <span className="flex items-center gap-2">
              <span className="truncate text-md font-medium text-text">{project.name}</span>
              <FormatBadge format={project.format} />
              {!project.available && (
                <Badge variant="warning">{t('projects.card.unavailable')}</Badge>
              )}
            </span>
            <span className="text-xs text-text-muted">
              {t('projects.card.openedAt', { date: openedAt })}
            </span>
            <span className="truncate text-xs text-text-subtle" title={project.path}>
              {project.path}
            </span>
          </button>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ProjectMenuItems project={project} menu="context" {...handlers} />
        </ContextMenuContent>
      </ContextMenu>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <IconButton
            icon={Ellipsis}
            size="sm"
            label={t('projects.card.actions', { name: project.name })}
            className="absolute top-2 right-2"
          />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <ProjectMenuItems project={project} menu="dropdown" {...handlers} />
        </DropdownMenuContent>
      </DropdownMenu>
    </li>
  );
}
