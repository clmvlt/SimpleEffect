# Licences of the components shipped with SimpleEffect

SimpleEffect bundles or downloads the components below. The app shows all of them, with the
npm and Cargo packages compiled into it, in « À propos → Licences »; installers carry this folder
and the generated `THIRD-PARTY-NOTICES.txt` in their `LICENSES` resources folder.

| Component | Licence | Text |
|---|---|---|
| FFmpeg 9.0.2 (`ffmpeg`, `ffprobe`) | GPL-3.0-or-later | `GPL-3.0.txt` |
| whisper.cpp 1.9.5 | MIT | `whisper.cpp-MIT.txt` |
| ONNX Runtime 1.31.0 | MIT | `onnxruntime-MIT.txt` (+ `bin/onnxruntime/ThirdPartyNotices.txt` in the app) |
| YuNet face detection model | MIT | `yunet-MIT.txt` |
| Whisper models (downloaded on request) | MIT | `whisper-models-MIT.txt` |
| Silero VAD 6.2.0 (downloaded on request) | MIT | `silero-vad-MIT.txt` |
| Electron 45 (Chromium, Node.js) | MIT, Chromium notices | `LICENSE.electron.txt`, `LICENSES.chromium.html` next to the app |
| Bundled fonts (Inter, Montserrat, Oswald, Bebas Neue, Playfair Display, Permanent Marker) | OFL-1.1, Apache-2.0 | `crates/se-fonts/fonts/*/LICENSE`, in the generated notices |
| npm and Cargo packages | MIT, Apache-2.0, BSD, ISC… | generated `THIRD-PARTY-NOTICES.txt` |

## FFmpeg

FFmpeg runs as separate, unmodified programs (SimpleEffect starts them; it does not link to
them). The builds are pinned in `scripts/binaries.manifest.json` (URL and SHA-256):

- Windows: Gyan.dev « essentials » builds — https://www.gyan.dev/ffmpeg/builds/
- macOS: Martin Riedl's builds — https://ffmpeg.martin-riedl.de
- Linux: BtbN FFmpeg-Builds — https://github.com/BtbN/FFmpeg-Builds

FFmpeg's source code: https://ffmpeg.org/download.html (and each builder's page above, which
publishes the configuration it used).

## Generated notices

`scripts/generate-notices.mjs` (run by `pnpm build`) writes `third-party-notices.json` and
`THIRD-PARTY-NOTICES.txt` into `apps/desktop/out/notices/`: the components of
`components.json`, every npm package bundled into the app (listed by the Vite plugin
`apps/desktop/tooling/bundled-packages-plugin.ts`) and every Cargo crate linked into `se-core`
(from `cargo metadata`), each with its licence files.
