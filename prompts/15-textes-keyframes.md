# 15 — Textes, graphiques et keyframes ⚠️

> **Phase** : 3 — Montage · **Dépend de** : 13 · **Parallélisable avec** : 14, 17

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §5.3, §8, §10
- `docs/adr/0011-motion-templates.md`, `docs/adr/0012-*` (unités spatiales)
- `CLAUDE.md` du domaine, du moteur, de `@se/ui` et de l'app

## Objectif
Créer et animer des textes, formes et images : styles typographiques complets, transformations manipulables directement dans le viewer, keyframes sur toutes les propriétés animables avec courbes d'easing, et une bibliothèque de presets d'animation (fondu, glissé, pop, machine à écrire…). C'est la base du motion design déclaratif que l'IA utilisera.

## Périmètre
**Inclus**
- Modèle complet des éléments Text/Shape/Image, polices, inspecteur, gizmo de transformation dans le viewer, édition des keyframes, presets d'animation, rendu dans le moteur.

**Exclus**
- Sous-titres → 23 · templates de motion par code → 24 · brand kit → 25.

## Spécifications

### Domaine
- `Text` : contenu (multiligne), police (famille, graisse, italique), taille (pixels de conception), couleur, contour (couleur, épaisseur), ombre (couleur, flou, décalage), boîte de fond (couleur, marge interne, rayon), interlettrage, interligne, alignement, largeur max / retour à la ligne, casse (normal/majuscules), **révélation** (`reveal` 0→1, utilisée par l'effet machine à écrire). Les propriétés pertinentes sont `Animatable`.
- `Shape` : rectangle, rectangle arrondi, ellipse, ligne ; remplissage, contour.
- `Image` : asset image, ajustement.
- Toutes ces propriétés exposées comme `AnimatableProperty` pour `SetKeyframes`.
- **Presets d'animation** définis en données (`crates/se-domain/assets/animation-presets.json`) : entrée (fondu, glissé depuis 4 côtés, pop/scale, flou → net si filtre disponible, machine à écrire), sortie (inverses), emphase (rebond, secousse, pulsation). Opération `ApplyAnimationPreset { item, preset, phase: In | Out | Emphasis, duration }` qui **se développe en keyframes** (éditables ensuite) — logique en Rust pour que l'IA (prompt 19) et l'UI partagent la même implémentation.

### Polices
- Liste des polices système côté cœur (crate `fontdb`) + polices embarquées ; fichiers servis par `/media/fonts/{id}` ; chargement dans le renderer via `FontFace` ; repli documenté si une police manque (avertissement visible).

### UI
- Outils de création : Texte (`T`), Forme, Image (depuis le panneau Médias) — placés sur la piste vidéo la plus haute libre à la tête de lecture, durée par défaut réglable.
- **Inspecteur** (`features/inspector`) : sections par type (Texte, Forme, Transformation, Opacité, Animation), sélection multiple (propriétés communes), `PropertyRow` + `KeyframeButton` sur chaque propriété animable.
- **Gizmo dans le viewer** : sélection par clic, déplacement, mise à l'échelle (Maj = proportionnel), rotation, ancre ; aimantation au centre et aux bords ; édition du texte en double-cliquant (champ superposé).
- **Keyframes** : ajout/suppression à la tête de lecture depuis l'inspecteur, navigation entre keyframes, piste de keyframes dépliable sous l'élément dans la timeline (sélection, déplacement, suppression), éditeur d'easing par keyframe (`EasingCurveEditor`).
- **Presets** : panneau « Animations » (catégories entrée/sortie/emphase, aperçu animé au survol), application à l'élément sélectionné.

### Moteur
- Rendu des textes avec PixiJS (style complet, contour, ombre, boîte de fond, révélation progressive), formes, images ; golden frames pour chaque style.

## Contraintes
- Une modification continue (glisser dans le gizmo ou un `ScrubInput`) = une seule transaction au relâchement.
- L'UI ne développe jamais un preset elle-même : elle appelle l'opération du domaine.

## Critères d'acceptation
- [ ] Créer un texte, le styler, l'animer avec un preset puis ajuster les keyframes générées à la main.
- [ ] Le gizmo déplace/redimensionne/tourne avec aimantation, une étape d'annulation par geste.
- [ ] Le rendu correspond aux golden frames (styles de texte, formes, rotation, opacité animée).

## Tests attendus
- Domaine : développement des presets (snapshots des keyframes générées), validation des propriétés.
- Moteur : golden frames texte/formes/presets à des instants clés.
- UI : inspecteur (modification + commit unique), gizmo (géométrie pure testée), édition des keyframes.

## Documentation à mettre à jour
- `docs/glossary.md`, `docs/architecture.md` §5.3, `CLAUDE.md` du domaine (format des presets), du moteur et de l'app.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
