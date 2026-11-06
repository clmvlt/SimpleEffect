# 09 — Médias 2 : codecs pro, mezzanines, proxies, analyses

> **Phase** : 2 — Médias · **Dépend de** : 08 · **Parallélisable avec** : 10, 17

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §7
- `docs/adr/0007-media-pipeline-sidecars-mezzanine.md`
- `crates/se-media/CLAUDE.md`, `prompts/PROGRESS.md`

## Objectif
Les fichiers de caméras pro (ProRes, DNxHR, 10 bits 4:2:2…) deviennent montables automatiquement grâce aux mezzanines et proxies, et chaque média reçoit les analyses dont l'IA aura besoin : silences, loudness, changements de plan, extraction d'images.

## Périmètre
**Inclus**
- Politique et jobs mezzanine/proxy, réglages associés, analyses audio et vidéo, service d'extraction d'images, affichage dans le panneau Médias.

**Exclus**
- Utilisation des proxies à la lecture → 12/13 · coupe automatique des silences → 22 · transcription → 21.

## Spécifications

### Politique de mezzanine
- Décision pure et testée (`MezzaninePolicy`) à partir de `MediaInfo` + décodabilité rapportée par le renderer : mezzanine si le codec n'est pas décodable, ou si bit depth/chroma ne le sont pas.
- Cible par défaut : HEVC Main10 si la machine le décode (préserve le 10 bits), sinon H.264 High 4:2:0 8 bits ; qualité élevée (CRF réglable, défaut ≈ 14) ; mêmes résolution et cadence ; GOP court pour un seek rapide ; audio dans un codec décodable par WebCodecs (AAC 320k ou FLAC, selon la vérification de capacité).
- Encodage matériel ffmpeg si disponible (NVENC, VideoToolbox, VAAPI) avec repli logiciel ; détection au démarrage, résultat mis en cache.

### Proxies
- H.264 basse résolution (540p ou 720p, réglable), `-tune fastdecode`, GOP très court (ex. 15) pour le scrub.
- Génération auto pour les médias > 1080p ou non décodables (réglable), et action manuelle « Générer un proxy » dans le panneau Médias.

### Réglages (`features/settings`, section Médias)
Qualité mezzanine, résolution des proxies, génération automatique, emplacement et taille du cache, bouton « Nettoyer le cache du projet ».

### Analyses
- `AudioAnalysis` : PCM → détection de silences (seuil dBFS, durée minimale ; liste de plages), loudness intégrée LUFS et courbe court terme (crate `ebur128`), crêtes. Stockage dans `project.db` (`analyses`) + résumé dans l'asset.
- `SceneDetection` : ffmpeg `select='gt(scene,SEUIL)',showinfo` → instants de changement de plan.
- `FrameExtraction` (service, pas un job) : `extract_frame(asset, time, max_width) -> chemin JPEG` mis en cache ; utilisé plus tard par les outils MCP.
- Priorités : miniatures/peaks > proxy > analyses > mezzanine des médias non utilisés.

### UI
- Panneau Médias : statut de la mezzanine et du proxy (en attente, en cours %, prêt, échec + raison), silences et loudness dans le panneau d'infos, nombre de changements de plan.
- Un média non décodable sans mezzanine prête est marqué « en préparation » et ne peut pas encore être lu.

## Contraintes
- Les jobs lourds ne doivent jamais bloquer l'UI ni la boucle tokio (processus séparés, `spawn_blocking` pour l'analyse PCM).
- Les mezzanines/proxies vont dans `cache/` et sont référencés dans `DerivedMedia`.

## Critères d'acceptation
- [ ] Importer un ProRes (généré par ffmpeg pour le test) → mezzanine + proxy produits automatiquement, média marqué lisible.
- [ ] Un H.264 1080p décodable ne génère pas de mezzanine.
- [ ] Les silences et la loudness s'affichent pour un fichier avec voix.
- [ ] Le nettoyage du cache supprime les dérivés et ils sont régénérés au besoin.

## Tests attendus
- Table de décision `MezzaninePolicy`.
- Détection de silences et LUFS sur des signaux synthétiques (silence/sinus, valeurs attendues).
- Parsing de la sortie `showinfo` (fixture).
- Intégration (si ffmpeg présent) : génération d'un ProRes court → mezzanine → probe du résultat.

## Documentation à mettre à jour
- `crates/se-media/CLAUDE.md`, `docs/architecture.md` §7 (politique réelle), `docs/glossary.md` si besoin.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
