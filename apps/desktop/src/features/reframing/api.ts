import type {
  AssetId,
  AutoReframeReport,
  CoreClient,
  ItemId,
  ProjectId,
  SubjectDetection,
  TrackingSettings,
  VisionStatus,
} from '@se/protocol';

/** Automatic reframing calls (prompt 26, ADR-0037). */
export interface ReframingApi {
  /** Clips follow their subject: at once when analysed, after the analysis otherwise. */
  readonly reframe: (
    itemIds: readonly ItemId[],
    settings: TrackingSettings,
  ) => Promise<AutoReframeReport>;
  /** The faces found in a media; `null` while it is not analysed. */
  readonly subjectDetection: (assetId: AssetId) => Promise<SubjectDetection | null>;
}

/** The reframing calls of project `projectId` through `client`. */
export function reframingApi(client: Pick<CoreClient, 'send'>, projectId: ProjectId): ReframingApi {
  return {
    reframe: async (itemIds, settings) => {
      const result = await client.send({
        kind: 'autoReframeItems',
        projectId,
        itemIds: [...itemIds],
        settings,
      });
      return {
        appliedItemIds: result.appliedItemIds,
        waitingAssetIds: result.waitingAssetIds,
        skipped: result.skipped,
      };
    },
    subjectDetection: async (assetId) =>
      (await client.send({ kind: 'getSubjectDetection', projectId, assetId })).detection,
  };
}

/** Whether faces can be detected on this machine. */
export async function visionStatus(client: Pick<CoreClient, 'send'>): Promise<VisionStatus> {
  return (await client.send({ kind: 'getVisionStatus' })).status;
}
