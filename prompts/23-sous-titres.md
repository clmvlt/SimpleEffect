# 23 — Sous-titres automatiques animés

> **Phase** : 5 — Transcription et audio intelligent · **Dépend de** : 15, 22 · **Parallélisable avec** : 24

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §5.3, §8, §9.4
- `docs/ai-tools.md`, `CLAUDE.md` du domaine, du moteur, de l'app

## Objectif
Générer des sous-titres depuis la transcription, avec des styles animés façon TikTok/Reels (mot surligné au fil de la parole, pop par mot…), modifiables (texte, timing, style) et exportables en SRT.

## Périmètre
**Inclus**
- Modèle `Caption` et `CaptionStyle`, génération et segmentation, éditeur de sous-titres, rendu animé, styles prédéfinis, import/export SRT, outils MCP.

**Exclus**
- Brand kit (style par défaut) → 25 · export Premiere → 28 (utilisera le SRT).

## Spécifications

### Domaine
- `ItemKind::Caption { words: Vec<CaptionWord { text, start, end }>, style_id }` (temps relatifs à l'élément) ; `CaptionStyle` dans le projet : police, taille, couleurs (texte, mot actif, contour, fond), position (normalisée, défaut tiers inférieur), boîte de fond, casse, animation (`None`, `WordHighlight`, `PopPerWord`, `BoxPerLine`, `RevealPerWord`), max caractères par ligne, max lignes.
- Opérations : création en lot, édition du texte, fusion, scission, décalage, changement de style ; tests de propriétés.

### Génération
- Depuis la transcription de la séquence (prompt 21) sur une plage → segmentation : max caractères/ligne, max lignes, durée min/max, coupure sur ponctuation et pauses ; création sur une piste dédiée « Sous-titres » (créée si absente).
- Re-génération possible d'une plage (remplace les sous-titres de la plage, en une transaction).

### Éditeur (`features/captions`)
- Liste des sous-titres (timecode, texte éditable inline), navigation synchronisée avec la lecture, fusion/scission, décalage global.
- Édition du texte : si le nombre de mots change, les timings des mots sont redistribués proportionnellement dans la durée (règle documentée et testée).
- Panneau de style : styles prédéfinis (données dans `packages/engine/src/caption-styles/` : « TikTok gras », « Minimal », « Karaoké », « Boîte »), personnalisation, application à une sélection ou à toute la piste.

### Moteur
- Rendu de chaque animation (mot actif surligné au bon instant, pop par mot, boîte par ligne) ; golden frames à des instants clés.

### SRT
- Export (utilisé par l'export Premiere) et import (création de sous-titres sans timings de mots : mots répartis uniformément) ; tests aller-retour.

### MCP
- `create_captions { range, style? }`, `update_caption_style { style_id | items, patch }`, `edit_caption_text { item_id, text }` ; instructions `07-captions.md` (lisibilité, longueur des lignes, éviter les safe areas des plateformes).
- Mise à jour de `docs/ai-tools.md`.

## Contraintes
- Le rendu des sous-titres passe par le même moteur que le reste (WYSIWYG à l'export).

## Critères d'acceptation
- [ ] Générer les sous-titres d'une vidéo de 2 minutes, appliquer « Karaoké », corriger une faute : le mot surligné suit toujours la parole.
- [ ] Export SRT importable dans un lecteur standard ; import d'un SRT crée des sous-titres éditables.
- [ ] Un agent peut créer et styliser des sous-titres sur sa portée.

## Tests attendus
- Segmentation (table de cas), redistribution des timings, SRT aller-retour, opérations du domaine (proptests), golden frames des styles.

## Documentation à mettre à jour
- `docs/glossary.md`, `docs/ai-tools.md`, `CLAUDE.md` du domaine, du moteur, de l'app.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
