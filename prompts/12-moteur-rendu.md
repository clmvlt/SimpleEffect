# 12 — Moteur A : évaluation, décodage, compositing, viewer ⚠️

> **Phase** : 3 — Montage · **Dépend de** : 09, 11 · **Parallélisable avec** : 17

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §8
- `docs/adr/0002-web-rendering-engine.md`, `docs/adr/0012-*` (unités spatiales)
- `packages/engine/CLAUDE.md`, `prompts/PROGRESS.md`

## Objectif
Afficher dans le viewer l'image exacte de la séquence à la position de la tête de lecture (scrub précis à l'image), avec le moteur `@se/engine` qui servira aussi à l'export.

## Périmètre
**Inclus**
- `evaluate`, interpolation des keyframes, pool de décodeurs, compositeur PixiJS, choix de la source (original/mezzanine/proxy), viewer, tests golden frame.

**Exclus**
- Lecture temps réel et audio → 13 · rendu avancé des textes → 15 · export → 27.

## Spécifications

### Évaluation (`packages/engine/src/evaluate/`)
- `evaluate(sequence, assets, time): FrameGraph` **pure** : éléments visibles à `time` (pistes non masquées, ordre de superposition), temps source des clips (`source_start + (time - start) * speed`), propriétés résolues (transform, opacité), mode de recadrage.
- `animation/interpolate.ts` : linear, hold, cubic-bezier (résolution numérique robuste), presets d'easing ; interpolation de nombres, vecteurs, couleurs (en OKLab).
- **Vecteurs de test partagés** `docs/test-vectors/interpolation.json` (créés ici) pour une future parité avec Rust.

### Décodage (`packages/engine/src/decode/`)
- `DecoderPool` : un décodeur Mediabunny par (asset, variante) avec `UrlSource` vers les routes `/media` (jeton), accès précis à l'image par timestamp, cache LRU de `VideoFrame` avec budget mémoire configurable, **fermeture systématique** des frames évincées, annulation des requêtes obsolètes lors d'un scrub rapide.
- Sélection de la variante : original si décodable, sinon mezzanine ; proxy si l'option « Proxy » du viewer est active et qu'il existe.
- Images fixes (assets image) chargées en `ImageBitmap`.

### Compositing (`packages/engine/src/composite/`)
- Interface `Compositor { resize, render(frameGraph, frames), destroy }` ; implémentation `PixiCompositor` (PixiJS v8, WebGL2 par défaut, WebGPU en option) fonctionnant sur `HTMLCanvasElement` et `OffscreenCanvas`.
- Dessin : clips vidéo (textures depuis `VideoFrame`), images, transformations (position normalisée, échelle, rotation, ancre), opacité, recadrage (fit/fill/custom), fond noir ; texte basique (police, taille, couleur) en attendant le prompt 15.

### Viewer (`features/viewer`)
- Canvas au ratio de la séquence, ajusté au panneau, niveaux de zoom (ajuster, 50 %, 100 %, 200 %), fond damier optionnel.
- Rendu à chaque changement de tête de lecture (scrub) ; indicateur de chargement si l'image n'est pas encore décodée ; bascule Proxy ; timecode courant.
- Le moteur vit hors de React : un hook fin crée/détruit le moteur et lui transmet séquence et temps.

### Tests golden frame
- Vitest en mode navigateur (Chromium via Playwright) : séquences synthétiques (aplats, mire, vidéo de test avec **numéro d'image incrusté** générée par ffmpeg) rendues et comparées à des PNG de référence avec tolérance ; dossier `packages/engine/test/golden/`.

## Contraintes
- `@se/engine` n'importe ni React ni `@se/ui`.
- Aucune fuite de `VideoFrame` (compteur de frames ouvertes exposé en dev et vérifié en test).

## Critères d'acceptation
- [ ] Le scrub affiche exactement la bonne image (vérifié avec la vidéo à numéro d'image incrusté, y compris en 29,97 i/s).
- [ ] Superposition de plusieurs pistes, transformations et opacité correctes (golden frames).
- [ ] La bascule Proxy change la source sans changer le cadrage.
- [ ] Aucune frame non fermée après 1 minute de scrub intensif.

## Tests attendus
- Unitaires : `evaluate` (visibilité, temps source, vitesse), interpolation (vecteurs partagés).
- Golden frames : 6 à 10 scènes de référence.
- Test de non-fuite du `DecoderPool`.

## Documentation à mettre à jour
- `packages/engine/CLAUDE.md` (architecture interne, règles de cycle de vie des frames), `docs/architecture.md` §8 si besoin.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
