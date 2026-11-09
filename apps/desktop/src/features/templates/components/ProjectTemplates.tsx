import type { MotionTemplate } from '@se/protocol';
import { Badge, EmptyState, IconButton, Tooltip, toast } from '@se/ui';
import { Code, Plus, Trash2, WandSparkles } from '@se/ui/icons';
import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectSelector } from '../../../shared/project-store';
import { showSidePanelTab } from '../../../shared/side-panel';
import { playheadStore, playheadTimeOf, selectItems, useActiveSequence } from '../../timeline';
import { templatesApi } from '../api';
import { PREVIEW_HEIGHT, PREVIEW_WIDTH, useTemplatePreview } from '../hooks/use-template-preview';
import { openTemplateCode } from '../store/code-dialog-store';

/**
 * « Modèles du projet » (prompt 24): the motion templates agents wrote in this project, each with
 * a preview of its default look (animated on hover), « Insérer » at the playhead of the active
 * sequence, « Voir le code », and « Supprimer » for templates no item uses; an empty state when the
 * project has none (agents write them).
 */
export function ProjectTemplates() {
  const { t } = useTranslation();
  const templates = useProjectSelector((state) => state.mirror?.project.templates);
  const items = useProjectSelector((state) => state.mirror?.items);
  if (templates === undefined || templates.length === 0) {
    return (
      <EmptyState
        icon={WandSparkles}
        title={t('editor.templates.list.emptyTitle')}
        description={t('editor.templates.list.emptyDescription')}
      />
    );
  }
  const used = new Set<string>();
  for (const item of Object.values(items ?? {})) {
    if (item.kind.kind === 'motionTemplate') used.add(item.kind.templateId);
  }
  return (
    <section aria-labelledby="project-templates" className="flex flex-col gap-2 p-2">
      <h3 id="project-templates" className="text-xs font-medium text-text-muted">
        {t('editor.templates.list.title')}
      </h3>
      <ul className="flex flex-col gap-2">
        {templates.map((template) => (
          <TemplateCard key={template.id} template={template} used={used.has(template.id)} />
        ))}
      </ul>
    </section>
  );
}

function TemplateCard({
  template,
  used,
}: {
  readonly template: MotionTemplate;
  readonly used: boolean;
}) {
  const { t } = useTranslation();
  const client = useCoreClient();
  const projectId = useProjectSelector((state) => state.mirror?.projectId);
  const sequence = useActiveSequence();
  const canvas = useRef<HTMLCanvasElement | null>(null);
  const [hovered, setHovered] = useState(false);
  useTemplatePreview(canvas, template, hovered);
  const api = projectId === undefined ? null : templatesApi(client, projectId);
  const fail = (error: unknown) => {
    toast.error(t(errorMessageKey(error)));
  };
  const insert = () => {
    if (api === null || sequence === null) return;
    const at = playheadTimeOf(playheadStore.getState(), sequence.id);
    api
      .insert(sequence.id, template.id, at, template.defaultDuration)
      .then((placed) => {
        selectItems([placed.itemId]);
        showSidePanelTab('inspector');
      })
      .catch(fail);
  };
  return (
    <li
      data-template-id={template.id}
      className="flex gap-2 rounded-md border border-border bg-panel-raised p-1.5"
      onPointerEnter={() => {
        setHovered(true);
      }}
      onPointerLeave={() => {
        setHovered(false);
      }}
    >
      <canvas
        ref={canvas}
        width={PREVIEW_WIDTH}
        height={PREVIEW_HEIGHT}
        aria-label={t('editor.templates.list.preview', { name: template.name })}
        className="h-[54px] w-24 shrink-0 rounded-sm bg-bg"
      />
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <div className="flex items-center gap-1">
          <span className="truncate text-xs font-medium">{template.name}</span>
          <Badge size="sm">
            {t('editor.templates.list.version', { version: template.version })}
          </Badge>
        </div>
        {template.description !== '' && (
          <p className="line-clamp-2 text-2xs text-text-muted">{template.description}</p>
        )}
        <div className="mt-auto flex items-center gap-0.5">
          <IconButton
            size="xs"
            icon={Plus}
            label={t('editor.templates.list.insert')}
            disabled={api === null || sequence === null}
            onClick={insert}
          />
          <IconButton
            size="xs"
            icon={Code}
            label={t('editor.templates.list.code')}
            onClick={() => {
              openTemplateCode(template.id);
            }}
          />
          <Tooltip
            content={used ? t('editor.templates.list.inUse') : t('editor.templates.list.remove')}
          >
            <span>
              <IconButton
                size="xs"
                icon={Trash2}
                label={t('editor.templates.list.remove')}
                disabled={used || api === null}
                onClick={() => {
                  api?.remove(template.id).catch(fail);
                }}
              />
            </span>
          </Tooltip>
        </div>
      </div>
    </li>
  );
}
