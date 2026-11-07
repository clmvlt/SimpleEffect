/**
 * AI feature (prompts 17–20). Prompt 17: the AI provider's status (`useAiStatusSync` mounted once
 * by the app layout, `useAiStatus`, `useRefreshAiStatus`, `AiStatusIndicator` for the status bar)
 * and, in development builds, a « Tester une tâche » panel in the « IA » tab.
 * TODO(prompt-20): prompt composer, task panel, scope overlays.
 */
export { AiStatusIndicator } from './components/AiStatusIndicator';
export type { AiStatusIndicatorProps } from './components/AiStatusIndicator';
export { AiView } from './components/AiView';
export {
  useAiRefreshing,
  useAiStatus,
  useAiStatusSync,
  useRefreshAiStatus,
} from './hooks/use-ai-status';
export { aiAvailability, canSubmitTasks, type AiAvailability } from './services/ai-indicator';
export { aiTextValues, type AiTextValues } from './services/ai-texts';
