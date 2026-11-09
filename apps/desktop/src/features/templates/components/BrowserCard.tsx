import type { MotionTemplate } from '@se/protocol';
import {
  Badge,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  IconButton,
} from '@se/ui';
import { Ellipsis, Plus, Sparkles } from '@se/ui/icons';
import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { endTemplateDrag, startTemplateDrag } from '../../../shared/drag';
import type { BrowserActions } from '../hooks/use-browser-actions';
import { PREVIEW_HEIGHT, PREVIEW_WIDTH, useTemplatePreview } from '../hooks/use-template-preview';
import type { BrowserEntry, LibraryEntry, ProjectEntry } from '../services/browser-entries';
import { CaptionStyleSample } from './CaptionStyleSample';

/**
 * One template of the browser: a preview (the template's default look, animated on hover; a
 * sample line for caption styles), its name, where it lives, its version, an update offered by
 * the library, « Insérer » (or « Appliquer ») and a menu of the other actions. Motion templates
 * drag to the timeline.
 */
export function BrowserCard({
  entry,
  actions,
}: {
  readonly entry: BrowserEntry;
  readonly actions: BrowserActions;
}) {
  const { t } = useTranslation();
  const [hovered, setHovered] = useState(false);
  const template = entry.template;
  const isLibrary = entry.kind === 'library';
  const version = isLibrary ? entry.entry.version : entry.template.version;
  const update = entry.kind === 'project' ? entry.update : null;
  return (
    <li
      data-browser-entry={entry.key}
      draggable={template !== null}
      onDragStart={(event) => {
        if (template === null) return;
        startTemplateDrag(event.dataTransfer, {
          source: entry.kind,
          id: entry.kind === 'library' ? entry.entry.id : entry.template.id,
          name: entry.name,
          duration: template.defaultDuration,
        });
      }}
      onDragEnd={endTemplateDrag}
      onPointerEnter={() => {
        setHovered(true);
      }}
      onPointerLeave={() => {
        setHovered(false);
      }}
      className="flex min-w-0 flex-col gap-1 rounded-md border border-border bg-panel-raised p-1"
    >
      <div className="relative aspect-video overflow-hidden rounded-sm bg-bg">
        {template !== null ? (
          <TemplatePreview template={template} playing={hovered} name={entry.name} />
        ) : (
          entry.kind === 'library' &&
          entry.captionStyle !== null && <CaptionStyleSample style={entry.captionStyle} />
        )}
        <div className="absolute top-1 left-1 flex gap-0.5">
          <Badge size="sm" variant={isLibrary ? 'accent' : 'neutral'}>
            {isLibrary
              ? t('editor.templates.browser.library')
              : t('editor.templates.browser.project')}
          </Badge>
          <Badge size="sm">{t('editor.templates.list.version', { version })}</Badge>
        </div>
        {update !== null && (
          <span className="absolute right-1 bottom-1">
            <Badge size="sm" variant="warning">
              {t('editor.templates.browser.updateBadge', { version: update.version })}
            </Badge>
          </span>
        )}
      </div>
      <div className="flex min-w-0 items-center gap-0.5">
        <span className="min-w-0 flex-1 truncate text-xs font-medium" title={entry.description}>
          {entry.name}
        </span>
        <PrimaryAction entry={entry} actions={actions} />
        <CardMenu entry={entry} actions={actions} />
      </div>
    </li>
  );
}

function TemplatePreview({
  template,
  playing,
  name,
}: {
  readonly template: MotionTemplate;
  readonly playing: boolean;
  readonly name: string;
}) {
  const { t } = useTranslation();
  const canvas = useRef<HTMLCanvasElement | null>(null);
  useTemplatePreview(canvas, template, playing);
  return (
    <canvas
      ref={canvas}
      width={PREVIEW_WIDTH}
      height={PREVIEW_HEIGHT}
      aria-label={t('editor.templates.list.preview', { name })}
      className="size-full"
    />
  );
}

function PrimaryAction({
  entry,
  actions,
}: {
  readonly entry: BrowserEntry;
  readonly actions: BrowserActions;
}) {
  const { t } = useTranslation();
  if (entry.kind === 'library' && entry.captionStyle !== null) {
    return (
      <IconButton
        size="xs"
        icon={Sparkles}
        label={t('editor.templates.browser.applyStyle')}
        onClick={() => {
          actions.applyCaptionStyle(entry.entry);
        }}
      />
    );
  }
  return (
    <IconButton
      size="xs"
      icon={Plus}
      label={t('editor.templates.list.insert')}
      disabled={!actions.ready}
      onClick={() => {
        if (entry.kind === 'project') actions.insertProject(entry.template);
        else actions.insertLibrary(entry.entry);
      }}
    />
  );
}

function CardMenu({
  entry,
  actions,
}: {
  readonly entry: BrowserEntry;
  readonly actions: BrowserActions;
}) {
  const { t } = useTranslation();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <IconButton
          size="xs"
          icon={Ellipsis}
          label={t('editor.templates.browser.more', { name: entry.name })}
        />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {entry.kind === 'project' ? (
          <ProjectItems entry={entry} actions={actions} />
        ) : (
          <LibraryItems entry={entry} actions={actions} />
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function ProjectItems({
  entry,
  actions,
}: {
  readonly entry: ProjectEntry;
  readonly actions: BrowserActions;
}) {
  const { t } = useTranslation();
  const { template, update } = entry;
  return (
    <>
      {update !== null && (
        <DropdownMenuItem
          onSelect={() => {
            actions.update(template, update);
          }}
        >
          {t('editor.templates.browser.update', { version: update.version })}
        </DropdownMenuItem>
      )}
      <DropdownMenuItem
        disabled={!actions.ready}
        onSelect={() => {
          actions.saveTemplate(template);
        }}
      >
        {template.library === null
          ? t('editor.templates.browser.saveToLibrary')
          : t('editor.templates.browser.saveVersion')}
      </DropdownMenuItem>
      <DropdownMenuItem
        onSelect={() => {
          actions.viewCode(template);
        }}
      >
        {t('editor.templates.list.code')}
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem
        disabled={entry.used}
        onSelect={() => {
          actions.removeProject(template);
        }}
      >
        {entry.used ? t('editor.templates.list.inUse') : t('editor.templates.list.remove')}
      </DropdownMenuItem>
    </>
  );
}

function LibraryItems({
  entry,
  actions,
}: {
  readonly entry: LibraryEntry;
  readonly actions: BrowserActions;
}) {
  const { t } = useTranslation();
  return (
    <>
      <DropdownMenuItem
        onSelect={() => {
          actions.exportEntry(entry.entry);
        }}
      >
        {t('editor.templates.browser.export')}
      </DropdownMenuItem>
      <DropdownMenuItem
        onSelect={() => {
          actions.renameEntry(entry.entry);
        }}
      >
        {t('editor.templates.browser.rename')}
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem
        onSelect={() => {
          actions.removeEntry(entry.entry);
        }}
      >
        {t('editor.templates.browser.removeFromLibrary')}
      </DropdownMenuItem>
    </>
  );
}
