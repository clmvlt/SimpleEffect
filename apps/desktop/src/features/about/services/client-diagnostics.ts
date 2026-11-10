import type { ClientDiagnostics, CodecSupport, ComponentVersion } from '@se/protocol';

import type { RuntimeInfo } from '../../../shared/desktop';

/** A codec probed with WebCodecs for diagnostics and the first-run wizard. */
export interface ProbedCodec {
  /** Name shown and written to the report (`H.264`). */
  readonly name: string;
  /** WebCodecs codec string of a common 1080p profile. */
  readonly codec: string;
}

/** The codecs that matter for editing and export, with a common 1080p profile each. */
export const PROBED_CODECS: readonly ProbedCodec[] = [
  { name: 'H.264', codec: 'avc1.640028' },
  { name: 'HEVC', codec: 'hvc1.1.6.L120.B0' },
  { name: 'VP9', codec: 'vp09.00.40.08' },
  { name: 'AV1', codec: 'av01.0.08M.08' },
];

/** What the renderer can tell about the machine (browser implementations in `browser-probes.ts`). */
export interface ClientProbes {
  /** The GPU's name from WebGL, `null` when WebGL hides it. */
  readonly gpu: () => string | null;
  /** Whether WebCodecs decodes and encodes `codec` here. */
  readonly codecSupport: (codec: ProbedCodec) => Promise<Omit<CodecSupport, 'codec'>>;
  /** The interface language (`navigator.language`). */
  readonly locale: string;
}

/** Versions known to the renderer: the engine and the runtime (when the bridge answered). */
export function componentVersions(
  engineVersion: string,
  runtime: RuntimeInfo | null,
): ComponentVersion[] {
  const components: ComponentVersion[] = [{ name: '@se/engine', version: engineVersion }];
  if (runtime !== null) {
    components.push(
      { name: 'Electron', version: runtime.electron },
      { name: 'Chromium', version: runtime.chrome },
      { name: 'Node.js', version: runtime.node },
      { name: 'V8', version: runtime.v8 },
      { name: 'Platform', version: `${runtime.platform}-${runtime.arch}` },
    );
  }
  return components;
}

/** Every codec of {@link PROBED_CODECS} with its support (a failing probe counts as no). */
export async function probeCodecs(
  probes: Pick<ClientProbes, 'codecSupport'>,
): Promise<CodecSupport[]> {
  return Promise.all(
    PROBED_CODECS.map(async (probed) => {
      try {
        return { codec: probed.name, ...(await probes.codecSupport(probed)) };
      } catch {
        return { codec: probed.name, decode: false, encode: false };
      }
    }),
  );
}

/** The renderer's part of a diagnostics report (no paths, no project content). */
export async function collectClientDiagnostics(
  versions: { readonly app: string; readonly engine: string; readonly runtime: RuntimeInfo | null },
  probes: ClientProbes,
): Promise<ClientDiagnostics> {
  return {
    appVersion: versions.app,
    components: componentVersions(versions.engine, versions.runtime),
    gpu: probes.gpu(),
    codecs: await probeCodecs(probes),
    locale: probes.locale,
  };
}

/** The proposed file name of a diagnostics archive: `SimpleEffect-diagnostic-2026-11-10-1830.zip`. */
export function diagnosticsFileName(now: Date): string {
  const pad = (value: number) => String(value).padStart(2, '0');
  const day = `${String(now.getFullYear())}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
  return `SimpleEffect-diagnostic-${day}-${pad(now.getHours())}${pad(now.getMinutes())}.zip`;
}
