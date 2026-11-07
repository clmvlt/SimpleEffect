/**
 * @vitest-environment jsdom
 */
import type { Animatable, ItemEntity, ProjectSession, Transition } from '@se/protocol';
import { fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { session } from '../../../shared/project-store/testing/session-fixture';
import { setPlayheadTime } from '../store/playhead-store';
import { selectionStore } from '../store/selection-store';
import {
  itemElement,
  renderTimeline as render,
  resetTimelineStores,
  seconds,
} from '../testing/render-timeline';
import { testAsset, testItem, testSequence, testTrack } from '../testing/fixtures';

const clients: { close: () => void }[] = [];

beforeEach(resetTimelineStores);

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
});

const STILL: Animatable<number> = { kind: 'static', value: 0 };

/** A media clip of the test asset playing from 5 s, at `volume`. */
function sound(id: string, start: number, volume: Animatable<number> = STILL): ItemEntity {
  return testItem(id, 'a1', start, 2, {
    kind: {
      kind: 'mediaClip',
      assetId: 'asset-1',
      sourceStart: seconds(5),
      speed: 1,
      transform: {
        position: { kind: 'static', value: { x: 0.5, y: 0.5 } },
        scale: { kind: 'static', value: { x: 1, y: 1 } },
        rotation: { kind: 'static', value: 0 },
        anchor: { kind: 'static', value: { x: 0.5, y: 0.5 } },
        opacity: { kind: 'static', value: 1 },
        blur: { kind: 'static', value: 0 },
      },
      volumeDb: volume,
      fadeIn: seconds(0),
      fadeOut: seconds(0),
      fadeInShape: 'linear',
      fadeOutShape: 'linear',
      reframe: { kind: 'fit' },
    },
  });
}

/**
 * Tracks V1 and A1 (rows: V1 0–52 px, divider, A1 58–110; the volume band of A1 spans 76–108):
 * sounds m1 (0–2 s, volume `volume`) and m2 (2–4 s) on A1, joined by `transitions`.
 */
function soundSession(volume: Animatable<number> = STILL, transitions: Transition[] = []) {
  const base = session();
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      sequences: [testSequence(['v1', 'a1'])],
      tracks: [testTrack('v1', 'video'), testTrack('a1', 'audio', ['m1', 'm2'], { transitions })],
      items: [sound('m1', 0, volume), sound('m2', 2)],
      markers: [],
      assets: [testAsset('asset-1', '/media/plage.mp4')],
    },
  } satisfies ProjectSession;
}

function renderTimeline(snapshot: ProjectSession) {
  const rendered = render(snapshot);
  clients.push(rendered.client);
  return rendered;
}

/** The element matching `selector` inside item `id`. */
function inItem(id: string, selector: string): Element {
  const element = itemElement(id).querySelector(selector);
  if (element === null) throw new Error(`${id} shows ${selector}`);
  return element;
}

describe('volume curves', () => {
  it('adds a keyframe with Ctrl+click on the line', () => {
    const { send, lanes } = renderTimeline(soundSession());
    const line = inItem('m1', '[data-volume-line]');
    // 20 px = 0.4 s, a whole frame at 25 fps.
    fireEvent.pointerDown(line, { button: 0, clientX: 20, clientY: 90, ctrlKey: true });
    fireEvent.pointerUp(lanes, { clientX: 20, clientY: 90, ctrlKey: true });
    expect(send).toHaveBeenCalledWith({
      kind: 'setItemAnimation',
      projectId: 'project-1',
      itemId: 'm1',
      animation: {
        property: 'volumeDb',
        value: {
          kind: 'keyframed',
          keyframes: [{ time: seconds(0.4), value: 0, easing: { kind: 'linear' } }],
        },
      },
    });
  });

  it('moves a keyframe in time and volume', () => {
    const ramp: Animatable<number> = {
      kind: 'keyframed',
      keyframes: [
        { time: seconds(0), value: 0, easing: { kind: 'linear' } },
        { time: seconds(1), value: -6, easing: { kind: 'linear' } },
      ],
    };
    const { send, lanes } = renderTimeline(soundSession(ramp));
    const keyframe = inItem('m1', '[data-volume-keyframe="1"]');
    fireEvent.pointerDown(keyframe, { button: 0, clientX: 50, clientY: 85 });
    fireEvent.pointerMove(lanes, { clientX: 80, clientY: 76 });
    fireEvent.pointerUp(lanes, { clientX: 80, clientY: 76 });
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'setItemAnimation',
        animation: {
          property: 'volumeDb',
          value: {
            kind: 'keyframed',
            keyframes: [
              { time: seconds(0), value: 0, easing: { kind: 'linear' } },
              { time: seconds(1.6), value: 6, easing: { kind: 'linear' } },
            ],
          },
        },
      }),
    );
  });

  it('sizes a fade from its corner handle', () => {
    const { send, lanes } = renderTimeline(soundSession());
    const handle = inItem('m1', '[data-fade-handle="in"]');
    fireEvent.pointerDown(handle, { button: 0, clientX: 2, clientY: 62 });
    fireEvent.pointerMove(lanes, { clientX: 32, clientY: 62 });
    fireEvent.pointerUp(lanes, { clientX: 32, clientY: 62 });
    expect(send).toHaveBeenCalledWith({
      kind: 'updateItems',
      projectId: 'project-1',
      updates: [{ itemId: 'm1', patch: { kind: 'mediaClip', fadeIn: seconds(0.6) } }],
    });
  });
});

describe('transitions', () => {
  it('adds one at the cut under the playhead with Ctrl+T', async () => {
    const { send } = renderTimeline(soundSession());
    setPlayheadTime('sequence-1', seconds(2.2));
    await userEvent.keyboard('{Control>}t{/Control}');
    expect(send).toHaveBeenCalledWith({
      kind: 'addTransitions',
      projectId: 'project-1',
      transitions: [
        {
          trackId: 'a1',
          leftItemId: 'm1',
          rightItemId: 'm2',
          duration: seconds(1),
          alignment: 'center',
        },
      ],
    });
  });

  it('sizes a new cross-fade by dragging from the cut', () => {
    const { send, lanes } = renderTimeline(soundSession());
    const cut = lanes.querySelector('[data-cut-left="m1"]');
    if (cut === null) throw new Error('the cut shows its handle');
    fireEvent.pointerDown(cut, { button: 0, clientX: 100, clientY: 100 });
    fireEvent.pointerMove(lanes, { clientX: 125, clientY: 100 });
    fireEvent.pointerUp(lanes, { clientX: 125, clientY: 100 });
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'addTransitions',
        transitions: [expect.objectContaining({ leftItemId: 'm1', duration: seconds(1) })],
      }),
    );
  });

  it('selects a transition on click and removes it with Suppr', async () => {
    const fade: Transition = {
      id: 't1',
      kind: 'audioCrossfade',
      leftItem: 'm1',
      rightItem: 'm2',
      duration: seconds(1),
      alignment: 'center',
    };
    const { send, lanes } = renderTimeline(soundSession(STILL, [fade]));
    const mark = lanes.querySelector('[data-transition-id="t1"]');
    if (mark === null) throw new Error('the transition is drawn');
    expect(lanes.querySelector('[data-cut-left="m1"]')).toBeNull();
    fireEvent.pointerDown(mark, { button: 0, clientX: 100, clientY: 100 });
    fireEvent.pointerUp(lanes, { clientX: 100, clientY: 100 });
    expect(selectionStore.getState().transitionId).toBe('t1');
    await userEvent.keyboard('{Delete}');
    await waitFor(() => {
      expect(send).toHaveBeenCalledWith({
        kind: 'removeTransitions',
        projectId: 'project-1',
        transitionIds: ['t1'],
      });
    });
  });
});
