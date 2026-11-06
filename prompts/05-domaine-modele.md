# 05 — Domaine : temps et modèle de document

> **Phase** : 1 — Domaine et projets · **Dépend de** : 01 · **Parallélisable avec** : 02, 03, 04

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §5.1 à §5.3, `docs/glossary.md`
- `docs/adr/0004-time-in-flicks.md`, `docs/adr/0010-typed-protocol-codegen.md`
- `crates/se-domain/CLAUDE.md`

## Objectif
Implémenter dans `se-domain` (pur, sans IO) la représentation du temps et le modèle complet du document (projet, médias, séquences, pistes, éléments, propriétés animables), avec validation des invariants et export des types vers TypeScript.

## Périmètre
**Inclus**
- Temps (flicks, cadences, plages, timecodes), identifiants, modèle de document, unités spatiales, validation.
- Export ts-rs vers `@se/protocol`.

**Exclus**
- Opérations et historique → 06 · interpolation des keyframes (vit dans `@se/engine`, prompt 12) · détails complets des textes/formes (prompt 15), sous-titres (23), templates (24).

## Spécifications

### Temps (`time/`)
- `Flicks(i64)` : `PER_SECOND = 705_600_000`, arithmétique vérifiée (`checked_add`…), bornes ±2^53 (`MAX_SAFE`), conversions `from_seconds_f64` (arrondi au plus proche, documenté), `to_seconds_f64`, `from_frames(n, rate)`, `to_frames(rate, RoundingMode)`.
- `FrameRate { numerator: u32, denominator: u32 }` avec constantes (23.976, 24, 25, 29.97, 30, 50, 59.94, 60, 120), `frame_duration()` exacte en flicks, `is_ntsc()`.
- `TimeRange { start, duration }` : `end`, `contains`, `intersects`, `intersection`, `shifted`, invariants (durée ≥ 0).
- `Timecode` : parsing de `51s`, `51.5s`, `00:51`, `1:02.500`, `00:00:51:12` (frames, exige une cadence), `00:00:51;12` (drop-frame noté mais rendu non-drop pour l'instant, documenté) ; formatage selon la cadence ; erreurs explicites (`"'1:75' : les secondes doivent être < 60"`… message en anglais).
- **Vecteurs de test partagés** : crée `docs/test-vectors/timecode.json` et `docs/test-vectors/flicks.json` (entrées → sorties attendues), utilisés par les tests Rust ici et par les tests TS plus tard (prompt 10), pour garantir la parité.

### Identifiants (`ids.rs`)
Newtypes sur UUIDv7 (chaîne) : `ProjectId`, `SequenceId`, `TrackId`, `ItemId`, `AssetId`, `TaskId`, `ChangesetId`, `TemplateId`, `LinkGroupId`, `MarkerId`. Aucune génération ici (port `IdGenerator` dans se-app) : seulement parsing/validation.

### Unités spatiales — décision à consigner
- Positions normalisées dans l'espace de la séquence (`0,0` = coin haut-gauche, `1,1` = coin bas-droit) ; ancre normalisée dans la boîte de l'élément ; échelle relative ; rotation en degrés.
- Tailles (taille de police, épaisseur de contour, rayon…) en **pixels de conception** rapportés à une hauteur de référence de 1080 ; l'UI affiche des pixels du format courant.
- Rédige l'**ADR 0012 « Spatial units »** (contexte multi-format 16:9/9:16, décision, conséquences) et ajoute-la à l'index.

### Modèle de document
- `Project { id, name, schema_version, revision, settings, assets, sequences, templates, brand_kit: Option<…> }` (templates et brand kit : structures minimales, complétées aux prompts 24/25).
- `Asset { id, original_path, relative_hint, kind: Video|Audio|Image, media_info: Option<MediaInfo>, derived: DerivedMedia, decodability, missing: bool }` ; `MediaInfo` (durée, conteneur, flux vidéo : codec, profil, bit depth, chroma, largeur, hauteur, cadence, rotation ; flux audio : codec, canaux, fréquence) ; `DerivedMedia` (mezzanine/proxy : chemin + statut).
- `Sequence { id, name, format: Format, tracks, markers }` ; `Format { width, height, frame_rate, sample_rate }` + `FormatPreset` (16:9 1080p, 16:9 4K, 9:16 1080×1920, 1:1 1080, 4:5 1080×1350, Custom).
- `Track { id, kind: Video|Audio, name, muted, solo, locked, hidden, volume_db, pan, items }` (ordre des pistes vidéo = ordre de superposition).
- `Item { id, revision, origin: Origin, timeline_range, link_group: Option<LinkGroupId>, kind: ItemKind }`.
- `ItemKind` : `MediaClip { asset_id, source_start, speed, transform, opacity, volume_db, fade_in, fade_out, reframe }`, `Text { content, style, transform }` (style minimal : police, taille, couleur), `Shape`, `Image`, `MotionTemplate { template_id, params }`, `Caption` (squelettes documentés `TODO(prompt-NN)`).
- `Transform2D { position, scale, rotation, anchor, opacity }` en `Animatable<…>`.
- `Animatable<T> = Static(T) | Keyframed(Vec<Keyframe<T>>)`, `Keyframe<T> { time (relatif à l'élément), value, easing }`, `Easing = Linear | Hold | CubicBezier { x1, y1, x2, y2 } | Preset(name)`.
- `Origin = User | AiTask(TaskId)` ; `Marker { id, time, label, color }`.
- serde : `rename_all = "camelCase"`, enums taggés `kind`. Tous les types dérivent `ts_rs::TS` et sont inclus dans `pnpm codegen` (si le prompt 02 n'est pas encore fait, prépare l'export et note-le dans PROGRESS.md).

### Validation (`validation/`)
`validate_project(&Project) -> Vec<Violation>` avec `Violation { code, path, message }`. Invariants : pas de chevauchement sur une même piste, temps ≥ 0 et dans les bornes, plage source dans la durée du média (si connue), keyframes triées et dans la durée de l'élément, références existantes, pistes audio sans élément visuel et inversement, vitesse > 0.

## Contraintes
- Aucune dépendance IO/async ; pas d'horloge ; pas de génération aléatoire.
- Un fichier par concept, modules courts, doc comments partout.

## Critères d'acceptation
- [ ] Conversions exactes pour toutes les cadences listées (ex. 1 frame à 30000/1001 = 23 543 520 flicks).
- [ ] Tous les vecteurs de test partagés passent.
- [ ] `validate_project` détecte chaque type de violation (un test par code).
- [ ] Types exportés en TS sans erreur de compilation.
- [ ] ADR 0012 rédigée.

## Tests attendus
- Unitaires : temps, timecodes (table de cas valides/invalides), plages, validation.
- Propriétés (proptest) : aller-retour flicks ↔ frames pour chaque cadence ; propriétés algébriques de `TimeRange` (intersection commutative, contenue dans les deux).
- Snapshot (insta) d'un projet d'exemple sérialisé en JSON (fixe le format du document).

## Documentation à mettre à jour
- `docs/glossary.md` (nouveaux termes), `docs/architecture.md` §5.3 si le modèle diffère, `crates/se-domain/CLAUDE.md`, ADR 0012 + index.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
