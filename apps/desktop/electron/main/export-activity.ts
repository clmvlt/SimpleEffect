import type { BrowserWindow } from 'electron';

import type { ExportActivity } from '../shared/bridge-api';

/** The system services an {@link ExportActivityGuard} uses (Electron's, or fakes in tests). */
export interface ExportGuardServices {
  /** Keeps the system from suspending the app; returns the blocker's id. */
  readonly preventSuspension: () => number;
  /** Lifts a blocker. */
  readonly allowSuspension: (blockerId: number) => void;
  /** Asks whether to close while exports run: `true` closes. */
  readonly confirmLeave: (window: BrowserWindow | null, activity: ExportActivity) => boolean;
}

/**
 * What the main process does while the renderer exports (prompt 27): the system does not suspend
 * the app (exports run with the window hidden or the screen off), and closing the window or
 * quitting asks first — once per close: the window close that follows a confirmed quit (or the
 * quit that follows a confirmed close) does not ask again.
 */
export class ExportActivityGuard {
  private activity: ExportActivity | null = null;
  private blockerId: number | null = null;
  private leaving = false;

  constructor(private readonly services: ExportGuardServices) {}

  /** Exports are running (with the confirmation's texts), or none any more (`null`). */
  set(activity: ExportActivity | null): void {
    this.activity = activity;
    if (activity !== null && this.blockerId === null) {
      this.blockerId = this.services.preventSuspension();
    }
    if (activity === null && this.blockerId !== null) {
      this.services.allowSuspension(this.blockerId);
      this.blockerId = null;
    }
  }

  /** Whether the window may close (or the app quit) now: asks while exports run. */
  mayLeave(window: BrowserWindow | null): boolean {
    if (this.activity === null || this.leaving) return true;
    this.leaving = this.services.confirmLeave(window, this.activity);
    return this.leaving;
  }
}
