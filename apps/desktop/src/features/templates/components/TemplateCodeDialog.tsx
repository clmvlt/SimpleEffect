import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { codeLines, type CodeTokenKind } from '../services/code-tokens';
import { closeTemplateCode, useCodeDialogTemplate } from '../store/code-dialog-store';

/** Colour of each kind of token (design tokens only). */
const TOKEN_CLASSES: Record<CodeTokenKind, string> = {
  keyword: 'text-ai',
  string: 'text-success',
  number: 'text-warning',
  comment: 'text-text-subtle italic',
  punctuation: 'text-text-muted',
  plain: 'text-text',
};

/**
 * « Voir le code »: the TypeScript module of a motion template, read only, coloured, with line
 * numbers (problems reported to agents name these lines). Opened through `openTemplateCode`;
 * mounted once in the editor.
 */
export function TemplateCodeDialog() {
  const { t } = useTranslation();
  const templateId = useCodeDialogTemplate();
  const template = useProjectSelector((state) =>
    templateId === null
      ? undefined
      : state.mirror?.project.templates.find((candidate) => candidate.id === templateId),
  );
  const lines = template === undefined ? [] : codeLines(template.source);
  return (
    <Dialog
      open={template !== undefined}
      onOpenChange={(open) => {
        if (!open) closeTemplateCode();
      }}
    >
      <DialogContent size="lg" closeLabel={t('common.close')} className="max-h-[85vh]">
        <DialogHeader>
          <DialogTitle>
            {t('editor.templates.code.title', {
              name: template?.name ?? '',
              version: template?.version ?? 1,
            })}
          </DialogTitle>
          <DialogDescription>{t('editor.templates.code.description')}</DialogDescription>
        </DialogHeader>
        <div
          data-template-code
          className="timecode min-h-0 overflow-auto rounded-sm border border-border bg-input py-2 text-xs leading-relaxed select-text"
        >
          {lines.map((tokens, index) => (
            <div key={index} className="flex whitespace-pre">
              <span className="w-10 shrink-0 pr-3 text-right text-text-subtle select-none">
                {index + 1}
              </span>
              <code>
                {tokens.map((token, position) => (
                  <span key={position} className={TOKEN_CLASSES[token.kind]}>
                    {token.text}
                  </span>
                ))}
              </code>
            </div>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
