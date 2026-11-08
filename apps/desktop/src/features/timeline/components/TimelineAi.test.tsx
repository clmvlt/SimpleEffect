/**
 * @vitest-environment jsdom
 */
import { act, fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import {
  aiTasksStore,
  composerRequestStore,
  markUserEdited,
  pulseItems,
  resetAiTasks,
  setProjectTasks,
} from '../../../shared/ai-tasks';
import { sidePanelStore } from '../../../shared/side-panel';
import { task } from '../../../shared/ai-tasks/testing/task-fixture';
import { aiScopeBoxes } from '../services/ai-scope-boxes';
import { layoutTracks } from '../services/track-layout';
import { seconds, testTrack } from '../testing/fixtures';
import {
  itemElement,
  renderTimeline,
  resetTimelineStores,
  timelineSession,
} from '../testing/render-timeline';

const clients: { close: () => void }[] = [];

beforeEach(resetTimelineStores);

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  resetAiTasks();
});

/** The timeline session with c2 made by task-1. */
function aiSession() {
  const base = timelineSession();
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      items: base.snapshot.items.map((item) =>
        item.id === 'c2'
          ? { ...item, origin: { kind: 'aiTask' as const, taskId: 'task-1' } }
          : item,
      ),
    },
  };
}

function render() {
  const rendered = renderTimeline(aiSession());
  clients.push(rendered.client);
  return rendered;
}

const RANGED = {
  sequenceId: 'sequence-1',
  range: { start: seconds(2), duration: seconds(3) },
  tracks: null,
};

describe('AI tasks on the timeline', () => {
  it('shades the scopes of running and queued tasks, with a mark after a user edit', async () => {
    render();
    act(() => {
      setProjectTasks('project-1', [
        task({ status: { kind: 'running' }, scope: RANGED }),
        task({ id: 'task-2', createdAt: 2, scope: { ...RANGED, tracks: ['v1'] } }),
        task({ id: 'task-3', createdAt: 3, status: { kind: 'completed' }, scope: RANGED }),
      ]);
    });
    expect(document.querySelectorAll('[data-ai-scope="task-1"]')).toHaveLength(1);
    expect(document.querySelectorAll('[data-ai-scope="task-2"]')).toHaveLength(1);
    expect(document.querySelector('[data-ai-scope="task-3"]')).toBeNull();
    expect(screen.getByRole('button', { name: 'IA #1 en cours' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'IA #2 en file' })).toBeInTheDocument();
    act(() => {
      markUserEdited(['task-1']);
    });
    const label = screen.getByRole('button', { name: /IA #1 en cours/ });
    expect(label).toHaveAccessibleName(/Vous avez modifié cette zone/);
    await userEvent.click(label);
    expect(aiTasksStore.getState().focus?.taskId).toBe('task-1');
    expect(sidePanelStore.getState().tab).toBe('ai');
  });

  it('badges items made by a task (a click shows it) and makes changed items pulse', async () => {
    render();
    act(() => {
      setProjectTasks('project-1', [task({ status: { kind: 'completed' } })]);
    });
    const badge = screen.getByRole('button', { name: /Créé par la tâche #1/ });
    await userEvent.click(badge);
    expect(aiTasksStore.getState().focus?.taskId).toBe('task-1');
    expect(itemElement('c2').querySelector('[data-ai-pulse]')).toBeNull();
    act(() => {
      pulseItems(['c2']);
    });
    expect(itemElement('c2').querySelector('[data-ai-pulse]')).toBeInTheDocument();
  });

  it('offers « Refaire avec l’IA » in the context menu of an item', async () => {
    render();
    fireEvent.contextMenu(itemElement('c2'));
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Refaire avec l’IA…' }));
    expect(composerRequestStore.getState().request?.itemId).toBe('c2');
    expect(sidePanelStore.getState().tab).toBe('ai');
  });
});

describe('aiScopeBoxes', () => {
  const tracks = [testTrack('v1', 'video', []), testTrack('v2', 'video'), testTrack('a1', 'audio')];
  const layout = layoutTracks(tracks, () => 52);
  const viewport = { pixelsPerSecond: 50, scrollTime: seconds(0), width: 500 };

  it('spans every row without tracks, one box per track otherwise, clamped to the view', () => {
    const whole = aiScopeBoxes([task({ scope: { ...RANGED, range: null } })], layout, viewport);
    expect(whole).toEqual([
      expect.objectContaining({
        left: 0,
        width: 700,
        top: 0,
        height: layout.height,
        labelled: true,
      }),
    ]);
    const some = aiScopeBoxes(
      [task({ scope: { ...RANGED, tracks: ['a1', 'v1'] } })],
      layout,
      viewport,
    );
    expect(some.map((box) => [box.left, box.width, box.labelled])).toEqual([
      [100, 150, true],
      [100, 150, false],
    ]);
    const before = aiScopeBoxes([task({ scope: RANGED })], layout, {
      ...viewport,
      scrollTime: seconds(10),
    });
    expect(before).toEqual([]);
  });
});
