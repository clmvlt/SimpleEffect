import type { LibraryBrandKit } from '@se/protocol';
import {
  Badge,
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  Input,
  Separator,
} from '@se/ui';
import { Ellipsis } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { useBrandKitActions } from '../hooks/use-brand-kit-actions';
import { isDirty, withAddedLogos } from '../services/kit-draft';
import { BrandKitPreview } from './BrandKitPreview';
import { KitCaptionFields } from './KitCaptionFields';
import { KitFavorites } from './KitFavorites';
import { KitIdentityFields } from './KitIdentityFields';
import { KitLogos } from './KitLogos';
import { KitTextStyleFields } from './KitTextStyleFields';

/**
 * The editor of one library kit: a draft of every field (name, fonts, palette, logos, the looks
 * of new texts and captions, favourite templates) beside a live preview, saved as a new version
 * by « Enregistrer le kit » (open projects using the kit follow at once). Logos are added on the
 * spot (copied into the kit), the rest waits for the save.
 */
export function BrandKitEditor({
  kit,
  projectId,
  onDuplicate,
}: {
  readonly kit: LibraryBrandKit;
  readonly projectId: string | null;
  readonly onDuplicate: () => void;
}) {
  const { t } = useTranslation();
  const actions = useBrandKitActions();
  const [saved, setSaved] = useState(kit);
  const [draft, setDraft] = useState(kit);
  const [busy, setBusy] = useState(false);
  const usedSource = useProjectSelector((state) => state.mirror?.project.brandKit?.source ?? null);
  const dirty = isDirty(draft, saved);
  const usedHere = usedSource?.kitId === kit.id;
  const save = () => {
    setBusy(true);
    void actions
      .save(draft)
      .then((result) => {
        if (result !== null) {
          setSaved(result);
          setDraft(result);
        }
      })
      .finally(() => {
        setBusy(false);
      });
  };
  const addLogos = () => {
    void actions.addLogos(saved).then((result) => {
      if (result === null) return;
      setSaved(result);
      setDraft((current) => withAddedLogos(current, result));
    });
  };
  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-2">
        <Input
          size="sm"
          value={draft.name}
          aria-label={t('brandKit.settings.name')}
          className="min-w-0 flex-1"
          onChange={(event) => {
            setDraft({ ...draft, name: event.target.value });
          }}
        />
        <Badge size="sm">{t('brandKit.settings.version', { version: saved.version })}</Badge>
        {projectId !== null &&
          (usedHere ? (
            <Badge size="sm" variant="accent">
              {t('brandKit.settings.usedInProject')}
            </Badge>
          ) : (
            <Button
              size="xs"
              variant="secondary"
              disabled={dirty}
              onClick={() => {
                void actions.setProjectKit(projectId, saved);
              }}
            >
              {t('brandKit.settings.useInProject')}
            </Button>
          ))}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <IconButton
              size="xs"
              icon={Ellipsis}
              label={t('brandKit.settings.actions', { name: saved.name })}
            />
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onSelect={onDuplicate}>
              {t('brandKit.settings.duplicate')}
            </DropdownMenuItem>
            <DropdownMenuItem
              variant="danger"
              onSelect={() => {
                void actions.remove(saved);
              }}
            >
              {t('brandKit.settings.remove')}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
      <div className="grid grid-cols-[minmax(0,1fr)_14rem] gap-4">
        <div className="flex max-h-[50vh] min-w-0 flex-col gap-3 overflow-auto pr-1">
          <KitIdentityFields kit={draft} onChange={setDraft} />
          <Separator />
          <KitLogos kit={draft} onChange={setDraft} onAdd={addLogos} />
          <Separator />
          <KitTextStyleFields kit={draft} onChange={setDraft} />
          <Separator />
          <KitCaptionFields kit={draft} onChange={setDraft} />
          <Separator />
          <KitFavorites kit={draft} onChange={setDraft} />
        </div>
        <BrandKitPreview kit={draft} />
      </div>
      <div className="flex items-center justify-end gap-2">
        {dirty && (
          <span className="mr-auto text-xs text-text-muted">{t('brandKit.settings.dirty')}</span>
        )}
        <Button
          variant="ghost"
          size="sm"
          disabled={!dirty}
          onClick={() => {
            setDraft(saved);
          }}
        >
          {t('brandKit.settings.discard')}
        </Button>
        <Button size="sm" disabled={!dirty || busy} loading={busy} onClick={save}>
          {t('brandKit.settings.save')}
        </Button>
      </div>
    </div>
  );
}
