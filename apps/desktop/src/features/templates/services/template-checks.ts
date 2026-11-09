import type {
  CoreEndpoint,
  FontFamily,
  TemplateCheckOutcome,
  TemplateCheckRequest,
} from '@se/protocol';

/** What answering a template check needs (injected: tests use fakes). */
export interface TemplateCheckDeps {
  /** The core's HTTP endpoint (font files), `null` while disconnected. */
  readonly endpoint: () => CoreEndpoint | null;
  /** The font catalogue (`null` until listed). */
  readonly families: () => readonly FontFamily[] | null;
  /** Checks a module (`@se/engine/templates`' `checkTemplate`, imported lazily). */
  readonly check: (
    request: TemplateCheckRequest,
    families: readonly FontFamily[],
    endpoint: CoreEndpoint | null,
  ) => Promise<TemplateCheckOutcome>;
  /** Sends the verdict to the core (`reportTemplateCheck`). */
  readonly report: (requestId: string, outcome: TemplateCheckOutcome) => Promise<void>;
}

/**
 * Checks the template module an AI agent wrote (`templateCheckRequested`, prompt 24) in this
 * window's sandbox and reports the verdict; a check that cannot run here is reported as a failure
 * the agent can read.
 */
export async function answerTemplateCheck(
  request: TemplateCheckRequest,
  deps: TemplateCheckDeps,
): Promise<void> {
  let outcome: TemplateCheckOutcome;
  try {
    outcome = await deps.check(request, deps.families() ?? [], deps.endpoint());
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    outcome = {
      kind: 'failed',
      problems: [
        {
          stage: 'render',
          message: `the editor could not run the template: ${detail}`,
          line: null,
          column: null,
          time: null,
        },
      ],
    };
  }
  await deps.report(request.requestId, outcome).catch(() => undefined);
}
