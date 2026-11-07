import { toast } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { clearPropertyPreview, previewContent } from '../../../shared/animation';
import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectSelector } from '../../../shared/project-store';
import { graphicsApi } from '../../graphics';
import { cornersOf, type FrameSize, type PlacedLayer } from '../services/gizmo-geometry';

/** Props of {@link TextEditOverlay}. */
export interface TextEditOverlayProps {
  readonly layer: PlacedLayer;
  readonly frame: FrameSize;
  readonly displayScale: number;
  readonly onClose: () => void;
}

/**
 * A field over a text in the viewer (double click): typing previews the text in the picture,
 * Ctrl/⌘+Entrée or leaving the field sends it (one undo step), Échap cancels.
 */
export function TextEditOverlay({ layer, frame, displayScale, onClose }: TextEditOverlayProps) {
  const { t } = useTranslation();
  const client = useCoreClient();
  const mirror = useProjectSelector((state) => state.mirror);
  const item = mirror?.items[layer.itemId];
  const original = item?.kind.kind === 'text' ? item.kind.content : '';
  const [content, setContent] = useState(original);
  const corners = cornersOf(layer, frame);
  const xs = corners.map((corner) => corner.x * displayScale);
  const ys = corners.map((corner) => corner.y * displayScale);
  const left = Math.min(...xs);
  const top = Math.min(...ys);
  const close = (save: boolean) => {
    onClose();
    if (!save || mirror === null || content === original) {
      clearPropertyPreview();
      return;
    }
    graphicsApi(client, mirror.projectId)
      .updateItems([{ itemId: layer.itemId, patch: { kind: 'text', content } }])
      .then(clearPropertyPreview, (error: unknown) => {
        clearPropertyPreview();
        toast.error(t(errorMessageKey(error)));
      });
  };
  return (
    <textarea
      aria-label={t('editor.viewer.gizmo.editText')}
      // The field replaces the double-clicked text: focusing it is the user's intent.
      autoFocus
      value={content}
      className="absolute resize-none rounded-xs border border-accent bg-overlay/80 p-1 text-sm text-text"
      style={{
        left,
        top,
        width: Math.max(120, Math.max(...xs) - left),
        height: Math.max(40, Math.max(...ys) - top),
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      onChange={(event) => {
        setContent(event.target.value);
        previewContent(layer.itemId, event.target.value);
      }}
      onKeyDown={(event) => {
        event.stopPropagation();
        if (event.key === 'Escape') close(false);
        if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) close(true);
      }}
      onBlur={() => {
        close(true);
      }}
    />
  );
}
