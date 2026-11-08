/**
 * AI feature (prompts 17–20). Prompt 17: the AI provider's status (`useAiStatusSync` mounted once
 * by the app layout, `useAiStatus`, `useRefreshAiStatus`, `AiStatusIndicator` for the status bar).
 * Prompt 20: the « IA » tab (`AiView`: task cards and the composer) and `useAiTaskSync`, mounted
 * once by the editor layout (tasks mirrored in `shared/ai-tasks` for the timeline's overlays,
 * end-of-task toasts and system notifications, Ctrl/⌘+I, composer requests).
 */
export { AiStatusIndicator } from './components/AiStatusIndicator';
export type { AiStatusIndicatorProps } from './components/AiStatusIndicator';
export { AiView } from './components/AiView';
export { useAiTaskSync } from './hooks/use-ai-task-sync';
export {
  useAiRefreshing,
  useAiStatus,
  useAiStatusSync,
  useRefreshAiStatus,
} from './hooks/use-ai-status';
export { aiAvailability, canSubmitTasks, type AiAvailability } from './services/ai-indicator';
export { aiTextValues, type AiTextValues } from './services/ai-texts';
