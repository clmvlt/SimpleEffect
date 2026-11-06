# 27 — Export vidéo (rendu final) ⚠️

> **Phase** : 7 — Export · **Dépend de** : 16 · **Parallélisable avec** : 21 → 26 (à étendre ensuite pour sous-titres et templates)

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §8, §12
- `docs/adr/0002-web-rendering-engine.md`
- `CLAUDE.md` du moteur, de `se-export`, `se-core`, de l'app

## Objectif
Exporter la séquence en fichier vidéo final (MP4 H.264/HEVC/AV1, ou ProRes pour un master), avec le même moteur que la preview (rendu identique), en arrière-plan pendant que l'utilisateur continue à monter.

## Périmètre
**Inclus**
- Dialogue et préréglages d'export, `ExportWorker`, mixage audio, encodage WebCodecs, mux, écriture via le cœur, file d'exports, chemin ffmpeg pour ProRes/DNxHR.

**Exclus**
- Export Premiere → 28.

## Spécifications

### Dialogue (`features/export`)
- Préréglages : YouTube 1080p et 4K (H.264 ou HEVC), Shorts/Reels/TikTok 1080×1920, carré 1080, master ProRes 422 HQ, personnalisé (codec, débit ou qualité, résolution, audio AAC/Opus et débit).
- Plage : séquence entière ou in/out ; destination (dialogue natif) et modèle de nom de fichier ; estimation de taille.
- Avertissements : médias manquants, mezzanines non prêtes, polices manquantes.

### Rendu (`packages/engine/src/export/`)
- `ExportWorker` (Worker dédié) : `evaluate` + `PixiCompositor` sur `OffscreenCanvas`, décodeurs en variante **originale ou mezzanine (jamais proxy)**, boucle image par image déterministe, `VideoEncoder` avec préférence pour l'accélération matérielle (repli logiciel), gestion de la contre-pression de l'encodeur.
- Templates de motion et sous-titres rendus par les mêmes runtimes que la preview.
- **Audio** : `OfflineAudioContext` n'existant pas en Worker, le mixage est rendu par blocs (ex. 30 s) avec `OfflineAudioContext` sur le thread principal, **en réutilisant le même constructeur de graphe que la preview** (parité), puis le PCM est transféré au Worker et encodé avec `AudioEncoder`.
- Mux Mediabunny (MP4/MOV/WebM) avec cible en flux : les blocs sont envoyés au cœur (`/media/renders/{job}`), qui écrit le fichier sur disque.
- **Chemin ffmpeg** (ProRes/DNxHR) : images RGBA envoyées au cœur en flux binaire → ffmpeg (`-f rawvideo -pix_fmt rgba` en entrée standard, `prores_ks`) ; audio écrit en WAV temporaire puis muxé.

### Cœur
- `ExportService` (se-app) : jobs d'export dans la `JobQueue` (progression, ETA, annulation, nettoyage des fichiers partiels), **instantané de la séquence au lancement** (les modifications ultérieures n'affectent pas l'export en cours), file d'exports.
- Commandes `StartExport`, `CancelExport`, `ListExports` ; événements de progression.

### UI
- Panneau des exports (progression, ETA, annuler, ouvrir le dossier une fois terminé), notification système à la fin.

## Contraintes
- Le rendu d'export ne dépend jamais de la fenêtre visible (Worker) et ne bloque pas l'édition.
- Parité : une image exportée doit correspondre à la preview au même instant (à la compression près).

## Critères d'acceptation
- [ ] Exporter une séquence 1080p avec textes animés, fondus et audio : le fichier se lit partout et correspond à la preview.
- [ ] Continuer à monter pendant l'export ne modifie pas le fichier en cours.
- [ ] Annuler supprime le fichier partiel.
- [ ] Le master ProRes est lisible (ffprobe confirme le codec et la durée).

## Tests attendus
- Test déterministe : courte séquence synthétique exportée, frames redécodées et comparées aux golden frames de preview (tolérance de compression).
- Mixage audio par blocs : continuité aux frontières de blocs (pas de clic), durée exacte.
- Progression, annulation, instantané de séquence.

## Documentation à mettre à jour
- `packages/engine/CLAUDE.md` (export), `crates/se-export/CLAUDE.md`, `docs/architecture.md` §8 et §12.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
