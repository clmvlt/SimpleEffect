/**
 * `@se/engine/templates` (prompt 24, ADR-0035): motion templates written by AI agents — compiled
 * with the embedded sucrase, run in sandboxed Workers, rendered to pictures for the frame renderer
 * (`TemplateRuntime`, given to the preview engine and the still renderer) and checked before they
 * enter a project (`checkTemplate`). Loads the compiler: **import lazily** from UI code.
 */
export { checkTemplate, type TemplateCheckOptions } from './check-template';
export { checkDefaults, checkInstants, type CheckDefaults, type CheckInstants } from './check-plan';
export {
  compileTemplate,
  type CompileProblem,
  type CompileResult,
  type CompiledTemplate,
} from './compile';
export {
  TEMPLATE_FRAME_BUDGET_MS,
  TEMPLATE_LOAD_WATCHDOG_MS,
  TEMPLATE_WATCHDOG_MS,
  type TemplateFontFace,
  type TemplateImageHandle,
  type TemplateParamInput,
} from './contract';
export {
  catalogFontSource,
  fetchFontBytes,
  type FontBytesLoader,
  type TemplateFontSource,
} from './font-faces';
export { templateWorkerSource, type WorkerSource } from './sandbox-source';
export type {
  TemplateIssue,
  TemplateRenderRequest,
  TemplateRenderResult,
  TemplateRenderer,
} from './template-renderer';
export { TemplateRuntime, type TemplateRuntimeOptions } from './template-runtime';
export {
  TemplateFailure,
  TemplateWorker,
  blobWorkerFactory,
  type TemplateFailureKind,
  type TemplateFrame,
  type WorkerFactory,
} from './template-worker';
export { DEFAULT_TEMPLATE_FAMILY, type TemplateImageSource } from './worker-pool';
