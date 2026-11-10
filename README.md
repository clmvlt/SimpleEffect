# SimpleEffect

Éditeur vidéo **local** et **pensé pour l'IA**, pour Windows, macOS et Linux.

- Montage non linéaire classique (timeline, textes, keyframes, audio) avec un moteur de rendu identique pour l'aperçu et l'export.
- Montage assisté par une IA locale : l'application pilote un CLI d'IA installé sur ta machine (Claude Code en premier) qui agit uniquement via des outils MCP exposés par le cœur.
- Aucune télémétrie, aucun CDN : tout tourne sur ta machine.

Architecture : application Electron + React, cœur en Rust (`se-core`), rendu dans le renderer (WebCodecs, PixiJS, Web Audio). Détails dans [`docs/architecture.md`](docs/architecture.md).

> Projet en cours de construction : l'état d'avancement est suivi dans [`prompts/README.md`](prompts/README.md).

## Installation

Télécharge l'installeur de ta plateforme sur la [page des releases](https://github.com/clmvlt/SimpleEffect/releases) :

| Plateforme | Fichier | Remarques |
|---|---|---|
| Windows 10/11 (x64) | `SimpleEffect-<version>-windows-x64-setup.exe` | Installation pour ton compte, sans droits administrateur. L'installeur n'est pas signé : si SmartScreen s'affiche, « Informations complémentaires » → « Exécuter quand même ». |
| macOS Apple Silicon | `SimpleEffect-<version>-macos-arm64.dmg` | Glisse l'app dans Applications. Elle est signée « ad hoc », pas notariée : au premier lancement, clic droit → « Ouvrir », ou `xattr -dr com.apple.quarantine /Applications/SimpleEffect.app` dans le Terminal. |
| macOS Intel | `SimpleEffect-<version>-macos-x64.dmg` | Comme ci-dessus ; le recadrage automatique n'y est pas disponible (pas d'ONNX Runtime pour les Mac Intel). |
| Linux (x86_64) | `SimpleEffect-<version>-linux-x86_64.AppImage` ou `…-linux-amd64.deb` | AppImage : `chmod +x` puis lance-le. Deb : `sudo apt install ./SimpleEffect-<version>-linux-amd64.deb`. |

Au premier lancement, un assistant vérifie Claude Code (installé ? connecté ?), ffmpeg, le GPU et les codecs, propose les modèles de transcription (téléchargés seulement avec ton accord) et le dossier des projets. Il se rouvre depuis « À propos » (clic sur la version, en bas à droite), qui donne aussi les versions, les licences et « Exporter un diagnostic ».

Désinstaller : Windows « Applications installées », macOS app à la corbeille, Linux suppression de l'AppImage ou `sudo apt remove simpleeffect`. Tes projets et réglages sont conservés.

## Développer

### Prérequis

- [Node.js](https://nodejs.org/) 24 ou plus récent
- [pnpm](https://pnpm.io/) (la version utilisée est fixée par le champ `packageManager` de `package.json` ; pnpm la télécharge tout seul)
- [Rust](https://rustup.rs/) via rustup (la toolchain est fixée par `rust-toolchain.toml` et s'installe automatiquement)
- Windows : Visual Studio Build Tools (charge de travail « Développement Desktop en C++ »)
- Le dépôt doit être sur un disque **NTFS** (Windows), APFS (macOS) ou ext4 (Linux) : pnpm a besoin de liens symboliques, que l'exFAT ne gère pas.

### Démarrer

```bash
pnpm install
pnpm dev
```

`pnpm dev` compile le cœur Rust, télécharge le binaire Electron au premier lancement, puis ouvre la fenêtre de l'application.

### Commandes utiles

| Commande | Rôle |
|---|---|
| `pnpm dev` | Lance l'application en développement |
| `pnpm check` | Tout ce que vérifie la CI (format, lint, types, tests JS et Rust) |
| `pnpm test` | Tests uniquement |
| `pnpm fetch-binaries` | Télécharge ffmpeg, whisper.cpp et ONNX Runtime épinglés (à faire avant `pnpm package`) |
| `pnpm build` | Cœur en release, bundles de l'app et notices de licences |
| `pnpm package` | Construit l'installeur de ta plateforme dans `apps/desktop/release/` (vérifications : [`docs/release-checklist.md`](docs/release-checklist.md)) |

## Documentation

- [`docs/`](docs/) : architecture, décisions (ADR), glossaire, analyse de la stack
- [`prompts/`](prompts/) : feuille de route, une étape par fichier
- [`GUIDE-GIT.md`](GUIDE-GIT.md) : branches, promotion et publication des releases
- [`CLAUDE.md`](CLAUDE.md) : règles pour les agents IA qui développent le projet

## Licences

Les licences des composants embarqués (FFmpeg, whisper.cpp, ONNX Runtime, modèles, polices, paquets npm et crates Rust) sont dans [`LICENSES/`](LICENSES/) et dans l'app (« À propos → Licences »).
