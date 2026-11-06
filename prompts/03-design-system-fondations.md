# 03 — Design system : fondations

> **Phase** : 0 — Fondations · **Dépend de** : 01 · **Parallélisable avec** : 02, 05, 06

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §4.1
- `docs/adr/0009-design-system.md`
- `packages/ui/CLAUDE.md`

## Objectif
Poser les fondations de `@se/ui`, une librairie façon shadcn adaptée au montage vidéo : tokens, thème sombre « moderne épuré, dense », primitives accessibles, et Storybook comme documentation vivante.

## Périmètre
**Inclus**
- Tokens de design, thème, polices embarquées, utilitaires.
- Primitives génériques encapsulant Radix.
- Storybook (+ addon a11y) et tests de composants.

**Exclus**
- Composants spécifiques au montage, shell de l'app, i18n → 04.

## Spécifications

### Direction visuelle
« Moderne épuré, dense » : inspiration Linear/Vercel adaptée à un outil de montage. Surfaces sombres neutres (légère teinte froide), **une** couleur d'accent vive, contrastes doux, bordures fines, ombres quasi absentes, coins 4–8 px, contrôles compacts (24/28/32 px), texte de base 12–13 px.

### Tokens (`packages/ui/src/tokens/`)
- CSS variables en **OKLCH** exposées à Tailwind v4 via `@theme` :
  - surfaces : `bg`, `panel`, `panel-raised`, `overlay`, `input`, `border`, `border-strong`, `text`, `text-muted`, `text-subtle`, `text-inverted` ;
  - accent : `accent`, `accent-hover`, `accent-muted`, `accent-foreground`, `focus-ring` ;
  - sémantiques : `success`, `warning`, `danger`, `info` (+ variantes muted) ;
  - types d'éléments de timeline : `item-video`, `item-audio`, `item-graphic`, `item-caption`, `ai` (+ variantes muted/strong) ;
  - rayons, espacements (grille 4 px), tailles de contrôles, typographie (échelle 11/12/13/14/16/20), poids, interlignes, durées et courbes d'animation, couches z-index (`panel`, `dropdown`, `modal`, `toast`, `tooltip`).
- Accent par défaut : un indigo-violet électrique. Présente 3 variantes d'accent dans une story « Theme » pour que l'utilisateur puisse choisir ; documente le choix final dans le `CLAUDE.md` du package.
- Structure prête pour un futur thème clair (pas implémenté).

### Typographie et icônes
- Inter (variable) et JetBrains Mono embarquées via `@fontsource-variable/inter` et `@fontsource/jetbrains-mono` (aucun CDN).
- Utilitaire `tabular-nums` pour les valeurs numériques et timecodes.
- Icônes : `lucide-react`, encapsulées par un composant `Icon` (tailles normalisées).

### Utilitaires
- `cn()` (clsx + tailwind-merge), `cva` pour les variantes, `Slot` pour `asChild`.

### Primitives (`packages/ui/src/primitives/`)
Button (variantes `primary`, `secondary`, `outline`, `ghost`, `danger` ; tailles `xs`, `sm`, `md` ; état chargement), IconButton, Input, Textarea, Label, Field (label + aide + erreur), Select, Checkbox, Switch, Slider, RadioGroup, Tooltip (avec emplacement de raccourci clavier), Popover, Dialog, AlertDialog, DropdownMenu, ContextMenu, Tabs, ToggleGroup, Separator, ScrollArea, Kbd, Badge, Toast (encapsule `sonner`), Progress, Spinner, Skeleton.

Anatomie d'un composant : `primitives/button/{button.tsx, button.stories.tsx, button.test.tsx, index.ts}` ; export depuis `src/index.ts`. Aucun import Radix en dehors de `packages/ui`.

### Storybook
- Builder Vite, fond sombre par défaut, addon a11y, page « Principles » (règles d'usage), page « Tokens » (palette, typo, espacements générés depuis les tokens).
- Script racine `pnpm storybook`.

### Garde-fous
- Script de vérification (intégré à `pnpm check`) qui interdit les couleurs brutes (`#xxx`, `rgb(`, `oklch(` littéraux) hors du dossier `tokens/`, dans `packages/ui/src` et `apps/desktop/src`.

## Contraintes
- Accessibilité : focus visible partout, navigation clavier complète, contrastes AA sur le texte.
- Composants purement présentationnels : aucune connaissance du domaine ni du protocole.

## Critères d'acceptation
- [ ] `pnpm storybook` affiche toutes les primitives avec leurs variantes et la page Tokens.
- [ ] L'addon a11y ne remonte aucune violation critique.
- [ ] Le garde-fou des couleurs brutes fait échouer `pnpm check` si on en ajoute une.
- [ ] Le choix d'accent est documenté.

## Tests attendus
- Vitest + Testing Library pour les composants à comportement (Select, Dialog, DropdownMenu, Tabs, ToggleGroup, Slider) : clavier, focus, ouverture/fermeture.
- Test du garde-fou (fichier avec couleur brute → échec).

## Documentation à mettre à jour
- `packages/ui/CLAUDE.md` : anatomie, règles, comment ajouter un composant, accent choisi.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
