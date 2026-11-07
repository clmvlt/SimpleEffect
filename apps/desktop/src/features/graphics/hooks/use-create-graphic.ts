import type { AssetId, ShapeKind } from '@se/protocol';
import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectSelector } from '../../../shared/project-store';
import { showSidePanelTab } from '../../../shared/side-panel';
import { FLICKS_PER_SECOND, clampFlicks } from '../../../shared/time';
import { playheadStore, playheadTimeOf, selectItems, useActiveSequence } from '../../timeline';
import { graphicsApi } from '../api';
import { graphicsPreferencesStore } from '../store/graphics-preferences-store';

/** Creating graphics at the playhead of the active sequence (`null` without a sequence). */
export interface GraphicCreator {
  readonly createText: () => void;
  readonly createShape: (shape: ShapeKind) => void;
  readonly createImage: (assetId: AssetId) => void;
}

/**
 * Adds a text, a shape or an image at the playhead for the default length, on the highest free
 * video track (the core picks it), then selects it and shows the inspector.
 */
export function useCreateGraphic(): GraphicCreator | null {
  const { t } = useTranslation();
  const client = useCoreClient();
  const projectId = useProjectSelector((state) => state.mirror?.projectId);
  const sequence = useActiveSequence();
  if (projectId === undefined || sequence === null) return null;
  const api = graphicsApi(client, projectId);
  const create = (graphic: Parameters<typeof api.addGraphicItem>[3]) => {
    const at = playheadTimeOf(playheadStore.getState(), sequence.id);
    const seconds = graphicsPreferencesStore.getState().graphicSeconds;
    const duration = clampFlicks(seconds * FLICKS_PER_SECOND);
    api
      .addGraphicItem(sequence.id, at, duration, graphic)
      .then((placed) => {
        selectItems([placed.itemId]);
        showSidePanelTab('inspector');
      })
      .catch((error: unknown) => {
        toast.error(t(errorMessageKey(error)));
      });
  };
  return {
    createText: () => {
      create({ kind: 'text', content: t('editor.graphics.defaultText') });
    },
    createShape: (shape) => {
      create({ kind: 'shape', shape });
    },
    createImage: (assetId) => {
      create({ kind: 'image', assetId });
    },
  };
}
