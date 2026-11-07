/**
 * @vitest-environment jsdom
 */
import type { ProjectSession } from '@se/protocol';
import { act, fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it } from 'vitest';

import { SECOND, textKind } from '../../../shared/project-store/testing/graphics-fixture';
import { resetKeyframeLanes } from '../store/keyframe-store';
import { seconds, testItem, testSequence, testTrack } from '../testing/fixtures';
import { renderTimeline, resetTimelineStores, timelineSession } from '../testing/render-timeline';

/** V1 holds the 4 s title `t` whose opacity has keyframes at 0 s and 1 s. */
function titleSession(): ProjectSession {
  const base = timelineSession();
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      sequences: [testSequence(['v1'])],
      tracks: [testTrack('v1', 'video', ['t'])],
      items: [testItem('t', 'v1', 0, 4, { kind: textKind(true) })],
    },
  };
}

afterEach(() => {
  resetTimelineStores();
  resetKeyframeLanes();
});

const sentAnimations = (send: ReturnType<typeof renderTimeline>['send']) =>
  send.mock.calls
    .map(([command]) => command)
    .filter((command) => command.kind === 'setItemAnimations');

describe('keyframe lanes', () => {
  it('unfold under an item, then delete and move its keyframes in one step each', async () => {
    const { send } = renderTimeline(titleSession());
    await userEvent.click(screen.getByRole('button', { name: 'Afficher les images clés' }));
    const lane = screen.getByRole('group', { name: 'Images clés de Bonjour' });
    const diamonds = lane.querySelectorAll<HTMLElement>('[data-keyframe]');
    expect([...diamonds].map((diamond) => diamond.dataset['keyframe'])).toEqual([
      'opacity@0',
      `opacity@${String(SECOND)}`,
    ]);

    // Select the second keyframe, then drag it one second right (50 px per second).
    const second = diamonds[1];
    if (second === undefined) throw new Error('two keyframes');
    fireEvent.pointerDown(second, { clientX: 50, button: 0, pointerId: 1 });
    fireEvent.pointerMove(lane, { clientX: 100, pointerId: 1 });
    fireEvent.pointerUp(lane, { clientX: 100, pointerId: 1 });
    expect(sentAnimations(send)).toMatchObject([
      {
        animations: [
          {
            itemId: 't',
            animation: {
              property: 'opacity',
              value: { kind: 'keyframed', keyframes: [{ time: 0 }, { time: seconds(2) }] },
            },
          },
        ],
      },
    ]);

    // The moved keyframe stays selected once the command resolved.
    await act(async () => {
      await Promise.resolve();
    });
    // `Suppr` deletes the selected keyframes, not the item.
    act(() => {
      fireEvent.pointerDown(diamonds[0] ?? lane, { clientX: 0, button: 0, pointerId: 1 });
      fireEvent.pointerUp(lane, { clientX: 0, pointerId: 1 });
    });
    await userEvent.keyboard('{Delete}');
    const commands = send.mock.calls.map(([command]) => command.kind);
    expect(commands).not.toContain('deleteItems');
    expect(sentAnimations(send).at(-1)).toMatchObject({
      animations: [
        {
          itemId: 't',
          animation: {
            property: 'opacity',
            value: { kind: 'keyframed', keyframes: [{ time: SECOND, value: 1 }] },
          },
        },
      ],
    });
  });
});
