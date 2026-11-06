import type { VariantProps } from 'class-variance-authority';
import { AlertDialog as AlertDialogPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { buttonVariants } from '../button';
import { modalContentVariants, modalOverlayClasses } from '../dialog/dialog';

/**
 * A modal confirmation that interrupts the user, for destructive or irreversible actions. Unlike
 * `Dialog`, a click outside does not close it and focus starts on the cancel button, so a stray
 * Enter never confirms. Compose: `<AlertDialog><AlertDialogTrigger asChild>…</AlertDialogTrigger>
 * <AlertDialogContent><AlertDialogHeader><AlertDialogTitle/><AlertDialogDescription/></AlertDialogHeader>
 * <AlertDialogFooter><AlertDialogCancel/><AlertDialogAction variant="danger"/></AlertDialogFooter>
 * </AlertDialogContent></AlertDialog>`.
 */
export const AlertDialog = AlertDialogPrimitive.Root;

/** The element that opens the confirmation; use `asChild` to keep your own button. */
export const AlertDialogTrigger = AlertDialogPrimitive.Trigger;

/** Props of {@link AlertDialogContent}. */
export interface AlertDialogContentProps
  extends
    ComponentProps<typeof AlertDialogPrimitive.Content>,
    VariantProps<typeof modalContentVariants> {}

/** The confirmation window (narrow by default), rendered in a portal over a dimmed backdrop. */
export function AlertDialogContent({ size = 'sm', className, ...props }: AlertDialogContentProps) {
  return (
    <AlertDialogPrimitive.Portal>
      <AlertDialogPrimitive.Overlay className={cn(modalOverlayClasses)} />
      <AlertDialogPrimitive.Content
        className={cn(modalContentVariants({ size }), className)}
        {...props}
      />
    </AlertDialogPrimitive.Portal>
  );
}

/** Title and description block. */
export function AlertDialogHeader({ className, ...props }: ComponentProps<'div'>) {
  return <div className={cn('flex flex-col gap-1', className)} {...props} />;
}

/** Action row, right-aligned: cancel first, confirming action last. */
export function AlertDialogFooter({ className, ...props }: ComponentProps<'div'>) {
  return <div className={cn('flex justify-end gap-2 pt-1', className)} {...props} />;
}

/** The question asked (accessible name of the alert dialog). */
export function AlertDialogTitle({
  className,
  ...props
}: ComponentProps<typeof AlertDialogPrimitive.Title>) {
  return (
    <AlertDialogPrimitive.Title
      className={cn('text-md leading-tight font-semibold text-text', className)}
      {...props}
    />
  );
}

/** The consequences of confirming, announced with the title. */
export function AlertDialogDescription({
  className,
  ...props
}: ComponentProps<typeof AlertDialogPrimitive.Description>) {
  return (
    <AlertDialogPrimitive.Description
      className={cn('text-sm text-text-muted', className)}
      {...props}
    />
  );
}

/** Props of {@link AlertDialogAction} and {@link AlertDialogCancel}: button variants apply. */
export type AlertDialogButtonProps = ComponentProps<typeof AlertDialogPrimitive.Action> &
  VariantProps<typeof buttonVariants>;

/** Confirms and closes. Use `variant="danger"` for destructive actions. */
export function AlertDialogAction({
  variant = 'primary',
  size,
  className,
  ...props
}: AlertDialogButtonProps) {
  return (
    <AlertDialogPrimitive.Action
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    />
  );
}

/** Dismisses without acting; receives focus when the dialog opens. */
export function AlertDialogCancel({
  variant = 'ghost',
  size,
  className,
  ...props
}: AlertDialogButtonProps) {
  return (
    <AlertDialogPrimitive.Cancel
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    />
  );
}
