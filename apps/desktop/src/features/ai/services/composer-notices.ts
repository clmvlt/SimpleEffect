import type { AssetId, SequenceId, Task, TimeRange, TranscriptionStatus } from '@se/protocol';

import { isTaskActive } from '../../../shared/ai-tasks';
import type { ProjectMirror } from '../../../shared/project-store';
import { assetsOf, soundClipsOf } from '../../../shared/transcript';
import type { DerivedScope } from './task-scope';

/**
 * What the composer points out before a prompt is sent: a scope taken from a selection that
 * covers little of the sequence (often a clip left selected by accident), media the agent cannot
 * read because they have no transcript, and the recent task a new prompt continues by default.
 */

/** Below this share of the sequence, a range taken from the selection is pointed out. */
export const SMALL_SELECTION_SHARE = 0.1;

/** How long after a task finished the composer continues it by default. */
export const CONTINUE_WINDOW_MS = 30 * 60 * 1000;

/** The scope's range when it comes from the selection and covers little of the sequence. */
export function smallSelection(derived: DerivedScope, sequenceEnd: number): TimeRange | null {
  const range = derived.scope.range;
  if (derived.rangeSource !== 'selection' || range === null || sequenceEnd <= 0) return null;
  return range.duration < sequenceEnd * SMALL_SELECTION_SHARE ? range : null;
}

/** The assets played by the sound clips of `sequenceId` that have no transcript yet. */
export function untranscribedAssets(mirror: ProjectMirror, sequenceId: SequenceId): AssetId[] {
  return assetsOf(soundClipsOf(mirror, sequenceId)).filter((assetId) => {
    const asset = mirror.assets[assetId];
    return asset !== undefined && asset.kind !== 'image' && asset.analysis.transcript === null;
  });
}

/** Whether transcription can run: `noModel` (none installed), `unavailable` (whisper.cpp). */
export type SpeechReadiness = 'ready' | 'noModel' | 'unavailable';

/** What keeps media from being transcribed on this machine, if anything. */
export function speechReadiness(status: TranscriptionStatus): SpeechReadiness {
  if (status.whisper.problem !== null) return 'unavailable';
  return status.models.some((model) => model.installed) ? 'ready' : 'noModel';
}

/** When the task's latest run was queued (ms since the epoch). */
function lastQueuedAt(task: Task): number {
  return task.runs.at(-1)?.queuedAt ?? task.createdAt;
}

/**
 * The task a new prompt on `sequenceId` continues by default: the most recently run task of the
 * project, when it works on that sequence, is over, has a session to resume and finished less
 * than {@link CONTINUE_WINDOW_MS} before `now`; else `null`.
 */
export function continuableTask(
  tasks: readonly Task[],
  sequenceId: SequenceId,
  now: number,
): Task | null {
  const latest = tasks.reduce<Task | null>(
    (best, task) => (best === null || lastQueuedAt(task) > lastQueuedAt(best) ? task : best),
    null,
  );
  if (latest === null || isTaskActive(latest) || latest.agentSessionId === null) return null;
  if (latest.scope.sequenceId !== sequenceId) return null;
  const finishedAt = latest.runs.at(-1)?.finishedAt ?? null;
  return finishedAt !== null && now - finishedAt <= CONTINUE_WINDOW_MS ? latest : null;
}
