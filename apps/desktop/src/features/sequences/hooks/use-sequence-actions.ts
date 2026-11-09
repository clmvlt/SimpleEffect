import type { AutoReframeReport, ItemEntity, ItemId, SequenceId } from '@se/protocol';
import { toast } from '@se/ui';
import type { TFunction } from 'i18next';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { draftToFormat } from '../../../shared/format';
import { useProjectMirrorReader, type ProjectMirror } from '../../../shared/project-store';
import { DEFAULT_TRACKING_SETTINGS } from '../../reframing';
import { sequencesApi } from '../api';
import { itemsLeavingFrame, positionalOrigins, sameOrigins } from '../services/out-of-frame';
import { adaptationOf, evaluableOf, type SequenceFormatValues } from '../services/sequence-format';
import { createCanvasTextMeasurer, type TextMeasurer } from '../services/text-extent';

/** What the sequence actions ask of the timeline. */
export interface SequenceActionHandlers {
  /** Shows a sequence in the timeline (a new copy or version). */
  readonly onActivate: (sequenceId: SequenceId) => void;
  /** Shows `sequenceId` and selects these items (« Sélectionner » of the out-of-frame warning). */
  readonly onSelectItems: (sequenceId: SequenceId, itemIds: readonly ItemId[]) => void;
  /** Measures texts for the out-of-frame check (a canvas measurer by default). */
  readonly measureText?: TextMeasurer;
}

/** The sequence actions of the open project; failures are shown in a toast. */
export interface SequenceActions {
  readonly rename: (sequenceId: SequenceId, name: string) => Promise<void>;
  readonly duplicate: (sequenceId: SequenceId) => Promise<void>;
  readonly changeFormat: (sequenceId: SequenceId, values: SequenceFormatValues) => Promise<boolean>;
  readonly createVersion: (
    sequenceId: SequenceId,
    values: SequenceFormatValues,
  ) => Promise<boolean>;
  readonly remove: (sequenceId: SequenceId) => Promise<void>;
}

/** The evaluable sequence `sequenceId` of `mirror`, or `null`. */
function sequenceIn(mirror: ProjectMirror | null, sequenceId: SequenceId) {
  const sequence = mirror?.sequences[sequenceId];
  if (mirror === null || sequence === undefined) return null;
  return evaluableOf({ sequence, tracks: mirror.tracks, items: mirror.items });
}

/** « « Titre », forme, image » for the warning (three items at most, then « … »). */
function describeItems(
  t: TFunction,
  itemIds: readonly ItemId[],
  items: Readonly<Record<ItemId, ItemEntity>>,
) {
  const names = itemIds.slice(0, 3).map((id) => {
    const kind = items[id]?.kind;
    if (kind?.kind === 'text') {
      const content = kind.content.split('\n')[0] ?? '';
      return t('editor.sequences.leaving.quoted', {
        text: content.length > 24 ? `${content.slice(0, 23)}…` : content,
      });
    }
    return t(`editor.sequences.leaving.kinds.${kind?.kind ?? 'mediaClip'}`);
  });
  return itemIds.length > 3 ? `${names.join(', ')}…` : names.join(', ');
}

/** The sequence actions (rename, format, duplicate, version, remove) with their feedback. */
export function useSequenceActions(handlers: SequenceActionHandlers): SequenceActions {
  const client = useCoreClient();
  const readMirror = useProjectMirrorReader();
  const { t } = useTranslation();
  const measurer = () => handlers.measureText ?? createCanvasTextMeasurer();

  const api = () => {
    const projectId = readMirror()?.projectId;
    if (projectId === undefined) throw new Error('no project is open');
    return sequencesApi(client, projectId);
  };
  const failed = (error: unknown) => {
    console.warn('[sequences]', error);
    toast.error(t(errorMessageKey(error)));
  };
  /** Warns about the items of `sequenceId` the change pushed out of the frame. */
  const warnLeaving = (sequenceId: SequenceId, leaving: readonly ItemId[]) => {
    const mirror = readMirror();
    if (leaving.length === 0 || mirror === null) return;
    toast.warning(t('editor.sequences.leaving.title', { count: leaving.length }), {
      description: describeItems(t, leaving, mirror.items),
      action: {
        label: t('editor.sequences.leaving.select'),
        onClick: () => {
          handlers.onSelectItems(sequenceId, leaving);
        },
      },
    });
  };

  return {
    rename: async (sequenceId, name) => {
      try {
        await api().renameSequence(sequenceId, name);
      } catch (error) {
        failed(error);
      }
    },
    duplicate: async (sequenceId) => {
      const source = readMirror()?.sequences[sequenceId];
      if (source === undefined) return;
      try {
        const name = t('editor.sequences.copyName', { name: source.name });
        const copy = await api().duplicateSequence(sequenceId, name, null, {
          clipReframe: 'keep',
          fitTextWidths: false,
        });
        handlers.onActivate(copy.sequenceId);
      } catch (error) {
        failed(error);
      }
    },
    changeFormat: async (sequenceId, values) => {
      const mirror = readMirror();
      const before = sequenceIn(mirror, sequenceId);
      const from = mirror?.sequences[sequenceId]?.format;
      if (before === null || from === undefined || mirror === null) return false;
      try {
        await api().setSequenceFormat(
          sequenceId,
          draftToFormat(values),
          adaptationOf(values, from),
        );
      } catch (error) {
        failed(error);
        return false;
      }
      const after = sequenceIn(readMirror(), sequenceId);
      if (after !== null) {
        const origins = sameOrigins(after);
        const assets = mirror.assets;
        const change = { before, after, assets, origins, measureText: measurer() };
        warnLeaving(sequenceId, itemsLeavingFrame(change));
      }
      return true;
    },
    createVersion: async (sequenceId, values) => {
      const mirror = readMirror();
      const source = sequenceIn(mirror, sequenceId);
      const from = mirror?.sequences[sequenceId]?.format;
      if (source === null || from === undefined || mirror === null) return false;
      let versionId: SequenceId;
      let reframe: AutoReframeReport | null;
      try {
        ({ sequenceId: versionId, reframe } = await api().duplicateSequence(
          sequenceId,
          values.name,
          draftToFormat(values),
          adaptationOf(values, from),
          values.autoReframe ? DEFAULT_TRACKING_SETTINGS : undefined,
        ));
      } catch (error) {
        failed(error);
        return false;
      }
      handlers.onActivate(versionId);
      toast.success(t('editor.sequences.versionCreated', { name: values.name }), {
        description: t(
          reframe !== null && reframe.waitingAssetIds.length > 0
            ? 'editor.sequences.reframing'
            : 'editor.sequences.independent',
        ),
      });
      const version = sequenceIn(readMirror(), versionId);
      if (version !== null) {
        const origins = positionalOrigins(source, version);
        const assets = mirror.assets;
        const change = { before: source, after: version, assets, origins, measureText: measurer() };
        warnLeaving(versionId, itemsLeavingFrame(change));
      }
      return true;
    },
    remove: async (sequenceId) => {
      try {
        await api().removeSequence(sequenceId);
      } catch (error) {
        failed(error);
      }
    },
  };
}
