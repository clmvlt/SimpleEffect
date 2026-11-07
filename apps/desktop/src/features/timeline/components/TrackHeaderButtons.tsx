import type { TrackEntity, TrackPatch } from '@se/protocol';
import { cn, IconButton, Tooltip } from '@se/ui';
import {
  Eye,
  EyeOff,
  Headphones,
  Lock,
  LockOpen,
  Volume2,
  VolumeX,
  type LucideIcon,
} from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

/** Props of {@link TrackHeaderButtons}. */
export interface TrackHeaderButtonsProps {
  readonly track: TrackEntity;
  readonly onChange: (patch: TrackPatch) => void;
}

interface ToggleProps {
  readonly on: boolean;
  readonly icon: LucideIcon;
  readonly label: string;
  readonly onToggle: () => void;
}

/** A small toggle of a track header, tinted while on. */
function Toggle({ on, icon, label, onToggle }: ToggleProps) {
  return (
    <Tooltip content={label}>
      <IconButton
        size="xs"
        icon={icon}
        label={label}
        aria-pressed={on}
        className={cn(on && 'bg-accent-muted text-text hover:bg-accent-muted')}
        onClick={onToggle}
      />
    </Tooltip>
  );
}

/**
 * The toggles of a track: visibility and lock for video tracks; mute, solo and lock for audio
 * tracks. Each click is one undoable change.
 */
export function TrackHeaderButtons({ track, onChange }: TrackHeaderButtonsProps) {
  const { t } = useTranslation();
  const name = track.name;
  return (
    <div className="flex items-center gap-px">
      {track.kind === 'video' ? (
        <Toggle
          on={track.hidden}
          icon={track.hidden ? EyeOff : Eye}
          label={t(track.hidden ? 'editor.timeline.track.show' : 'editor.timeline.track.hide', {
            name,
          })}
          onToggle={() => {
            onChange({ hidden: !track.hidden });
          }}
        />
      ) : (
        <>
          <Toggle
            on={track.muted}
            icon={track.muted ? VolumeX : Volume2}
            label={t(track.muted ? 'editor.timeline.track.unmute' : 'editor.timeline.track.mute', {
              name,
            })}
            onToggle={() => {
              onChange({ muted: !track.muted });
            }}
          />
          <Toggle
            on={track.solo}
            icon={Headphones}
            label={t(track.solo ? 'editor.timeline.track.unsolo' : 'editor.timeline.track.solo', {
              name,
            })}
            onToggle={() => {
              onChange({ solo: !track.solo });
            }}
          />
        </>
      )}
      <Toggle
        on={track.locked}
        icon={track.locked ? Lock : LockOpen}
        label={t(track.locked ? 'editor.timeline.track.unlock' : 'editor.timeline.track.lock', {
          name,
        })}
        onToggle={() => {
          onChange({ locked: !track.locked });
        }}
      />
    </div>
  );
}
