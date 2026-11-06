import { ChevronRight } from 'lucide-react';
import { Collapsible as CollapsiblePrimitive } from 'radix-ui';
import { useId, type ComponentProps, type ReactNode } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';
import { Icon } from '../../primitives/icon';
import { Label } from '../../primitives/label';

/** Props of {@link PropertyGroup}. */
export interface PropertyGroupProps extends Omit<
  ComponentProps<typeof CollapsiblePrimitive.Root>,
  'title'
> {
  /** Group title, e.g. « Transformation ». */
  title: string;
  /** Buttons shown at the right of the title row (reset, keyframe all…). */
  actions?: ReactNode | undefined;
}

/**
 * A collapsible section of the inspector holding `PropertyRow`s. Open by default; the title row
 * is a button (Enter/Space toggles it). Control it with `open`/`onOpenChange` to remember which
 * groups the user folded.
 */
export function PropertyGroup({
  title,
  actions,
  defaultOpen = true,
  className,
  children,
  ...props
}: PropertyGroupProps) {
  return (
    <CollapsiblePrimitive.Root
      defaultOpen={defaultOpen}
      className={cn('border-b border-border', className)}
      {...props}
    >
      <div className="flex h-control-md items-center gap-1 pr-2">
        <CollapsiblePrimitive.Trigger
          className={cn(
            'group flex h-full min-w-0 flex-1 items-center gap-1 rounded-sm pl-1 text-left',
            'text-xs font-semibold text-text hover:bg-hover',
            focusRing,
          )}
        >
          <Icon
            icon={ChevronRight}
            size="sm"
            className="shrink-0 text-text-subtle transition-transform duration-(--duration-fast) group-data-[state=open]:rotate-90"
          />
          <span className="truncate">{title}</span>
        </CollapsiblePrimitive.Trigger>
        {actions !== undefined && <div className="flex items-center gap-0.5">{actions}</div>}
      </div>
      <CollapsiblePrimitive.Content className="flex flex-col gap-1 px-2 pb-2">
        {children}
      </CollapsiblePrimitive.Content>
    </CollapsiblePrimitive.Root>
  );
}

/** Props of {@link PropertyRow}. */
export interface PropertyRowProps extends Omit<ComponentProps<'div'>, 'children'> {
  /** Property name, e.g. « Opacité ». */
  label: string;
  /**
   * Id of the single control of the row: the label then names it (`<label for>`). Without it, the
   * control area becomes a group named by the label (for X/Y pairs and compound controls).
   */
  htmlFor?: string | undefined;
  /** The control: `ScrubInput`, `Select`, `ColorField`, `Switch`… */
  children: ReactNode;
  /** The keyframe button of an animatable property (`KeyframeButton`). */
  keyframe?: ReactNode | undefined;
  /** The value differs from its default: shows an accent mark before the label. */
  modified?: boolean | undefined;
  /** Text read by screen readers for the modified mark (default English « modified »). */
  modifiedLabel?: string | undefined;
}

/**
 * One property of the inspector: label, control and keyframe slot on a 28 px row. Rows of a group
 * align their columns, so labels, controls and keyframe buttons line up.
 */
export function PropertyRow({
  label,
  htmlFor,
  children,
  keyframe,
  modified = false,
  modifiedLabel = 'modified',
  className,
  ...props
}: PropertyRowProps) {
  const labelId = useId();
  return (
    <div
      data-modified={modified || undefined}
      className={cn(
        'grid min-h-control-sm grid-cols-[minmax(0,6.5rem)_minmax(0,1fr)_auto] items-center gap-2',
        className,
      )}
      {...props}
    >
      <Label
        id={labelId}
        htmlFor={htmlFor}
        className={cn('flex min-w-0 items-center gap-1.5', modified && 'text-text')}
      >
        <span
          aria-hidden
          className={cn(
            'size-1.5 shrink-0 rounded-full',
            modified ? 'bg-accent' : 'bg-transparent',
          )}
        />
        <span className="truncate">{label}</span>
        {modified && <span className="sr-only">({modifiedLabel})</span>}
      </Label>
      <div
        role={htmlFor === undefined ? 'group' : undefined}
        aria-labelledby={htmlFor === undefined ? labelId : undefined}
        className="flex min-w-0 items-center gap-1"
      >
        {children}
      </div>
      <div className="flex w-control-sm justify-center">{keyframe}</div>
    </div>
  );
}
