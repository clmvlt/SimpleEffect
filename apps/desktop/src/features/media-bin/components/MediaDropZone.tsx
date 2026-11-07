import { Icon, cn } from '@se/ui';
import { Upload } from '@se/ui/icons';
import { useState, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

/** Props of {@link MediaDropZone}. */
export interface MediaDropZoneProps {
  /** Called with the files dropped from the OS. */
  readonly onDropFiles: (files: FileList) => void;
  readonly children: ReactNode;
}

/** Whether a drag carries files from the OS (not text or elements of the page). */
function carriesFiles(event: React.DragEvent): boolean {
  return event.dataTransfer.types.includes('Files');
}

/** Accepts files dragged from the file manager, showing an overlay while they hover. */
export function MediaDropZone({ onDropFiles, children }: MediaDropZoneProps) {
  const { t } = useTranslation();
  const [hovering, setHovering] = useState(false);
  return (
    <div
      className="relative flex min-h-0 flex-1 flex-col"
      onDragOver={(event) => {
        if (!carriesFiles(event)) return;
        event.preventDefault();
        event.dataTransfer.dropEffect = 'copy';
        setHovering(true);
      }}
      onDragLeave={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setHovering(false);
      }}
      onDrop={(event) => {
        if (!carriesFiles(event)) return;
        event.preventDefault();
        setHovering(false);
        onDropFiles(event.dataTransfer.files);
      }}
    >
      {children}
      <div
        aria-hidden={!hovering}
        className={cn(
          'pointer-events-none absolute inset-1 z-(--z-panel) flex flex-col items-center justify-center gap-1',
          'rounded-md border-2 border-dashed border-accent bg-overlay/90 text-center',
          'transition-opacity duration-(--duration-fast)',
          hovering ? 'opacity-100' : 'opacity-0',
        )}
      >
        <Icon icon={Upload} size="lg" className="text-accent" />
        <span className="text-sm font-medium text-text">{t('editor.media.drop.title')}</span>
        <span className="text-xs text-text-muted">{t('editor.media.drop.description')}</span>
      </div>
    </div>
  );
}
