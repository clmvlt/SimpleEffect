import { CircleCheck, CircleX, Info, LoaderCircle, TriangleAlert, X } from 'lucide-react';
import { Toaster as SonnerToaster, toast as sonnerToast } from 'sonner';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';
import { buttonVariants } from '../button';
import { Icon } from '../icon';

/**
 * Shows a toast; the `Toaster` mounted at the app root renders it. `toast('Projet enregistré')`,
 * `toast.success(…)`, `toast.error(…, { description, action: { label, onClick } })`,
 * `toast.warning`, `toast.info`, `toast.loading` then `toast.dismiss(id)`, `toast.promise(…)`.
 * Messages come from i18n; keep them short (one line plus an optional description).
 */
export const toast: typeof sonnerToast = sonnerToast;

/** Props of {@link Toaster}. */
export interface ToasterProps {
  /** Accessible name of the notification region (e.g. « Notifications »). */
  label: string;
  /**
   * Accessible name of the × button of each toast (e.g. « Fermer la notification »). Toasts show
   * a close button only when this is set; they otherwise disappear by themselves or by swiping.
   */
  closeLabel?: string;
  /** Corner of the window (default `bottom-right`, away from the timeline playhead controls). */
  position?: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
  /** Default display time in milliseconds (default 4000). */
  duration?: number;
}

const toastClassNames = {
  toast: cn(
    'flex w-(--width) items-start gap-2 rounded-lg border border-border bg-overlay p-3',
    'text-sm text-text shadow-overlay',
  ),
  content: 'flex min-w-0 flex-1 flex-col gap-0.5',
  title: 'font-medium',
  description: 'text-xs text-text-muted',
  icon: 'mt-0.5 flex shrink-0',
  actionButton: buttonVariants({ variant: 'primary', size: 'xs' }),
  cancelButton: buttonVariants({ variant: 'ghost', size: 'xs' }),
  closeButton: cn(
    'absolute -top-2 -left-2 inline-flex size-5 items-center justify-center rounded-full',
    'border border-border bg-overlay text-text-muted hover:text-text',
    focusRing,
  ),
};

const toastIcons = {
  success: <Icon icon={CircleCheck} className="text-success" />,
  error: <Icon icon={CircleX} className="text-danger" />,
  warning: <Icon icon={TriangleAlert} className="text-warning" />,
  info: <Icon icon={Info} className="text-info" />,
  loading: <Icon icon={LoaderCircle} className="animate-spin text-text-muted" />,
  close: <Icon icon={X} size="xs" />,
};

/**
 * Renders the toasts sent with {@link toast}. Mount it once at the app root. Toasts are announced
 * politely by screen readers and stack in a corner above every other layer except tooltips.
 */
export function Toaster({ label, closeLabel, position = 'bottom-right', duration }: ToasterProps) {
  return (
    <SonnerToaster
      theme="dark"
      position={position}
      customAriaLabel={label}
      className="z-(--z-toast)!"
      icons={toastIcons}
      closeButton={closeLabel !== undefined}
      toastOptions={{
        unstyled: true,
        classNames: toastClassNames,
        ...(closeLabel === undefined ? {} : { closeButtonAriaLabel: closeLabel }),
        ...(duration === undefined ? {} : { duration }),
      }}
    />
  );
}
