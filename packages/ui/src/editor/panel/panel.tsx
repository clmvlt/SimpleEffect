import { createContext, use, useId, type ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { ScrollArea } from '../../primitives/scroll-area';

/** Shares the generated title id so `Panel` is named by its `PanelTitle` without extra props. */
const PanelTitleIdContext = createContext<string | undefined>(undefined);

/** Props of {@link Panel}. */
export type PanelProps = ComponentProps<'section'>;

/**
 * A docked editor panel (Médias, Viewer, Inspecteur…): a header row and a body filling the rest.
 * It is a named region: it takes its name from its `PanelTitle`, or from `aria-label` when the
 * header shows tabs instead of a title. Compose it with `PanelHeader`, `PanelTitle`,
 * `PanelActions` and `PanelBody`; wrap header and body in `Tabs` for a tabbed panel.
 */
export function Panel({ className, children, ...props }: PanelProps) {
  const titleId = useId();
  const labelledBy = props['aria-label'] === undefined ? titleId : undefined;
  return (
    <PanelTitleIdContext value={titleId}>
      <section
        aria-labelledby={labelledBy}
        className={cn('flex size-full min-h-0 min-w-0 flex-col bg-panel text-text', className)}
        {...props}
      >
        {children}
      </section>
    </PanelTitleIdContext>
  );
}

/** Props of {@link PanelHeader}. */
export type PanelHeaderProps = ComponentProps<'header'>;

/** The 32 px header row: title or tabs on the left, `PanelActions` pushed to the right. */
export function PanelHeader({ className, ...props }: PanelHeaderProps) {
  return (
    <header
      className={cn(
        'flex h-control-md shrink-0 items-center gap-2 border-b border-border px-2',
        className,
      )}
      {...props}
    />
  );
}

/** Props of {@link PanelTitle}. */
export type PanelTitleProps = ComponentProps<'h2'>;

/** The panel's title; it also names the panel region for assistive technologies. */
export function PanelTitle({ className, ...props }: PanelTitleProps) {
  const titleId = use(PanelTitleIdContext);
  return (
    <h2
      id={titleId}
      className={cn('truncate text-xs font-semibold text-text-muted select-none', className)}
      {...props}
    />
  );
}

/** Props of {@link PanelActions}. */
export type PanelActionsProps = ComponentProps<'div'>;

/** Icon buttons and menus of the panel, aligned to the right of the header. */
export function PanelActions({ className, ...props }: PanelActionsProps) {
  return <div className={cn('ml-auto flex items-center gap-0.5', className)} {...props} />;
}

/** Props of {@link PanelBody}. */
export interface PanelBodyProps extends ComponentProps<'div'> {
  /**
   * Scroll the content inside the panel (default). Turn it off for content that manages its own
   * size and scrolling (viewer canvas, timeline).
   */
  scrollable?: boolean | undefined;
}

/** The content area, filling the space under the header. */
export function PanelBody({ scrollable = true, className, children, ...props }: PanelBodyProps) {
  if (!scrollable) {
    return (
      <div className={cn('relative min-h-0 flex-1', className)} {...props}>
        {children}
      </div>
    );
  }
  return (
    <div className={cn('relative min-h-0 flex-1', className)} {...props}>
      <ScrollArea className="size-full">{children}</ScrollArea>
    </div>
  );
}
