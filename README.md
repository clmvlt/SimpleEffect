# SimpleEffect

Éditeur vidéo **local** et **pensé pour l'IA**, pour Windows, macOS et Linux.

- Montage non linéaire classique (timeline, textes, keyframes, audio) avec un moteur de rendu identique pour l'aperçu et l'export.
- Montage assisté par une IA locale : l'application pilote un CLI d'IA installé sur ta machine (Claude Code en premier) qui agit uniquement via des outils MCP exposés par le cœur.
- Aucune télémétrie, aucun CDN : tout tourne sur ta machine.

Architecture : application Electron + React, cœur en Rust (`se-core`), rendu dans le renderer (WebCodecs, PixiJS, Web Audio). Détails dans [`docs/architecture.md`](docs/architecture.md).

> Projet en cours de construction : l'état d'avancement est suivi dans [`prompts/README.md`](prompts/README.md).

## Prérequis

- [Node.js](https://nodejs.org/) 24 ou plus récent
- [pnpm](https://pnpm.io/) (la version utilisée est fixée par le champ `packageManager` de `package.json` ; pnpm la télécharge tout seul)
- [Rust](https://rustup.rs/) via rustup (la toolchain est fixée par `rust-toolchain.toml` et s'installe automatiquement)
- Windows : Visual Studio Build Tools (charge de travail « Développement Desktop en C++ »)
- Le dépôt doit être sur un disque **NTFS** (Windows), APFS (macOS) ou ext4 (Linux) : pnpm a besoin de liens symboliques, que l'exFAT ne gère pas.

## Démarrer

```bash
pnpm install
pnpm dev
```

`pnpm dev` compile le cœur Rust, télécharge le binaire Electron au premier lancement, puis ouvre la fenêtre de l'application.

## Commandes utiles

| Commande | Rôle |
|---|---|
| `pnpm dev` | Lance l'application en développement |
| `pnpm check` | Tout ce que vérifie la CI (format, lint, types, tests JS et Rust) |
| `pnpm test` | Tests uniquement |
| `pnpm package` | Construit l'installeur de ta plateforme dans `apps/desktop/release/` |

## Documentation

- [`docs/`](docs/) : architecture, décisions (ADR), glossaire, analyse de la stack
- [`prompts/`](prompts/) : feuille de route, une étape par fichier
- [`GUIDE-GIT.md`](GUIDE-GIT.md) : branches, promotion et publication des releases
- [`CLAUDE.md`](CLAUDE.md) : règles pour les agents IA qui développent le projet

## Téléchargements

Les installeurs (Windows `.exe`, macOS `.dmg`, Linux `.AppImage` et `.deb`) sont publiés sur la [page des releases](https://github.com/clmvlt/SimpleEffect/releases). Ils ne sont pas encore signés : voir la fin de [`GUIDE-GIT.md`](GUIDE-GIT.md) pour les ouvrir malgré l'avertissement du système.
