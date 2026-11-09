/**
 * Motion templates as features see them (prompt 24): the app's template runtime (shared by the
 * viewer, frames for agents and previews) and the failing instances of the frame on screen.
 */
export {
  templateRuntimeFor,
  templatesSupported,
  type AppTemplateRuntime,
  type TemplateRuntimeSource,
} from './template-runtime';
export {
  publishTemplateIssues,
  templateIssuesStore,
  useTemplateIssue,
} from './template-issues-store';
