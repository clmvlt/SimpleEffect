# 08 — Médias 1 : import, bin, jobs, miniatures, waveforms

> **Phase** : 2 — Médias · **Dépend de** : 07 · **Parallélisable avec** : 17

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §7, §13, §14
- `docs/adr/0007-media-pipeline-sidecars-mezzanine.md`
- `CLAUDE.md` de `se-media`, `se-app`, `se-core`, `apps/desktop`

## Objectif
Importer des fichiers (bouton ou glisser-déposer depuis l'OS), les voir dans un panneau Médias avec miniatures et métadonnées, pendant que des tâches de fond génèrent miniatures et formes d'onde, le tout servi au renderer par le cœur.

## Périmètre
**Inclus**
- Script de récupération des binaires ffmpeg/ffprobe, adaptateur `se-media` (probe, miniatures, pics de waveform), `JobQueue`, routes `/media`, panneau Médias, rapport de décodabilité, reliaison des médias manquants.

**Exclus**
- Mezzanines, proxies, analyses audio/scènes → 09 · placement sur la timeline → 11.

## Spécifications

### `pnpm fetch-binaries`
- Script Node `scripts/fetch-binaries.mjs` + manifeste `scripts/binaries.manifest.json` : par plateforme (`win32-x64`, `darwin-arm64`, `darwin-x64`, `linux-x64`), URL de builds statiques ffmpeg/ffprobe reconnus, version épinglée, **SHA-256 épinglé**, extraction dans `resources/bin/<platform>/`.
- Le script affiche ce qu'il va télécharger (taille, source) et demande confirmation. **L'agent qui exécute ce prompt doit demander l'accord de l'utilisateur avant de lancer le téléchargement.**
- Vérification de licence documentée (builds GPL acceptables : usage personnel).
- Option `--yes` (non interactive) pour la CI ; `.github/workflows/release.yml` récupère les binaires de chaque plateforme avant `pnpm package` (avec cache) et `electron-builder` les embarque en `extraResources`.

### `se-media`
- Résolution des binaires : chemin configuré par l'utilisateur → `resources/bin/<platform>/` → `PATH` ; erreur claire si absent.
- Lancement de processus via un helper unique (pas de fenêtre console sous Windows, arrêt de l'arbre de processus à l'annulation, capture stderr pour les erreurs).
- `MediaProber` (port) : `ffprobe -show_format -show_streams -of json` → `MediaInfo` du domaine.
- Jobs :
  - `Thumbnails` : planche (sprite) de N vignettes à intervalles réguliers (JPEG, hauteur 90 px) + poster ; métadonnées de la planche en JSON.
  - `WaveformPeaks` : ffmpeg → PCM f32 sur stdout → pics min/max par canal à plusieurs niveaux (ex. 256, 1024, 4096 échantillons/pic) → fichier binaire au format documenté dans le `CLAUDE.md` du crate.
  - Progression via `-progress pipe:1`.
- Empreinte de contenu : taille + mtime + xxh3 du premier et du dernier MiB (clé du cache).

### `se-app`
- `JobQueue` : priorités, concurrence par type de job, annulation, une nouvelle tentative, événements `JobQueued`, `JobProgress`, `JobCompleted`, `JobFailed` ; les jobs d'un projet s'arrêtent à sa fermeture.
- `MediaService` : `import_files(paths)` (déduplication par empreinte, opérations `AddAsset`, lancement des jobs), `relink_asset(asset, new_path)` (vérifie que l'empreinte correspond ou demande confirmation), `remove_asset` (refusé si utilisé), détection des médias manquants à l'ouverture.

### `se-core`
- Routes `/media/assets/{id}/original`, `/media/assets/{id}/thumbnails` (planche + JSON), `/media/assets/{id}/peaks/{level}` ; support Range (206/416) ; jeton en query `?t=` ; types MIME corrects.

### Protocole
- Commandes : `ImportMedia { paths }`, `RelinkAsset`, `RemoveAsset`, `ListJobs`, `CancelJob`, `ReportDecodability { asset, video, audio }`.
- Événements des jobs ; patchs projet pour les assets.

### Renderer
- `@se/engine/capabilities` : construit la chaîne de codec WebCodecs à partir de `MediaInfo` (avc1/hvc1/vp09/av01, profils, bit depth) et appelle `VideoDecoder.isConfigSupported` / `AudioDecoder.isConfigSupported` → `ReportDecodability`.
- `features/media-bin` :
  - Import par bouton (dialogue natif multi-sélection) et glisser-déposer depuis l'OS (chemins via `webUtils.getPathForFile` exposé par le preload).
  - Vue grille et liste, survol d'une vignette = défilement dans la planche, filtre par type, recherche par nom.
  - Panneau d'infos : codec, résolution, cadence, durée, audio, chemin, décodabilité.
  - Badges de progression des jobs, indicateur « média manquant » + action « Relier… ».

## Contraintes
- Aucun appel réseau à l'exécution (le seul téléchargement est le script de dev, avec accord).
- Les tests qui ont besoin de ffmpeg génèrent leurs médias de test (`testsrc`, `sine`) et sont ignorés avec un message si le binaire est absent.

## Critères d'acceptation
- [ ] Glisser trois fichiers (dont un 4K) → ils apparaissent, vignettes et waveforms générées avec progression visible.
- [ ] Un fichier déjà importé n'est pas dupliqué.
- [ ] Renommer un fichier source → indicateur « manquant » à la réouverture → « Relier… » le répare.
- [ ] Les routes `/media` répondent correctement aux requêtes Range et refusent un mauvais jeton.

## Tests attendus
- Parsing ffprobe sur des fixtures JSON commitées (vidéo H.264, HEVC 10 bits, ProRes, audio seul, image).
- Calcul des pics sur un signal synthétique ; format binaire aller-retour.
- `JobQueue` avec un faux runner (priorités, annulation, nouvelle tentative).
- Intégration HTTP Range ; test renderer de la construction des chaînes de codec.

## Documentation à mettre à jour
- `crates/se-media/CLAUDE.md` (formats de fichiers, jobs), `docs/architecture.md` §7 si besoin, `CLAUDE.md` racine (commande `fetch-binaries`).

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
