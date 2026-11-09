import type { ItemId, TemplateId } from '@se/protocol';

import type { PictureLease } from '../decode/frame-cache';
import type { TemplateLayer } from '../evaluate/frame-graph';

/** One template frame to render at an output size. */
export interface TemplateRenderRequest {
  readonly layer: TemplateLayer;
  /** Output pixels of the template's frame. */
  readonly width: number;
  readonly height: number;
  /** The design frame it draws in (shorter side 1080). */
  readonly designWidth: number;
  readonly designHeight: number;
}

/** Why a template frame shows the error picture instead of its drawing. */
export interface TemplateIssue {
  readonly itemId: ItemId;
  readonly templateId: TemplateId;
  /** What went wrong (English, from the sandbox or the compiler). */
  readonly message: string;
  /** Line of the module, when known. */
  readonly line: number | null;
}

/** A rendered template frame (the error picture when `issue` is set). */
export interface TemplateRenderResult {
  readonly lease: PictureLease;
  readonly issue: TemplateIssue | null;
}

/**
 * What the frame renderer needs from the template runtime (`TemplateRuntime`): cached frames at
 * once, others rendered asynchronously. Kept apart so preview code imports no compiler.
 */
export interface TemplateRenderer {
  /** The cached frame of `request`, if any (with its issue when it is the error picture). */
  peek(request: TemplateRenderRequest): TemplateRenderResult | null;
  /** Renders `request` (never rejects: failures give the error picture and an issue). */
  render(request: TemplateRenderRequest): Promise<TemplateRenderResult>;
}
