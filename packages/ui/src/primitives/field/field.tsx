import { useId, type ReactElement, type ReactNode } from 'react';

import { cn } from '../../lib/cn';
import { Slot } from '../../lib/slot';
import { Label } from '../label';

/** Props of {@link Field}. */
export interface FieldProps {
  /** Visible label of the control. */
  label: ReactNode;
  /** Optional help text under the control. */
  description?: ReactNode;
  /** Error message; when present the control is marked `aria-invalid` and styled as invalid. */
  error?: ReactNode;
  /** `id` given to the control (generated when omitted). Set it here, not on the child. */
  id?: string;
  /**
   * `vertical` (default): label above the control. `horizontal`: control then label on one row,
   * for checkboxes and switches.
   */
  orientation?: 'vertical' | 'horizontal';
  className?: string;
  /** Exactly one focusable control element: `Input`, `Textarea`, `SelectTrigger`, `Checkbox`… */
  children: ReactElement;
}

/**
 * A labelled form control with optional description and error, wired for assistive technologies.
 * The single child receives `id`, `aria-describedby` and `aria-invalid` through a `Slot`, so any
 * control works without a dedicated API: `<Field label="Nom" error={error}><Input /></Field>`.
 * For a `Select`, wrap the `SelectTrigger` (the element that takes focus), not the `Select` root.
 * Props set explicitly on the child win over the injected ones.
 */
export function Field({
  label,
  description,
  error,
  id,
  orientation = 'vertical',
  className,
  children,
}: FieldProps) {
  const generatedId = useId();
  const controlId = id ?? generatedId;
  const descriptionId = description === undefined ? undefined : `${controlId}-description`;
  const errorId = error === undefined ? undefined : `${controlId}-error`;
  const describedBy = [descriptionId, errorId].filter(Boolean).join(' ') || undefined;

  const control = (
    <Slot id={controlId} aria-describedby={describedBy} aria-invalid={errorId ? true : undefined}>
      {children}
    </Slot>
  );
  const fieldLabel = <Label htmlFor={controlId}>{label}</Label>;
  const horizontal = orientation === 'horizontal';

  return (
    <div className={cn('flex flex-col gap-1.5', className)}>
      {horizontal ? (
        <div className="flex items-center gap-2">
          {control}
          {fieldLabel}
        </div>
      ) : (
        <>
          {fieldLabel}
          {control}
        </>
      )}
      <FieldMessages
        descriptionId={descriptionId}
        description={description}
        errorId={errorId}
        error={error}
      />
    </div>
  );
}

interface FieldMessagesProps {
  descriptionId: string | undefined;
  description: ReactNode;
  errorId: string | undefined;
  error: ReactNode;
}

/** Description and error lines of a {@link Field}, referenced by the control's `aria-describedby`. */
function FieldMessages({ descriptionId, description, errorId, error }: FieldMessagesProps) {
  return (
    <>
      {descriptionId !== undefined && (
        <p id={descriptionId} className="text-xs text-text-subtle">
          {description}
        </p>
      )}
      {errorId !== undefined && (
        <p id={errorId} className="text-xs text-danger">
          {error}
        </p>
      )}
    </>
  );
}
