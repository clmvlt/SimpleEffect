import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Toolbar,
  ToolbarButton,
  Tooltip,
} from '@se/ui';
import { Gauge, Grid2x2 } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import {
  VIEWER_ZOOMS,
  setViewerPreferences,
  type ViewerPreferences,
  type ViewerZoom,
} from '../store/viewer-store';

/** Props of {@link ViewerToolbar}. */
export interface ViewerToolbarProps {
  readonly preferences: ViewerPreferences;
}

const zoomValue = (zoom: ViewerZoom) => String(zoom);

/** The viewer options: zoom level, proxy playback, transparency checkerboard. */
export function ViewerToolbar({ preferences }: ViewerToolbarProps) {
  const { t } = useTranslation();
  const zoomLabel = (zoom: ViewerZoom) =>
    zoom === 'fit'
      ? t('editor.viewer.toolbar.zoomFit')
      : t('editor.viewer.toolbar.zoomPercent', { percent: zoom * 100 });
  return (
    <div className="ml-auto flex items-center gap-1">
      <Select
        value={zoomValue(preferences.zoom)}
        onValueChange={(value) => {
          const zoom = VIEWER_ZOOMS.find((candidate) => zoomValue(candidate) === value);
          if (zoom !== undefined) setViewerPreferences({ zoom });
        }}
      >
        <SelectTrigger size="xs" className="w-24" aria-label={t('editor.viewer.toolbar.zoom')}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {VIEWER_ZOOMS.map((zoom) => (
            <SelectItem key={zoomValue(zoom)} value={zoomValue(zoom)}>
              {zoomLabel(zoom)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Toolbar label={t('editor.viewer.toolbar.label')}>
        <Tooltip content={t('editor.viewer.toolbar.proxy')}>
          <ToolbarButton
            icon={Gauge}
            label={t('editor.viewer.toolbar.proxy')}
            pressed={preferences.proxy}
            onClick={() => {
              setViewerPreferences({ proxy: !preferences.proxy });
            }}
          />
        </Tooltip>
        <Tooltip content={t('editor.viewer.toolbar.checkerboard')}>
          <ToolbarButton
            icon={Grid2x2}
            label={t('editor.viewer.toolbar.checkerboard')}
            pressed={preferences.checkerboard}
            onClick={() => {
              setViewerPreferences({ checkerboard: !preferences.checkerboard });
            }}
          />
        </Tooltip>
      </Toolbar>
    </div>
  );
}
