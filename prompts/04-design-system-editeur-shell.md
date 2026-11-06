# 04 — Design system : composants montage, shell de l'app, i18n

> **Phase** : 0 — Fondations · **Dépend de** : 02, 03 · **Parallélisable avec** : 05, 06

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §4
- `docs/adr/0009-design-system.md`
- `packages/ui/CLAUDE.md`, stories existantes de `@se/ui`

## Objectif
Ajouter à `@se/ui` les composants propres à un logiciel de montage, puis construire le shell de l'application (écran d'accueil vide, mise en page de l'éditeur en panneaux redimensionnables, barre d'état) entièrement en français via i18n.

## Périmètre
**Inclus**
- Composants « editor » de `@se/ui` (visuels et interactions locales, sans domaine).
- Routage, mise en page, barre d'état, i18n dans le renderer.

**Exclus**
- Toute donnée réelle (projets, médias, timeline) → prompts 07 et suivants.

## Spécifications

### Composants `packages/ui/src/editor/`
- **Panel** (en-tête avec titre, actions, onglets optionnels ; corps scrollable) et **Dock** basé sur `react-resizable-panels` (tailles persistées par identifiant de layout via un adaptateur de stockage injecté).
- **Toolbar**, **ToolbarGroup**, **ToolbarSeparator**.
- **PropertyGroup** (repliable) et **PropertyRow** (libellé, contrôle, emplacement pour bouton keyframe, état « modifié »).
- **ScrubInput** : valeur numérique modifiable en glissant horizontalement (curseur `ew-resize`), Maj = rapide, Alt = fin, double-clic = saisie clavier, flèches haut/bas, min/max/pas, unité affichée, formatage injecté. Émet `onChange` pendant le glissement et **`onCommit` une seule fois** à la fin (indispensable pour qu'un geste = une étape d'annulation).
- **TimecodeInput / TimecodeDisplay** : agnostiques du domaine — reçoivent `format(value)` et `parse(text)` en props ; police mono, chiffres tabulaires ; saisie avec validation.
- **ColorField** : pastille + popover (sélecteur, champs hex et OKLCH, pipette via l'API `EyeDropper` si disponible, couleurs récentes injectables).
- **KeyframeButton** : losange avec états `none`, `onKeyframe`, `betweenKeyframes`, navigation précédent/suivant.
- **EasingCurveEditor** : éditeur cubic-bezier avec poignées déplaçables, préréglages (linear, ease-in, ease-out, ease-in-out, back, hold), aperçu animé.
- **TransportBar** : lecture/pause, stop, image précédente/suivante, aller au début/à la fin, boucle, emplacement timecode.
- **Meter** : VU-mètre stéréo vertical/horizontal, échelle dB, maintien de crête ; valeurs en props.
- **AspectRatioPicker** : 16:9, 9:16, 1:1, 4:5, personnalisé, avec vignettes proportionnelles.
- **StatusDot**, **EmptyState**, **KeyboardShortcut** (affichage de raccourcis selon l'OS : ⌘ ou Ctrl).
- **AiTaskCard** (visuel seulement) : états `queued`, `running`, `completed`, `failed`, `cancelled`, emplacement pour journal en streaming, coût, actions.
- **PromptComposer** (visuel seulement) : zone de texte auto-extensible, puces (chips) de portée supprimables passées en props, envoi Ctrl/⌘+Entrée, emplacements pour sélecteurs de modèle et d'effort.
- Chaque composant : stories (états et variantes), tests des interactions clés.

### i18n (renderer)
- `i18next` + `react-i18next`, langue par défaut `fr`, fichier `src/i18n/locales/fr.json` organisé par feature (`home.*`, `editor.*`, `statusBar.*`…), clés typées (augmentation de types i18next).
- Règle ESLint interdisant les chaînes littérales dans le JSX de `apps/desktop/src` (ex. `eslint-plugin-i18next` `no-literal-string`, avec exceptions documentées).
- `@se/ui` ne contient aucun texte métier : libellés passés en props (les quelques libellés génériques d'accessibilité sont des props avec valeur par défaut anglaise surchargée par l'app).

### Shell de l'application
- Routage (React Router en mode bibliothèque) : `/` accueil, `/project/:projectId` éditeur.
- **Accueil** : état vide « Aucun projet » + bouton « Nouveau projet » (inactif jusqu'au 07).
- **Éditeur** (mise en page Dock) : à gauche « Médias », au centre « Viewer », à droite « Inspecteur » et « IA » (onglets), en bas « Timeline ». Chaque panneau affiche un EmptyState explicite. Tailles persistées (localStorage via l'adaptateur).
- **Barre d'état** : état de la connexion au cœur (via `useConnectionStatus`), emplacement pour l'état de l'IA (gris « non configurée » pour l'instant), version.
- Remplace l'écran de test du prompt 01 par ce shell.

## Contraintes
- Densité élevée, tout au clavier, focus visible.
- Les composants editor restent agnostiques : aucun import de `@se/protocol` ou `@se/engine` dans `@se/ui`.

## Critères d'acceptation
- [ ] Toutes les stories existent et passent l'addon a11y.
- [ ] `ScrubInput` : un glissement produit plusieurs `onChange` et exactement un `onCommit` (testé).
- [ ] Le shell s'affiche, les panneaux se redimensionnent et retrouvent leur taille au redémarrage.
- [ ] Aucune chaîne en dur dans le JSX de l'app (lint vert).

## Tests attendus
- Vitest + Testing Library : ScrubInput (glisser, modificateurs, saisie, commit unique), TimecodeInput (parse invalide), EasingCurveEditor (déplacement de poignée → valeurs), KeyframeButton (états), PromptComposer (envoi raccourci, suppression de puce).
- Test de rendu du shell (routes, panneaux présents).

## Documentation à mettre à jour
- `packages/ui/CLAUDE.md` (inventaire editor), `apps/desktop/CLAUDE.md` (routage, i18n, comment ajouter une clé).

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
