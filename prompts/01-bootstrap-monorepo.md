# 01 — Bootstrap du monorepo

> **Phase** : 0 — Fondations · **Dépend de** : — · **Parallélisable avec** : —

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §1 à §4 et §13
- `docs/adr/0001-electron-shell-rust-core-process.md`, `docs/adr/0010-typed-protocol-codegen.md`

## Objectif
Un squelette complet et propre : `pnpm dev` ouvre une fenêtre Electron qui affiche « Cœur connecté — se-core 0.1.0 » après un aller-retour `ping` WebSocket avec le processus Rust `se-core`. Tous les crates et packages existent (vides mais documentés) et `pnpm check` passe.

## Périmètre
**Inclus**
- Vérification des prérequis et installation de Rust si absent.
- Workspaces pnpm et Cargo, outillage qualité, squelettes de tous les modules avec leur `CLAUDE.md`.
- `se-core` minimal (serveur, handshake, ping), supervision par Electron main, renderer React minimal.
- Packaging local, CI GitHub et workflow de release (installeurs Windows/macOS/Linux téléchargeables depuis les releases GitHub).

**Exclus**
- Protocole typé généré → 02 · Design system → 03/04 · toute fonctionnalité métier.

## Spécifications

### Prérequis
- Vérifie `rustc`, `cargo`, `rustup`. S'ils manquent, **demande à l'utilisateur l'autorisation** puis installe rustup (Windows : `winget install Rustlang.Rustup` ; macOS/Linux : script officiel rustup). Toolchain stable, composants `rustfmt` et `clippy`.
- `rust-toolchain.toml` (canal stable figé à la version installée).

### Racine du repo
- `.gitignore` (node_modules, target, dist, out, `resources/bin/`, fichiers de test temporaires, `*.log`), `.gitattributes` (normalisation LF, binaires), `.editorconfig`.
- `package.json` racine (privé) + `pnpm-workspace.yaml` (`apps/*`, `packages/*`). Scripts : `dev`, `build`, `check`, `test`, `lint`, `format`, `format:check`, `typecheck`, `codegen` (placeholder documenté jusqu'au prompt 02), `storybook` (placeholder jusqu'au prompt 03).
- `tsconfig.base.json` : `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `verbatimModuleSyntax`, cible ES2023, résolution `bundler`.
- ESLint (flat config) : `typescript-eslint` strict-type-checked, `eslint-plugin-react-hooks`, `eslint-plugin-boundaries` (règles de frontières entre `features/*` posées dès maintenant, même si vides). Prettier. Vitest configuré par package.
- `Cargo.toml` workspace : membres = tous les crates de `docs/architecture.md` §2 ; `[workspace.package]` (edition 2024, version 0.1.0) ; `[workspace.dependencies]` pour centraliser les versions ; `[workspace.lints]` : `unsafe_code = "forbid"`, clippy `unwrap_used`, `expect_used`, `panic`, `todo`, `dbg_macro` = deny, `pedantic` en warn (avec exceptions justifiées).
- Script `pnpm check` qui enchaîne : prettier --check, eslint, `tsc -b`, vitest run, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. Il doit fonctionner sous Windows (pas de syntaxe bash-only : utilise un petit script Node `scripts/check.mjs`).

### Crates Rust (squelettes)
`se-domain`, `se-protocol`, `se-app`, `se-store`, `se-media`, `se-transcribe`, `se-vision`, `se-agents`, `se-mcp`, `se-export` (lib) et `se-core` (bin). Chacun avec un `lib.rs`/`main.rs` contenant un doc comment de module qui décrit son rôle, ses dépendances autorisées entre crates déjà déclarées dans `Cargo.toml` selon le graphe de l'architecture §3, et un `CLAUDE.md` (gabarit ci-dessous).

### `se-core` minimal
- tokio + axum, écoute sur `127.0.0.1:0` (port aléatoire), jeton aléatoire de 256 bits (crate `rand`, encodé hex).
- Affiche **une seule ligne** JSON sur stdout au démarrage : `{"port":<u16>,"token":"<hex>","version":"0.1.0"}` ; tout le reste des logs va sur stderr via `tracing`.
- `/rpc` en WebSocket : vérifie le jeton (query `?t=`) et l'en-tête `Host` (`127.0.0.1:<port>` ou `localhost:<port>` uniquement) ; répond à `{"type":"request","id":1,"command":{"kind":"ping"}}` par `{"type":"response","id":1,"result":{"kind":"pong","version":"0.1.0"}}`. Types écrits à la main pour l'instant dans `se-protocol` (remplacés au prompt 02).
- **S'arrête proprement quand stdin se ferme** (le parent est mort) et sur Ctrl+C : jamais de processus zombie.

### Application Electron (`apps/desktop`)
- `electron-vite` (main, preload, renderer), React 19, TypeScript, Tailwind CSS v4, React Compiler activé.
- `electron/main/core-supervisor.ts` : lance `se-core` (dev : binaire de `target/debug` après un `cargo build -p se-core` déclenché par le script `dev` ; prod : binaire empaqueté), lit la ligne de handshake, redémarre avec backoff exponentiel en cas de crash, tue le processus à la fermeture de l'app.
- Fenêtre : `contextIsolation: true`, `nodeIntegration: false`, `sandbox: true`, CSP stricte (connexions uniquement vers `127.0.0.1`).
- `electron/preload` : expose via `contextBridge` une API typée minimale `window.simpleEffect.getCoreConnection(): Promise<{ url, token, version }>`.
- Renderer : page unique qui se connecte au WebSocket, envoie `ping`, affiche l'état (connexion…, connecté + version, reconnexion…). Pas encore d'i18n ni de design system : un texte simple suffit, il sera remplacé au prompt 04.

### Packages TS (squelettes)
`packages/protocol` (`@se/protocol`), `packages/engine` (`@se/engine`), `packages/ui` (`@se/ui`) : `package.json`, `tsconfig.json`, `src/index.ts`, `CLAUDE.md`, un test Vitest trivial. `apps/desktop` a aussi son `CLAUDE.md` (rôles de `electron/` et `src/`, structure des features).

### Gabarit des `CLAUDE.md` de module
```markdown
# <module name>

## Purpose
One paragraph: what this module is responsible for (and what it is not).

## Public API
Main entry points (types, functions, traits) — keep updated.

## Rules and invariants
Module-specific rules on top of the root CLAUDE.md.

## Allowed dependencies
Which workspace modules this one may depend on (see docs/architecture.md §3–4).

## How to test
Commands and where tests live.
```

### README racine
Remplace le `README.md` actuel par une présentation courte en français (ce qu'est SimpleEffect, prérequis, `pnpm install`, `pnpm dev`, lien vers `docs/`, `prompts/`, `GUIDE-GIT.md` et la page des releases).

### CI/CD GitHub
Le dépôt `github.com/clmvlt/SimpleEffect` (public) a déjà les branches `main`, `recette` et `dev` ; respecte la section *Git workflow* du `CLAUDE.md`. Les commandes de release côté utilisateur sont décrites dans `GUIDE-GIT.md` : le workflow doit s'y conformer exactement.
- **Packaging local** : `electron-builder` minimal (appId `com.simpleeffect.app`, productName `SimpleEffect`, binaire `se-core` release embarqué en `extraResources`, sans signature) ; script `pnpm package` qui construit l'installeur de la plateforme courante (NSIS `.exe` sous Windows, `.dmg` sous macOS, `.AppImage` + `.deb` sous Linux).
- **Version** : script `scripts/set-version.mjs <version>` qui écrit la version dans tous les `package.json` et dans `[workspace.package]` de `Cargo.toml` ; utilisé par la release à partir du tag.
- **`.github/workflows/ci.yml`** : sur push vers `dev`, `recette`, `main`, `feature/**` et sur pull request ; jobs `ubuntu-latest` et `windows-latest` ; pnpm (version du champ `packageManager`), Node 24, toolchain Rust de `rust-toolchain.toml`, caches pnpm et cargo (`Swatinem/rust-cache`) ; `pnpm install --frozen-lockfile` puis `pnpm check`.
- **`.github/workflows/release.yml`** :
  - déclencheurs : push d'un tag `v*.*.*` (y compris `v1.2.3-rc.1`) et `workflow_dispatch` ;
  - matrice `windows-latest`, `macos-latest`, `ubuntu-latest` : installation, `set-version` (version = tag sans le `v` ; en `workflow_dispatch`, version de dev `0.0.0-dev.<run_number>`), `cargo build --release -p se-core`, `pnpm package`, upload des installeurs en artefacts ;
  - job final (tags uniquement) : création de la release GitHub avec tous les installeurs, **pré-release si le tag contient un `-`** (ex. `-rc.1`), notes générées automatiquement ; `permissions: contents: write` uniquement sur ce job ;
  - actions épinglées sur une version majeure explicite.
- Valide la syntaxe des workflows avec `actionlint` (via `npx` ou binaire) et vérifie que la CI passe sur ta branche après le push (`gh run watch`). Tu ne peux pas pousser de tag : teste le packaging localement avec `pnpm package` et lance l'installeur Windows produit.

## Contraintes
- Tout doit fonctionner sous Windows, macOS et Linux (chemins, scripts, spawn).
- Aucune logique métier dans Electron main ni dans le renderer à ce stade.
- Versions des dépendances : dernières stables au moment de l'exécution ; épingle-les.

## Critères d'acceptation
- [ ] `pnpm install && pnpm dev` ouvre la fenêtre et affiche « connecté » avec la version du cœur.
- [ ] Tuer le processus `se-core` à la main → l'UI affiche « reconnexion… » puis se reconnecte après redémarrage automatique.
- [ ] Fermer l'app ne laisse aucun processus `se-core` actif.
- [ ] Une connexion WebSocket avec un mauvais jeton ou un mauvais `Host` est refusée.
- [ ] `pnpm check` passe sans avertissement.
- [ ] Chaque crate et package a un `CLAUDE.md` rempli.
- [ ] `pnpm package` produit un installeur Windows qui installe et lance l'app (le cœur démarre depuis les ressources empaquetées).
- [ ] La CI est verte sur la branche du prompt et sur `dev` après fusion ; `actionlint` ne remonte rien sur les deux workflows.

## Tests attendus
- Rust : test d'intégration qui démarre le serveur sur un port aléatoire et vérifie ping/pong, refus du mauvais jeton, refus du mauvais `Host`.
- TS : test Vitest du parsing de la ligne de handshake dans le superviseur (fonction pure extraite).

## Documentation à mettre à jour
- `CLAUDE.md` racine (section Commands si un nom de script diffère), `docs/architecture.md` si la structure réelle diffère.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
