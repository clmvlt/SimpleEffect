# 21 — Transcription locale (whisper.cpp)

> **Phase** : 5 — Transcription et audio intelligent · **Dépend de** : 09, 18 · **Parallélisable avec** : 19, 20

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §7, §9.4, §11, §14
- `docs/adr/0007-media-pipeline-sidecars-mezzanine.md`
- `CLAUDE.md` de `se-transcribe`, `se-media`, `se-store`, `se-mcp`

## Objectif
Transcrire localement, au mot près, la parole des médias (accélération GPU quand elle existe), afficher la transcription synchronisée avec la lecture, et la rendre disponible aux agents IA pour qu'ils puissent couper intelligemment.

## Périmètre
**Inclus**
- Binaires whisper.cpp par plateforme, gestionnaire de modèles, adaptateur `Transcriber`, stockage, panneau Transcription, outil MCP `get_transcript` réel.

**Exclus**
- Montage par le texte, mots de remplissage, suppression des silences → 22 · sous-titres → 23 · identification des locuteurs → backlog.

## Spécifications

### Binaires et modèles
- Étendre `scripts/binaries.manifest.json` et `pnpm fetch-binaries` avec whisper.cpp : Windows (build CUDA + build CPU de repli), macOS (Metal), Linux (CPU, CUDA si disponible) ; versions et SHA-256 épinglés ; **demander l'accord** avant téléchargement.
- Mettre à jour `.github/workflows/release.yml` pour embarquer les binaires whisper.cpp de chaque plateforme (les modèles ne sont **pas** embarqués : téléchargés par l'utilisateur).
- Gestionnaire de modèles (`features/settings`, section Transcription) : liste (tiny, base, small, medium, large-v3-turbo) avec tailles et recommandation selon le matériel ; téléchargement depuis le dépôt officiel des modèles ggml **uniquement après consentement explicite** (taille affichée), vérification de l'empreinte, ou import d'un fichier local ; stockage dans `<app data>/models/whisper/`.
- Détection du backend disponible (CUDA/Metal/CPU) affichée dans les réglages.

### Adaptateur (`se-transcribe`)
- Port `Transcriber` : `transcribe(audio_path, options) -> flux de progression + Transcript`.
- Extraction audio 16 kHz mono WAV via ffmpeg (`se-media`), puis whisper.cpp avec sortie JSON complète (timestamps par token), option d'alignement amélioré (DTW) si supportée par la version.
- Fusion tokens → **mots** (gestion des espaces, apostrophes françaises, ponctuation), confiance par mot, segments.
- Langue : auto ou forcée (défaut `fr`, réglable par projet).
- Job `Transcription` dans la `JobQueue` (progression, annulation) ; lancement manuel (« Transcrire ») ou automatique à l'import (réglage).

### Stockage
- `project.db` : tables mots (asset, mot, début, fin, confiance, segment) et segments, index FTS5 pour la recherche ; versionné par modèle/langue (retranscrire remplace).

### Panneau Transcription (`features/transcript`)
- Transcription de la séquence active (mots des médias **ramenés au temps de la séquence** via les clips : un mot n'apparaît que si sa portion de média est utilisée) ou d'un média sélectionné.
- Mot courant surligné pendant la lecture, clic sur un mot = déplacement de la tête de lecture, sélection de mots = sélection de la plage correspondante (in/out), recherche, mots peu fiables soulignés.

### MCP
- `get_transcript { range }` réel : segments et mots dans le temps de la séquence, paginé ; mise à jour de `docs/ai-tools.md`.

## Contraintes
- Aucun appel réseau sans consentement ; aucune donnée audio ne quitte la machine.
- La correspondance média ↔ séquence est une fonction pure partagée (réutilisée aux prompts 22 et 23).

## Critères d'acceptation
- [ ] Une vidéo de 10 minutes en français est transcrite sur le GPU de la machine de dev en un temps raisonnable (mesure notée dans PROGRESS.md).
- [ ] Le panneau suit la lecture au mot près ; cliquer un mot positionne la tête de lecture.
- [ ] Après avoir coupé un passage, ses mots disparaissent de la transcription de la séquence.
- [ ] Un agent peut lire la transcription d'une plage via `get_transcript`.

## Tests attendus
- Fusion tokens → mots sur des fixtures JSON whisper (français : apostrophes, nombres, ponctuation).
- Correspondance temps média ↔ séquence (clips coupés, vitesse, éléments liés).
- Gestionnaire de modèles (vérification d'empreinte, import).

## Documentation à mettre à jour
- `crates/se-transcribe/CLAUDE.md`, `docs/ai-tools.md`, `docs/architecture.md` si besoin, `CLAUDE.md` racine (binaires).

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
