import type { Flicks } from '@se/protocol';
import { Button, Icon, PropertyGroup } from '@se/ui';
import { Code, Sparkles } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { openComposer } from '../../../shared/ai-tasks';
import { useProjectSelector } from '../../../shared/project-store';
import { useTemplateIssue } from '../../../shared/templates';
import { usePlayheadTime } from '../../timeline';
import { openTemplateCode } from '../../templates';
import type { Inspection } from '../hooks/use-inspection';
import { useTemplateParams } from '../hooks/use-template-params';
import { templateOf } from '../services/template-params';
import { TemplateParamField } from './TemplateParamField';

/**
 * « Modèle » of selected motion template instances (prompt 24): one control per parameter of the
 * template's schema — texts, numbers and colours with keyframes, switches, options, fonts,
 * images —, the template's failure in the frame on screen, « Voir le code » and « Demander une
 * modification à l'IA ». Shows when every selected item is an instance of the same template.
 */
export function TemplateSection({ inspection }: { readonly inspection: Inspection }) {
  const { t } = useTranslation();
  const templates = useProjectSelector((state) => state.mirror?.project.templates);
  const projectId = useProjectSelector((state) => state.mirror?.projectId ?? '');
  const { items, sequence } = inspection;
  const [first] = items;
  const template = first === undefined ? null : templateOf(first, templates ?? []);
  const playhead = usePlayheadTime(sequence.id);
  const issue = useTemplateIssue(first?.id ?? null);
  const editor = useTemplateParams(
    projectId,
    sequence,
    template ?? EMPTY_TEMPLATE,
    items,
    playhead,
  );
  const sameTemplate = items.every(
    (item) => item.kind.kind === 'motionTemplate' && item.kind.templateId === template?.id,
  );
  if (template === null || first === undefined || !sameTemplate) return null;
  return (
    <PropertyGroup title={t('editor.templates.inspector.title', { name: template.name })}>
      {issue !== null && (
        <p role="alert" className="px-2 pb-1 text-xs break-words text-danger">
          {issue.line === null
            ? t('editor.templates.inspector.failed', { message: issue.message })
            : t('editor.templates.inspector.failedAt', {
                message: issue.message,
                line: issue.line,
              })}
        </p>
      )}
      {template.params.length === 0 && (
        <p className="px-2 pb-1 text-xs text-text-muted">
          {t('editor.templates.inspector.noParams')}
        </p>
      )}
      {template.params.map((param) => (
        <TemplateParamField key={param.name} editor={editor} param={param} />
      ))}
      <div className="flex flex-wrap gap-1 px-2 pt-1 pb-2">
        <Button
          size="xs"
          variant="secondary"
          onClick={() => {
            openTemplateCode(template.id);
          }}
        >
          <Icon icon={Code} size="sm" />
          {t('editor.templates.inspector.code')}
        </Button>
        <Button
          size="xs"
          variant="secondary"
          onClick={() => {
            openComposer(
              first.id,
              t('editor.templates.inspector.askAiDraft', { name: template.name }),
            );
          }}
        >
          <Icon icon={Sparkles} size="sm" />
          {t('editor.templates.inspector.askAi')}
        </Button>
      </div>
    </PropertyGroup>
  );
}

/** Stands in while no template is inspected (the editor is then unused). */
const EMPTY_TEMPLATE = {
  id: '',
  name: '',
  description: '',
  version: 1,
  source: '',
  params: [],
  defaultDuration: 0 as Flicks,
  origin: { kind: 'user' },
} satisfies Parameters<typeof useTemplateParams>[2];
