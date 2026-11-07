import type { AssetId } from '@se/protocol';

import type { DecodeFailure } from '../decode/decode-error';

/** Why a picture layer was drawn without its media. */
export type MediaIssueKind =
  /** Decodability check or mezzanine still running. */
  | 'preparing'
  /** The original is offline (no proxy to fall back on). */
  | 'missing'
  /** The original is not decodable and its mezzanine failed. */
  | 'mezzanineFailed'
  /** The file was found but could not be read or decoded. */
  | 'decodeFailed';

/** A media problem of the last drawn frame, reported once per asset. */
export interface MediaIssue {
  readonly assetId: AssetId;
  readonly kind: MediaIssueKind;
  /** For `decodeFailed`: what went wrong (no URL, no token). */
  readonly failure?: DecodeFailure;
  readonly message?: string;
}

/** Collects issues of one frame, one per asset (the first wins). */
export class MediaIssues {
  private readonly byAsset = new Map<AssetId, MediaIssue>();

  add(issue: MediaIssue): void {
    if (!this.byAsset.has(issue.assetId)) this.byAsset.set(issue.assetId, issue);
  }

  list(): readonly MediaIssue[] {
    return [...this.byAsset.values()];
  }
}
