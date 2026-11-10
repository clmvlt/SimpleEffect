import type { ClientProbes, ProbedCodec } from './client-diagnostics';

/** The GPU's name as WebGL reports it (`ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 …)`). */
function webglRenderer(): string | null {
  try {
    const gl = document.createElement('canvas').getContext('webgl');
    if (gl === null) return null;
    const info = gl.getExtension('WEBGL_debug_renderer_info');
    const name: unknown = gl.getParameter(
      info === null ? gl.RENDERER : info.UNMASKED_RENDERER_WEBGL,
    );
    gl.getExtension('WEBGL_lose_context')?.loseContext();
    return typeof name === 'string' && name !== '' ? name : null;
  } catch {
    return null;
  }
}

async function webCodecsSupport({
  codec,
}: ProbedCodec): Promise<{ decode: boolean; encode: boolean }> {
  const decode =
    typeof VideoDecoder === 'undefined'
      ? false
      : ((await VideoDecoder.isConfigSupported({ codec, codedWidth: 1920, codedHeight: 1080 }))
          .supported ?? false);
  const encode =
    typeof VideoEncoder === 'undefined'
      ? false
      : ((
          await VideoEncoder.isConfigSupported({
            codec,
            width: 1920,
            height: 1080,
            bitrate: 8_000_000,
            framerate: 30,
          })
        ).supported ?? false);
  return { decode, encode };
}

/** The probes of this window: WebGL for the GPU, WebCodecs for codecs. */
export function browserProbes(): ClientProbes {
  return { gpu: webglRenderer, codecSupport: webCodecsSupport, locale: navigator.language };
}
