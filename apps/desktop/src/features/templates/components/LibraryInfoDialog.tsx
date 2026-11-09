import type { LibraryTemplateInfo, TemplateCategory } from '@se/protocol';
import {
  Button,
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Field,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Textarea,
  toast,
} from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectSelector } from '../../../shared/project-store';
import { libraryApi, templatesApi } from '../api';
import { BROWSER_CATEGORIES, parseTags } from '../services/browser-entries';
import {
  closeLibraryDialog,
  useLibraryDialogRequest,
  type LibraryDialogRequest,
} from '../store/library-dialog-store';

/** The template categories a user may file a motion template under. */
const TEMPLATE_CATEGORIES = BROWSER_CATEGORIES.filter(
  (category): category is TemplateCategory => category !== 'captions',
);

/** What the form starts with. */
function initialInfo(request: LibraryDialogRequest): LibraryTemplateInfo {
  switch (request.kind) {
    case 'saveTemplate':
      return {
        name: request.linked?.name ?? request.template.name,
        description: request.linked?.description ?? request.template.description,
        tags: request.linked?.tags ?? [],
        category: request.template.category,
      };
    case 'saveCaptionStyle':
      return { name: request.name, description: '', tags: [] };
    case 'rename': {
      const { content } = request.entry;
      return {
        name: request.entry.name,
        description: request.entry.description,
        tags: request.entry.tags,
        ...(content.kind === 'motionTemplate' ? { category: content.category } : {}),
      };
    }
  }
}

/**
 * « Enregistrer dans la bibliothèque » (a project template or caption style) and « Renommer » (a
 * library entry): name, description, tags and, for motion templates, the category. Mounted once
 * by the editor; opened through the library dialog store.
 */
export function LibraryInfoDialog() {
  const { t } = useTranslation();
  const request = useLibraryDialogRequest();
  const title =
    request?.kind === 'rename'
      ? t('editor.templates.library.renameTitle')
      : request?.kind === 'saveTemplate' && request.linked !== null
        ? t('editor.templates.library.saveVersionTitle', { version: request.linked.version + 1 })
        : t('editor.templates.library.saveTitle');
  return (
    <Dialog
      open={request !== null}
      onOpenChange={(open) => {
        if (!open) closeLibraryDialog();
      }}
    >
      <DialogContent size="sm" closeLabel={t('common.close')}>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        {request !== null && <InfoForm key={keyOf(request)} request={request} />}
      </DialogContent>
    </Dialog>
  );
}

function keyOf(request: LibraryDialogRequest): string {
  switch (request.kind) {
    case 'saveTemplate':
      return `template:${request.template.id}`;
    case 'saveCaptionStyle':
      return `style:${request.styleId}`;
    case 'rename':
      return `entry:${request.entry.id}`;
  }
}

function InfoForm({ request }: { readonly request: LibraryDialogRequest }) {
  const { t } = useTranslation();
  const client = useCoreClient();
  const projectId = useProjectSelector((state) => state.mirror?.projectId ?? null);
  const initial = initialInfo(request);
  const [name, setName] = useState(initial.name);
  const [description, setDescription] = useState(initial.description);
  const [tags, setTags] = useState(initial.tags.join(', '));
  const [category, setCategory] = useState(initial.category);
  const [tried, setTried] = useState(false);
  const [busy, setBusy] = useState(false);
  const empty = name.trim() === '';
  const save = async (info: LibraryTemplateInfo) => {
    switch (request.kind) {
      case 'rename':
        return libraryApi(client).updateInfo(request.entry.id, info);
      case 'saveTemplate':
        if (projectId === null) throw new Error('no open project');
        return templatesApi(client, projectId).saveToLibrary(request.template.id, info);
      case 'saveCaptionStyle':
        if (projectId === null) throw new Error('no open project');
        return templatesApi(client, projectId).saveCaptionStyle(request.styleId, info);
    }
  };
  const submit = () => {
    setTried(true);
    if (empty || busy) return;
    setBusy(true);
    const info: LibraryTemplateInfo = {
      name: name.trim(),
      description: description.trim(),
      tags: [...parseTags(tags)],
      ...(category === undefined ? {} : { category }),
    };
    save(info)
      .then((saved) => {
        toast.success(
          request.kind === 'rename'
            ? t('editor.templates.library.renamed', { name: saved.name })
            : t('editor.templates.library.saved', { name: saved.name, version: saved.version }),
        );
        closeLibraryDialog();
      })
      .catch((error: unknown) => {
        toast.error(t(errorMessageKey(error)));
      })
      .finally(() => {
        setBusy(false);
      });
  };
  return (
    <form
      noValidate
      className="flex flex-col gap-3"
      onSubmit={(event) => {
        event.preventDefault();
        submit();
      }}
    >
      <Field
        label={t('editor.templates.library.name')}
        error={tried && empty ? t('editor.templates.library.nameRequired') : undefined}
      >
        <Input
          autoFocus
          value={name}
          onChange={(event) => {
            setName(event.target.value);
          }}
        />
      </Field>
      <Field label={t('editor.templates.library.description')}>
        <Textarea
          rows={2}
          value={description}
          onChange={(event) => {
            setDescription(event.target.value);
          }}
        />
      </Field>
      {category !== undefined && (
        <Field label={t('editor.templates.library.category')}>
          <Select
            value={category}
            onValueChange={(value) => {
              const chosen = TEMPLATE_CATEGORIES.find((candidate) => candidate === value);
              if (chosen !== undefined) setCategory(chosen);
            }}
          >
            <SelectTrigger size="sm">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {TEMPLATE_CATEGORIES.map((option) => (
                <SelectItem key={option} value={option}>
                  {t(`editor.templates.categories.${option}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
      )}
      <Field
        label={t('editor.templates.library.tags')}
        description={t('editor.templates.library.tagsHint')}
      >
        <Input
          value={tags}
          onChange={(event) => {
            setTags(event.target.value);
          }}
        />
      </Field>
      <DialogFooter>
        <DialogClose asChild>
          <Button variant="secondary">{t('common.cancel')}</Button>
        </DialogClose>
        <Button type="submit" disabled={busy}>
          {request.kind === 'rename' ? t('common.save') : t('editor.templates.library.save')}
        </Button>
      </DialogFooter>
    </form>
  );
}
