import type { AssetId, SubjectDetection, VisionStatus } from '@se/protocol';
import { useEffect } from 'react';
import { useStore } from 'zustand';

import { useCoreClient, useCoreEvent } from '../../../shared/core';
import { selectProjectId, useProjectSelector } from '../../../shared/project-store';
import { detectionKey, loadDetection, loadVisionStatus, visionStore } from '../store/vision-store';

/** Whether faces can be detected on this machine (`null` while the core has not answered). */
export function useVisionStatus(): VisionStatus | null {
  const client = useCoreClient();
  const status = useStore(visionStore, (state) => state.status);
  useEffect(() => {
    if (status === null) void loadVisionStatus(client);
  }, [client, status]);
  return status;
}

/**
 * The faces found in `assetId` (`null` while not analysed, or when `assetId` is `null`): read
 * when first needed and again when its face analysis completes.
 */
export function useSubjectDetection(assetId: AssetId | null): SubjectDetection | null {
  const client = useCoreClient();
  const projectId = useProjectSelector(selectProjectId);
  const key = projectId !== null && assetId !== null ? detectionKey(projectId, assetId) : null;
  const known = useStore(visionStore, (state) =>
    key === null ? undefined : state.detections[key],
  );
  useEffect(() => {
    if (key !== null && known === undefined && projectId !== null && assetId !== null) {
      void loadDetection(client, projectId, assetId);
    }
  }, [client, key, known, projectId, assetId]);
  useCoreEvent('jobCompleted', ({ job }) => {
    const analysed = job.kind === 'subjectDetection' && job.projectId === projectId;
    if (analysed && assetId !== null && job.assetId === assetId) {
      void loadDetection(client, job.projectId, assetId);
    }
  });
  return known ?? null;
}
