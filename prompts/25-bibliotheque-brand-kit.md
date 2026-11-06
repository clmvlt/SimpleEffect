# 25 — Bibliothèque de templates et brand kit

> **Phase** : 6 — Motion design avancé · **Dépend de** : 23, 24 · **Parallélisable avec** : 26

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §10, §11
- `docs/adr/0011-motion-templates.md`
- `docs/ai-tools.md`, `CLAUDE.md` de `se-store`, `se-app`, de l'app

## Objectif
Réutiliser son travail d'un projet à l'autre : une bibliothèque globale de templates de motion (titres, lower thirds, CTA, styles de sous-titres) et des brand kits (polices, couleurs, logo, styles par défaut) que l'utilisateur et les agents IA appliquent automatiquement.

## Périmètre
**Inclus**
- Bibliothèque globale (stockage, import/export, versions), navigateur de templates, brand kits, application des valeurs par défaut, outils MCP.

**Exclus**
- Partage en ligne (aucun réseau).

## Spécifications

### Bibliothèque globale
- Stockage `<app data>/templates/<id>/` : `template.json` (métadonnées, schéma, version, catégorie, tags), `source.ts`, `thumbnail.png`.
- Actions : enregistrer un template du projet dans la bibliothèque, insérer un template de la bibliothèque dans le projet (copie versionnée), notifier quand la bibliothèque contient une version plus récente d'un template utilisé (mise à jour proposée, jamais automatique).
- Import/export d'un fichier `.setemplate` (zip) ; validation complète à l'import (même validation qu'au prompt 24).
- Les styles de sous-titres (prompt 23) peuvent aussi être enregistrés dans la bibliothèque.

### Navigateur de templates (`features/templates`)
- Grille avec aperçu animé au survol (quelques images rendues par le runtime), recherche, catégories (titres, lower thirds, CTA, transitions, sous-titres), filtres projet/bibliothèque.
- Glisser un template sur la timeline = instanciation à l'endroit déposé.

### Brand kits
- Plusieurs kits dans `<app data>/brand-kits/` ; un kit sélectionné par projet (`Project.brand_kit` référence + copie des valeurs pour la portabilité du projet).
- Champs : nom, polices (titres, texte), palette nommée (primaire, secondaire, accent, texte, fond), logo(s) (assets), style de texte par défaut, style de sous-titres par défaut, templates d'intro/outro favoris.
- Éditeur de brand kit (réglages), aperçu.
- Application : les nouveaux textes et sous-titres prennent les valeurs du kit ; les paramètres `color`/`font` des templates proposent les valeurs du kit en raccourcis.

### MCP
- `get_brand_kit` réel, `list_templates { source: project | library, category? }` réel, `insert_library_template { template_id, range, params }`.
- Instructions : utiliser le brand kit de façon cohérente (polices, couleurs) sauf demande contraire.
- Mise à jour de `docs/ai-tools.md`.

## Contraintes
- Un projet doit rester ouvrable sur une autre machine sans la bibliothèque (copies dans le projet).

## Critères d'acceptation
- [ ] Enregistrer un template dans la bibliothèque, l'utiliser dans un autre projet, l'exporter puis le réimporter.
- [ ] Avec un brand kit actif, un agent qui crée un titre utilise les polices et couleurs du kit.
- [ ] Une mise à jour de template dans la bibliothèque est proposée (et non imposée) aux projets qui l'utilisent.

## Tests attendus
- Entrées/sorties de la bibliothèque, import/export `.setemplate` (aller-retour, fichier invalide), résolution des valeurs par défaut du brand kit, comparaison de versions.

## Documentation à mettre à jour
- `docs/ai-tools.md`, `docs/architecture.md` §10–11, `docs/glossary.md`, `CLAUDE.md` des modules touchés.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
