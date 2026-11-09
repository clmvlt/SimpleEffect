/**
 * What the main thread and a template's sandboxed Worker exchange (prompt 24, ADR-0035), and the
 * limits of a template's render. The Worker side is serialized into the Worker's source
 * (`sandbox-source.ts`), so it cannot import these types: keep both sides in step by hand.
 */

/** Time budget of one frame during a check, in milliseconds (prompt 24: 50 ms). */
export const TEMPLATE_FRAME_BUDGET_MS = 50;

/**
 * Longest a frame may take outside checks before its Worker is killed (an endless loop): the
 * budget is strict when a template is checked, lenient at playback and export so a slower machine
 * never fails an export.
 */
export const TEMPLATE_WATCHDOG_MS = 1000;

/** Longest a module may take to load (top-level code, fonts) before its Worker is killed. */
export const TEMPLATE_LOAD_WATCHDOG_MS = 3000;

/** An image a template draws, as the template receives it in `params` (or `null`). */
export interface TemplateImageHandle {
  readonly kind: 'image';
  /** Opaque id the drawing API maps to the picture. */
  readonly id: string;
  /** Natural size in pixels. */
  readonly width: number;
  readonly height: number;
}

/** A value of `params` as `render` receives it. */
export type TemplateParamInput = string | number | boolean | TemplateImageHandle | null;

/** A font face sent to a Worker (its bytes: the Worker never fetches anything). */
export interface TemplateFontFace {
  readonly family: string;
  /** CSS `font-weight` descriptor (`400`, `100 900`). */
  readonly weight: string;
  readonly style: 'normal' | 'italic';
  /** CSS `unicode-range` of a subset, or `null` for a whole font. */
  readonly unicodeRange: string | null;
  readonly data: ArrayBuffer;
}

/** Messages to a template Worker. */
export type TemplateWorkerRequest =
  | { readonly type: 'fonts'; readonly id: number; readonly faces: readonly TemplateFontFace[] }
  | {
      readonly type: 'images';
      readonly id: number;
      readonly images: readonly { readonly id: string; readonly bitmap: ImageBitmap }[];
    }
  | {
      readonly type: 'render';
      readonly id: number;
      /** Seconds since the instance's start. */
      readonly time: number;
      /** The instance's length in seconds. */
      readonly duration: number;
      /** Output pixels. */
      readonly width: number;
      readonly height: number;
      /** The design frame the template draws in (shorter side 1080). */
      readonly designWidth: number;
      readonly designHeight: number;
      readonly params: Readonly<Record<string, TemplateParamInput>>;
    };

/** Where a template threw, in its own module's lines (1-based). */
export interface TemplateErrorPlace {
  readonly message: string;
  readonly line: number | null;
  readonly column: number | null;
}

/** Messages from a template Worker. */
export type TemplateWorkerReply =
  | {
      readonly type: 'loaded';
      /** `JSON.stringify(meta)`. */
      readonly metaJson: string;
      /** `params` as `[name, schema]` pairs in declaration order. */
      readonly paramsJson: string;
    }
  | { readonly type: 'loadFailed'; readonly problems: readonly TemplateErrorPlace[] }
  | { readonly type: 'done'; readonly id: number }
  | {
      readonly type: 'rendered';
      readonly id: number;
      readonly bitmap: ImageBitmap;
      /** Time spent in `render`, in milliseconds. */
      readonly elapsedMs: number;
      /** Font families `render` asked for that the Worker has not been given. */
      readonly missingFamilies: readonly string[];
    }
  | { readonly type: 'renderFailed'; readonly id: number; readonly error: TemplateErrorPlace };
