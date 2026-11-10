import { Badge, Button, Spinner } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { usePerformanceMeasure } from '../hooks/use-performance-measure';
import { budgetRows, type BudgetRow } from '../services/budgets';
import type { PerformanceReport } from '../services/measure-performance';

const STATUS_TONES = { within: 'success', over: 'danger', skipped: 'neutral' } as const;

function ResultRow({ row }: { readonly row: BudgetRow }) {
  const { t, i18n } = useTranslation();
  const number = (value: number) =>
    row.unit === 'ms'
      ? t('performance.milliseconds', {
          value: value.toLocaleString(i18n.language, { maximumFractionDigits: 1 }),
        })
      : value.toLocaleString(i18n.language);
  return (
    <tr data-budget={row.id} data-status={row.status} className="border-t border-border">
      <th scope="row" className="py-1.5 pr-3 text-left font-normal text-text">
        {t(`performance.rows.${row.id}`)}
      </th>
      <td className="py-1.5 pr-3 text-right timecode">
        {row.value === null ? '—' : number(row.value)}
      </td>
      <td className="py-1.5 pr-3 text-right text-text-muted timecode">≤ {number(row.budget)}</td>
      <td className="py-1.5 text-right">
        <Badge variant={STATUS_TONES[row.status]}>{t(`performance.status.${row.status}`)}</Badge>
      </td>
    </tr>
  );
}

function ReportTable({ report }: { readonly report: PerformanceReport }) {
  const { t } = useTranslation();
  return (
    <>
      <table className="w-full text-xs">
        <thead>
          <tr className="text-text-muted">
            <th scope="col" className="pb-1 text-left font-normal">
              {t('performance.columns.measure')}
            </th>
            <th scope="col" className="pb-1 pr-3 text-right font-normal">
              {t('performance.columns.value')}
            </th>
            <th scope="col" className="pb-1 pr-3 text-right font-normal">
              {t('performance.columns.budget')}
            </th>
            <th scope="col" className="pb-1 text-right font-normal">
              {t('performance.columns.status')}
            </th>
          </tr>
        </thead>
        <tbody>
          {budgetRows(report).map((row) => (
            <ResultRow key={row.id} row={row} />
          ))}
        </tbody>
      </table>
      <p className="text-xs text-text-muted">
        {report.scrub === null
          ? t('performance.noViewer')
          : t('performance.details', {
              fps: report.playback?.framesPerSecond ?? 0,
              frames: report.openVideoFrames ?? 0,
              heap: report.heapMegabytes ?? '—',
            })}
      </p>
    </>
  );
}

/**
 * « Performances »: measures core round trips, scrub latency, playback and main-thread frames in
 * this window and compares them with the budgets of `docs/performance.md` (prompt 29).
 */
export function PerformanceSection() {
  const { t } = useTranslation();
  const { state, measure } = usePerformanceMeasure();
  return (
    <section aria-labelledby="settings-performance" className="flex flex-col gap-3">
      <h3 id="settings-performance" className="text-sm font-semibold text-text">
        {t('performance.title')}
      </h3>
      <p className="text-xs text-text-muted">{t('performance.description')}</p>
      <div className="flex items-center gap-3">
        <Button onClick={measure} disabled={state.kind === 'running'}>
          {t('performance.measure')}
        </Button>
        {state.kind === 'running' && (
          <p role="status" className="flex items-center gap-2 text-xs text-text-muted">
            <Spinner size="sm" label={t(`performance.steps.${state.step}`)} />
            {t(`performance.steps.${state.step}`)}
          </p>
        )}
      </div>
      {state.kind === 'done' && <ReportTable report={state.report} />}
      {state.kind === 'stopped' && (
        <p role="alert" className="text-xs text-warning">
          {t(`performance.stopped.${state.reason}`)}
        </p>
      )}
      {state.kind === 'failed' && (
        <p role="alert" className="text-xs text-danger">
          {t(state.errorKey)}
        </p>
      )}
    </section>
  );
}
