import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
  Button,
  type ButtonProps,
} from '@se/ui';
import type { ReactNode } from 'react';

/** Props of {@link ConfirmButton}. */
export interface ConfirmButtonProps {
  /** Text of the button opening the confirmation. */
  readonly children: ReactNode;
  readonly variant?: ButtonProps['variant'];
  readonly title: string;
  /** What will happen: for a download, what it is, where it comes from, its size, its check. */
  readonly description: ReactNode;
  readonly confirmLabel: string;
  readonly cancelLabel: string;
  /** `danger` for deletions. */
  readonly confirmVariant?: ButtonProps['variant'];
  /** Called only when the user confirms. */
  readonly onConfirm: () => void;
}

/**
 * A small button that asks first: the app never downloads (or deletes) a transcription file
 * without the user's explicit « Télécharger » (or « Supprimer ») in this dialog.
 */
export function ConfirmButton({
  children,
  variant = 'secondary',
  title,
  description,
  confirmLabel,
  cancelLabel,
  confirmVariant = 'primary',
  onConfirm,
}: ConfirmButtonProps) {
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button size="xs" variant={variant}>
          {children}
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent size="sm">
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription asChild>
            <div className="flex flex-col gap-2">{description}</div>
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{cancelLabel}</AlertDialogCancel>
          <AlertDialogAction variant={confirmVariant} onClick={onConfirm}>
            {confirmLabel}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
