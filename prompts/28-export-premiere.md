# 28 — Export Premiere Pro ⚠️

> **Phase** : 7 — Export · **Dépend de** : 23, 27 · **Parallélisable avec** : —

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §12
- `docs/adr/0008-premiere-export.md`, `docs/adr/0004-time-in-flicks.md`
- `CLAUDE.md` de `se-export`, du moteur, de l'app

## Objectif
Exporter un projet vers Adobe Premiere Pro sous forme de dossier prêt à importer : coupes, pistes et niveaux audio en natif, textes et motion design pré-rendus avec transparence, sous-titres éditables.

## Périmètre
**Inclus**
- Générateur FCP7 XML, générateur OTIO, pré-rendu des overlays en ProRes 4444 alpha, SRT, dossier d'export, checklist d'import manuelle.

**Exclus**
- Textes éditables dans Premiere (MOGRT) → `docs/backlog.md`.

## Spécifications

### Dossier `<nom>_premiere/`
`sequence.xml` (FCP7), `sequence.otio`, `overlays/*.mov`, `captions.srt`, `README.txt` (en français : étapes d'import dans Premiere, import du SRT en légendes, limites connues).

### FCP7 XML (`se-export`, fonctions pures)
- xmeml version 4 ; séquence avec cadence (`timebase` + `ntsc`), timecode de départ, format (résolution, ratio de pixels carré).
- Pistes vidéo et audio dans l'ordre ; `clipitem`s avec référence au **fichier original** (`pathurl` en URI `file://` correctement encodée : espaces, accents, lettres de lecteur Windows, chemins macOS/Linux), `in`/`out` source et `start`/`end` séquence en images, éléments liés (`link`).
- Niveaux audio (filtre « Audio Levels » avec keyframes) et fondus, opacité et Basic Motion (échelle, position, rotation ; keyframes si présents), fondus enchaînés vidéo et audio (transitions), marqueurs.
- Les overlays pré-rendus sont placés sur des pistes vidéo supérieures aux bons instants.
- Conversion flicks → images à la cadence de la séquence avec règle d'arrondi documentée.

### Overlays pré-rendus
- Regroupement des éléments graphiques (textes, formes, images, templates ; sous-titres optionnellement incrustés si l'utilisateur le demande) en **segments contigus par piste**.
- Rendu de chaque segment avec fond transparent par l'`ExportWorker` (prompt 27), images RGBA → cœur → ffmpeg `prores_ks` profil 4444 (`yuva444p10le`) → `overlays/<piste>_<début>.mov`.

### OTIO
- Timeline OTIO (JSON) équivalente : pistes, clips avec références externes (originaux et overlays), temps rationnels à la cadence de la séquence, marqueurs.

### SRT
- À partir des sous-titres du prompt 23.

### UI
- Option « Exporter vers Premiere Pro » dans le dialogue d'export : destination, inclure OTIO (coché), incruster les sous-titres (décoché), progression (le pré-rendu est un job long).

### Checklist manuelle
- `docs/premiere-import-checklist.md` : projet de test (multi-pistes, 29,97 i/s, éléments liés, keyframes de volume, fondus, textes animés, sous-titres) et liste de vérifications à faire dans Premiere. **Demande à l'utilisateur de faire l'import et de te rapporter le résultat** ; corrige ce qui ne va pas.

## Contraintes
- Les générateurs sont des fonctions pures du modèle de domaine vers du texte ; tout est testé par snapshots.

## Critères d'acceptation
- [ ] Le dossier s'importe dans Premiere : séquence, pistes, coupes, niveaux, fondus et overlays aux bons endroits (validé par l'utilisateur avec la checklist).
- [ ] Les sous-titres s'importent comme légendes éditables.
- [ ] Les chemins avec espaces/accents fonctionnent sur Windows et macOS.
- [ ] L'OTIO s'ouvre dans DaVinci Resolve ou Premiere 26+ (vérification manuelle si disponible).

## Tests attendus
- Snapshots insta : XML, OTIO, SRT sur des projets de test (dont 29,97 et 23,976 i/s, éléments liés, keyframes, transitions).
- Encodage des `pathurl` (table de cas par OS), conversion flicks → images, regroupement des overlays en segments.

## Documentation à mettre à jour
- `crates/se-export/CLAUDE.md`, `docs/premiere-import-checklist.md`, `docs/architecture.md` §12.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
