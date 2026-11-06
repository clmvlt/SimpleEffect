/**
 * The EyeDropper API (Chromium, hence Electron) picks a colour anywhere on screen. It is not in
 * TypeScript's DOM library, so this module declares the part we use and detects it at runtime.
 */
interface EyeDropperInstance {
  open: () => Promise<{ sRGBHex: string }>;
}

type EyeDropperConstructor = new () => EyeDropperInstance;

function getEyeDropperConstructor(): EyeDropperConstructor | undefined {
  const candidate: unknown = Reflect.get(globalThis, 'EyeDropper');
  return typeof candidate === 'function' ? (candidate as EyeDropperConstructor) : undefined;
}

/** Whether the screen colour picker is available. */
export function isEyeDropperSupported(): boolean {
  return getEyeDropperConstructor() !== undefined;
}

/**
 * Lets the user pick a colour on screen. Resolves with its hex text, or `null` when the user
 * cancels (Escape) or the API is missing.
 */
export async function pickScreenColor(): Promise<string | null> {
  const EyeDropper = getEyeDropperConstructor();
  if (EyeDropper === undefined) {
    return null;
  }
  try {
    const { sRGBHex } = await new EyeDropper().open();
    return sRGBHex;
  } catch {
    // The user dismissed the picker: not an error.
    return null;
  }
}
