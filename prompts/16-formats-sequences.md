# 16 — Formats, séquences multiples, safe areas

> **Phase** : 3 — Montage · **Dépend de** : 15 · **Parallélisable avec** : 17

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §5.3
- `docs/adr/0012-*` (unités spatiales)
- `CLAUDE.md` du domaine, du moteur et de l'app

## Objectif
Passer facilement d'un format à l'autre (paysage, portrait, carré, 4:5), gérer plusieurs séquences par projet (ex. une version 16:9 et sa déclinaison 9:16), choisir comment chaque clip s'adapte au cadre et afficher les zones de sécurité des plateformes.

## Périmètre
**Inclus**
- Changement de format d'une séquence, modes de recadrage par clip, duplication d'une séquence dans un autre format, gestion des séquences, safe areas.

**Exclus**
- Recadrage automatique par suivi du sujet → 26.

## Spécifications

### Recadrage par clip (`Reframe`)
- `Fit` (tout le cadre visible, bandes), `Fill` (remplit, recadre au centre), `Custom` (transform libre), et préparation de `Tracked { keyframes }` (rempli au prompt 26).
- Position de recadrage animable (keyframes) pour `Fill` (choisir quelle partie de l'image garder au fil du temps).
- Inspecteur : section « Cadrage » avec le mode et la position ; dans le viewer, poignée pour déplacer la zone gardée en mode `Fill`.

### Changement de format
- Dialogue « Format de la séquence » (presets + personnalisé, cadence non modifiable si la séquence contient déjà des éléments, sinon modifiable) ; politique par défaut des clips (Fit/Fill/garder).
- Grâce aux unités normalisées (ADR 0012), textes et graphiques restent positionnés proportionnellement ; avertissement listant les éléments qui sortent du cadre après changement.

### Séquences multiples
- Onglets de séquences (renommer, dupliquer, supprimer avec confirmation).
- **« Créer une version… »** : duplique la séquence dans un autre format (ex. 9:16) avec mode de recadrage par défaut `Fill` ; la nouvelle séquence est indépendante (modifications non propagées), ce qui est indiqué à l'utilisateur.

### Safe areas
- Données dans `packages/engine/src/safe-areas/` (JSON) : TikTok, Instagram Reels, YouTube Shorts (zones couvertes par l'interface de l'app), YouTube 16:9 (title/action safe). Indique la source et la date des mesures dans le fichier.
- Superposition dans le viewer (bascule + choix de la plateforme, mémorisé par séquence) ; non rendue à l'export.

## Contraintes
- Le recadrage est évalué dans `evaluate` (moteur) à partir des données du domaine : une seule implémentation.

## Critères d'acceptation
- [ ] Passer une séquence de 16:9 à 9:16 garde les textes à des positions cohérentes ; les clips suivent la politique choisie.
- [ ] « Créer une version 9:16 » produit une nouvelle séquence montable indépendamment.
- [ ] La position de recadrage en mode Fill s'anime par keyframes.
- [ ] Les safe areas TikTok/Reels/Shorts s'affichent et ne sont pas exportées.

## Tests attendus
- Domaine : opérations de changement de format et de duplication (validité, ids), proptests.
- Moteur : golden frames Fit/Fill/Custom et position animée.
- UI : dialogue de format, duplication de séquence.

## Documentation à mettre à jour
- `docs/glossary.md` (Reframe, Safe area), `CLAUDE.md` du moteur et de l'app.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
