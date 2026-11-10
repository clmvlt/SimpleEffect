# Release checklist

Run it on each platform before promoting a release (prompt 30). Use a **fresh user profile** of
the app so the first-run wizard shows: either a machine/account that never ran SimpleEffect, or
start the installed app with `--user-data-dir=<empty folder>` (Windows: `"%LOCALAPPDATA%\Programs\SimpleEffect\SimpleEffect.exe" --user-data-dir=C:\tmp\se-profile`).

Prepare: a short video with sound (e.g. a phone clip, or
`ffmpeg -f lavfi -i testsrc2=size=1920x1080:rate=30 -f lavfi -i sine=frequency=440 -t 10 -c:v libx264 -c:a aac clip.mp4`),
and Claude Code installed and logged in (also try once logged out).

Record the result of each line (✅ / ❌ + note) in the release's PROGRESS entry or issue.

| # | Step | Expected |
|---|---|---|
| 1 | **Install** the installer of the platform (below) | Installs without administrator rights (Windows, AppImage); the app appears in the Start menu / Applications / launcher with its icon |
| 2 | **First launch** | The window opens; the status bar says « Cœur connecté »; the wizard « Bienvenue dans SimpleEffect » opens |
| 3 | Wizard → **Assistant IA** | Claude Code « Trouvé et utilisable » and « Prêt à recevoir des tâches »; logged out: the `claude` / `/login` instructions; not installed: « Comment installer » opens the guide in the browser |
| 4 | Wizard → **Binaires et GPU** | ffmpeg 9.0.2 found (hardware encoders listed when present), the GPU named, H.264 decoded; face detection available (unavailable with a reason on Intel Macs) |
| 5 | Wizard → **Modèles** | The Whisper models listed with sizes; a download asks first (size, source), shows progress and can be cancelled |
| 6 | Wizard → **Dossier des projets** → « Choisir un autre dossier… » → Terminer | The folder shows; the wizard does not open again at the next launch |
| 7 | **Create a project** in that folder | The editor opens; the `.seproj` folder exists |
| 8 | **Import** the clip (button and drag and drop) | Thumbnail, waveform, duration; placed on the timeline |
| 9 | **Playback** (Space, J/K/L) | Picture and sound in sync, no frozen frame |
| 10 | **AI task**: « Coupe les 2 premières secondes » | The task runs, the clip is trimmed, the change is undoable |
| 11 | **Video export** (Exporter → default preset) | The MP4 plays in the system player; « Afficher » reveals it |
| 12 | **Premiere export** | N/A until prompt 28 (FCP7 XML) is done |
| 13 | **À propos** (click the version in the status bar) | Versions of the app, se-core, engine, Electron, Chromium and Claude Code; « Licences » lists components, npm packages and crates with their texts; « Ouvrir le dossier des licences » opens `resources/LICENSES` |
| 14 | **Exporter un diagnostic** | A zip is written; it holds `README.txt`, `report.json`, `logs/`; no user name, project or media name, token or e-mail inside (search the files) |
| 15 | **Uninstall** | The app and its shortcuts are gone; projects and `%APPDATA%\simpleeffect` (macOS `~/Library/Application Support/simpleeffect`, Linux `~/.config/simpleeffect`) are kept |

## Platforms

### Windows 10/11 (x64)
- File: `SimpleEffect-<version>-windows-x64-setup.exe`. Per-user install in
  `%LOCALAPPDATA%\Programs\SimpleEffect` by default.
- The installer is not signed by a certificate: SmartScreen shows « Windows a protégé votre
  ordinateur » → « Informations complémentaires » → « Exécuter quand même ».
- whisper.cpp needs the Visual C++ Redistributable 2015–2022 (x64); the transcription settings say
  so when it is missing.
- Silent install for tests: `SimpleEffect-<version>-windows-x64-setup.exe /S /D=C:\tmp\se-install`;
  uninstall: `"C:\tmp\se-install\Uninstall SimpleEffect.exe" /S`.

### macOS (Apple Silicon and Intel)
- Files: `SimpleEffect-<version>-macos-arm64.dmg`, `SimpleEffect-<version>-macos-x64.dmg`. Drag the
  app to Applications.
- The app is ad-hoc signed, not notarised. Gatekeeper workaround (personal use): right-click the
  app → « Ouvrir » → « Ouvrir », or
  `xattr -dr com.apple.quarantine /Applications/SimpleEffect.app`.
- Intel Macs have no ONNX Runtime build: automatic reframing is unavailable there (step 4 expects
  it).
- Uninstall: move the app to the Bin.

### Linux (x86_64)
- Files: `SimpleEffect-<version>-linux-x86_64.AppImage` (`chmod +x`, then run it) or
  `SimpleEffect-<version>-linux-amd64.deb` (`sudo apt install ./SimpleEffect-<version>-linux-amd64.deb`).
- Hardware decoding depends on VA-API drivers; the wizard's codecs row shows what works.
- Uninstall: delete the AppImage, or `sudo apt remove simpleeffect`.

## Results

| Version | Platform | Date | Result |
|---|---|---|---|
| 0.1.0 | Windows 11 x64 (dev machine, RTX 3080) | 2026-11-10 | ✅ 14/14 applicable steps (12 N/A); details in `prompts/PROGRESS.md`, prompt 30 |
