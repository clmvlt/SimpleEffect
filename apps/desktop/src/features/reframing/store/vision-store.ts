import type { CoreClient, SubjectDetection, VisionStatus } from '@se/protocol';
import { createStore } from 'zustand/vanilla';

import { reframingApi, visionStatus } from '../api';

/** What the reframing feature keeps: whether faces can be detected, and the faces of media. */
export interface VisionState {
  /** `null` until asked. */
  readonly status: VisionStatus | null;
  /** Faces by `<project>:<asset>`: `null` while the media is not analysed. */
  readonly detections: Readonly<Record<string, SubjectDetection | null>>;
}

/** The reframing feature's store (shared by the inspector, the viewer and the version dialog). */
export const visionStore = createStore<VisionState>(() => ({ status: null, detections: {} }));

let statusRequest: Promise<void> | null = null;

/** Asks the core whether faces can be detected (once; again after a failure). */
export function loadVisionStatus(client: Pick<CoreClient, 'send'>): Promise<void> {
  statusRequest ??= visionStatus(client)
    .then((status) => {
      visionStore.setState({ status });
    })
    .catch(() => {
      statusRequest = null;
    });
  return statusRequest;
}

/** The key of a media's faces. */
export function detectionKey(projectId: string, assetId: string): string {
  return `${projectId}:${assetId}`;
}

/** Reads the faces of `assetId` (again, replacing what was kept). */
export async function loadDetection(
  client: Pick<CoreClient, 'send'>,
  projectId: string,
  assetId: string,
): Promise<void> {
  try {
    const detection = await reframingApi(client, projectId).subjectDetection(assetId);
    visionStore.setState((state) => ({
      detections: { ...state.detections, [detectionKey(projectId, assetId)]: detection },
    }));
  } catch {
    // The viewer simply shows no faces.
  }
}

/** Forgets every kept answer (tests, core restarts). */
export function resetVisionStore(): void {
  statusRequest = null;
  visionStore.setState({ status: null, detections: {} });
}
