/**
 * In-app performance measure (prompt 29, `docs/performance.md`): « Mesurer les performances »
 * times core round trips, scrubbing, playback and main-thread frames against the budgets.
 */
export { PerformanceSection } from './components/PerformanceSection';
export { BUDGETS, budgetRows } from './services/budgets';
export type { BudgetRow } from './services/budgets';
export { measurePerformance } from './services/measure-performance';
export type { MeasureDependencies, PerformanceReport } from './services/measure-performance';
