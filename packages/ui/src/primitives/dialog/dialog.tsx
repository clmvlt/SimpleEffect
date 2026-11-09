import { cva, type VariantProps } from 'class-variance-authority';
import { X } from 'lucide-react';
import { Dialog as DialogPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';
import { Icon } from '../icon';

/**
 * A modal window: traps focus, closes on Escape and returns focus to its trigger. Compose:
 * `<Dialog><DialogTrigger asChild>…</DialogTrigger><DialogContent closeLabel="Fermer">
 * <DialogHeader><DialogTitle/><DialogDescription/></DialogHeader>…<DialogFooter/></DialogContent></Dialog>`.
 * Use `AlertDialog` instead to confirm a destructive action.
 */
export const Dialog = DialogPrimitive.Root;

/** The element that opens the dialog; use `asChild` to keep your own button. */
export const DialogTrigger = DialogPrimitive.Trigger;

/** Closes the dialog when activated (e.g. a « Annuler » button, `asChild`). */
export const DialogClose = DialogPrimitive.Close;

/** Backdrop behind every modal layer (dialogs and alert dialogs). */
export const modalOverlayClasses = [
  'fixed inset-0 z-(--z-modal) bg-scrim',
  'data-[state=open]:animate-overlay-in data-[state=closed]:animate-overlay-out',
];

/**
 * Centred modal surface, shared with `AlertDialog`. Centring uses the `translate` property, so the
 * enter/exit `transform: scale()` animation composes with it.
 */
export const modalContentVariants = cva(
  [
    'fixed top-1/2 left-1/2 z-(--z-modal) flex -translate-x-1/2 -translate-y-1/2 flex-col gap-4',
    'max-h-[calc(100vh-4rem)] w-[calc(100vw-2rem)] overflow-y-auto',
    'rounded-lg border border-border bg-overlay p-5 text-text shadow-overlay outline-none',
    'data-[state=open]:animate-content-in data-[state=closed]:animate-content-out',
  ],
  {
    variants: {
      size: { sm: 'max-w-sm', md: 'max-w-md', lg: 'max-w-2xl', xl: 'max-w-4xl' },
    },
    defaultVariants: { size: 'md' },
  },
);

/** Props of {@link DialogContent}. */
export interface DialogContentProps
  extends
    ComponentProps<typeof DialogPrimitive.Content>,
    VariantProps<typeof modalContentVariants> {
  /**
   * Accessible name of the × button in the top-right corner (e.g. « Fermer »). The button is shown
   * only when this is set; Escape always closes the dialog.
   */
  closeLabel?: string;
}

/** The modal window itself, rendered in a portal over a dimmed backdrop. */
export function DialogContent({
  size,
  closeLabel,
  className,
  children,
  ...props
}: DialogContentProps) {
  return (
    <DialogPrimitive.Portal>
      <DialogPrimitive.Overlay className={cn(modalOverlayClasses)} />
      <DialogPrimitive.Content className={cn(modalContentVariants({ size }), className)} {...props}>
        {children}
        {closeLabel !== undefined && (
          <DialogPrimitive.Close
            aria-label={closeLabel}
            className={cn(
              'absolute top-3 right-3 inline-flex size-control-xs items-center justify-center rounded-sm',
              'text-text-muted transition-colors duration-(--duration-fast) hover:bg-hover hover:text-text',
              focusRing,
            )}
          >
            <Icon icon={X} size="sm" />
          </DialogPrimitive.Close>
        )}
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  );
}

/** Title and description block at the top of a dialog. */
export function DialogHeader({ className, ...props }: ComponentProps<'div'>) {
  return <div className={cn('flex flex-col gap-1 pr-6', className)} {...props} />;
}

/** Action row at the bottom of a dialog, right-aligned (primary action last). */
export function DialogFooter({ className, ...props }: ComponentProps<'div'>) {
  return <div className={cn('flex justify-end gap-2 pt-1', className)} {...props} />;
}

/** The dialog's accessible name. Required by Radix: every dialog needs a title. */
export function DialogTitle({ className, ...props }: ComponentProps<typeof DialogPrimitive.Title>) {
  return (
    <DialogPrimitive.Title
      className={cn('text-md leading-tight font-semibold text-text', className)}
      {...props}
    />
  );
}

/** One sentence explaining the dialog, announced with the title. */
export function DialogDescription({
  className,
  ...props
}: ComponentProps<typeof DialogPrimitive.Description>) {
  return (
    <DialogPrimitive.Description className={cn('text-sm text-text-muted', className)} {...props} />
  );
}
