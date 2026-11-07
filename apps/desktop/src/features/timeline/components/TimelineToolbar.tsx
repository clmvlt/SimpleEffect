import type { SequenceEntity, TrackKind } from '@se/protocol';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Slider,
  TimecodeInput,
  Toolbar,
  ToolbarButton,
  ToolbarGroup,
  ToolbarSeparator,
  Tooltip,
} from '@se/ui';
import { Magnet, Maximize2, Plus, Rows3, ZoomIn, ZoomOut } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';
import { useStore } from 'zustand';

import { useShortcutHint, type ShortcutActionId } from '../../../shared/shortcuts';
import { formatTimecode, parseTimecode } from '../../../shared/time';
import type { PlayheadControls } from '../hooks/use-playhead-controls';
import { ZOOM_STEP, type TimelineViewportApi } from '../hooks/use-timeline-viewport';
import { TRACK_HEIGHT_PRESETS, type TrackHeightPreset } from '../services/track-height';
import { sliderToZoom, ZOOM_SLIDER_MAX, zoomToSlider } from '../services/viewport';
import { usePlayheadTime } from '../store/playhead-store';
import { setSnapping, timelineViewStore } from '../store/timeline-view-store';

/** Props of {@link TimelineToolbar}. */
export interface TimelineToolbarProps {
  readonly sequence: SequenceEntity;
  readonly api: TimelineViewportApi;
  readonly playhead: PlayheadControls;
  readonly onAddTrack: (kind: TrackKind) => void;
  readonly onTrackHeights: (preset: TrackHeightPreset) => void;
}

/** Tooltip props for an action: its label and its shortcut, when it has one. */
function useActionTooltip(label: string, action: ShortcutActionId) {
  const shortcut = useShortcutHint(action);
  return shortcut === undefined ? { content: label } : { content: label, shortcut };
}

/**
 * The timeline tools: add a track, zoom (buttons, slider, fit), track heights, snapping, and the
 * playhead timecode, which accepts a typed time (« 51s », « 1:02.5 », « 00:00:51:12 »).
 */
export function TimelineToolbar({
  sequence,
  api,
  playhead,
  onAddTrack,
  onTrackHeights,
}: TimelineToolbarProps) {
  const { t } = useTranslation();
  const snapping = useStore(timelineViewStore, (state) => state.snapping);
  const zoomInTip = useActionTooltip(t('editor.timeline.toolbar.zoomIn'), 'zoomIn');
  const zoomOutTip = useActionTooltip(t('editor.timeline.toolbar.zoomOut'), 'zoomOut');
  const fitTip = useActionTooltip(t('editor.timeline.toolbar.fit'), 'zoomToFit');
  const snapTip = useActionTooltip(t('editor.timeline.toolbar.snapping'), 'toggleSnapping');
  return (
    <div className="flex min-w-0 flex-1 items-center gap-2">
      <PlayheadTimecodeField sequence={sequence} playhead={playhead} />
      <Toolbar label={t('editor.timeline.toolbar.label')} className="ml-auto">
        <ToolbarGroup label={t('editor.timeline.toolbar.tracks')}>
          <DropdownMenu>
            <Tooltip content={t('editor.timeline.toolbar.addTrack')}>
              <DropdownMenuTrigger asChild>
                <ToolbarButton icon={Plus} label={t('editor.timeline.toolbar.addTrack')} />
              </DropdownMenuTrigger>
            </Tooltip>
            <DropdownMenuContent align="end">
              <DropdownMenuItem
                onSelect={() => {
                  onAddTrack('video');
                }}
              >
                {t('editor.timeline.toolbar.addVideoTrack')}
              </DropdownMenuItem>
              <DropdownMenuItem
                onSelect={() => {
                  onAddTrack('audio');
                }}
              >
                {t('editor.timeline.toolbar.addAudioTrack')}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
          <DropdownMenu>
            <Tooltip content={t('editor.timeline.toolbar.trackHeights')}>
              <DropdownMenuTrigger asChild>
                <ToolbarButton icon={Rows3} label={t('editor.timeline.toolbar.trackHeights')} />
              </DropdownMenuTrigger>
            </Tooltip>
            <DropdownMenuContent align="end">
              {(Object.keys(TRACK_HEIGHT_PRESETS) as TrackHeightPreset[]).map((preset) => (
                <DropdownMenuItem
                  key={preset}
                  onSelect={() => {
                    onTrackHeights(preset);
                  }}
                >
                  {t(`editor.timeline.toolbar.heights.${preset}`)}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
          <Tooltip {...snapTip}>
            <ToolbarButton
              icon={Magnet}
              label={t('editor.timeline.toolbar.snapping')}
              pressed={snapping}
              onClick={() => {
                setSnapping(!snapping);
              }}
            />
          </Tooltip>
        </ToolbarGroup>
        <ToolbarSeparator />
        <ToolbarGroup label={t('editor.timeline.toolbar.zoom')}>
          <Tooltip {...zoomOutTip}>
            <ToolbarButton
              icon={ZoomOut}
              label={zoomOutTip.content}
              onClick={() => {
                api.zoomBy(1 / ZOOM_STEP);
              }}
            />
          </Tooltip>
          <Slider
            className="w-28"
            thumbLabels={[t('editor.timeline.toolbar.zoomLevel')]}
            min={0}
            max={ZOOM_SLIDER_MAX}
            step={1}
            value={[zoomToSlider(api.viewport.pixelsPerSecond, api.limits)]}
            onValueChange={([value]) => {
              if (value !== undefined) api.zoomTo(sliderToZoom(value, api.limits));
            }}
          />
          <Tooltip {...zoomInTip}>
            <ToolbarButton
              icon={ZoomIn}
              label={zoomInTip.content}
              onClick={() => {
                api.zoomBy(ZOOM_STEP);
              }}
            />
          </Tooltip>
          <Tooltip {...fitTip}>
            <ToolbarButton icon={Maximize2} label={fitTip.content} onClick={api.fit} />
          </Tooltip>
        </ToolbarGroup>
      </Toolbar>
    </div>
  );
}

/**
 * The playhead timecode field, alone in its component: it re-renders on every playhead move
 * (each frame during playback), the rest of the toolbar does not.
 */
function PlayheadTimecodeField({
  sequence,
  playhead,
}: Pick<TimelineToolbarProps, 'sequence' | 'playhead'>) {
  const { t } = useTranslation();
  const rate = sequence.format.frameRate;
  const time = usePlayheadTime(sequence.id);
  return (
    <TimecodeInput
      size="xs"
      className="w-28"
      label={t('editor.timeline.toolbar.playhead')}
      value={time}
      format={(value) => formatTimecode(value as typeof time, rate)}
      parse={(text) => {
        const parsed = parseTimecode(text, rate);
        return parsed.ok ? parsed.value : null;
      }}
      onCommit={(value) => {
        playhead.moveTo(value as typeof time, true);
      }}
    />
  );
}
