# 10 — Timeline A : affichage et navigation

> **Phase** : 3 — Montage · **Dépend de** : 08 · **Parallélisable avec** : 09, 17

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §4.2, §4.3, §5.1, §5.6
- `docs/adr/0004-time-in-flicks.md`
- `CLAUDE.md` de `apps/desktop` et `packages/ui`, vecteurs de test `docs/test-vectors/`

## Objectif
Une timeline fluide qui affiche la séquence active : pistes, éléments avec vignettes et formes d'onde, règle en timecode, tête de lecture, zoom, défilement, sélection de plage (points d'entrée/sortie) — cette plage servira plus tard de portée aux prompts IA.

## Périmètre
**Inclus**
- Rendu de la timeline, navigation, gestion des pistes, sélection, plage in/out, marqueurs, registre de raccourcis, utilitaires de temps côté TS.

**Exclus**
- Déplacer/rogner/couper des éléments → 11 · lecture vidéo → 12/13.

## Spécifications

### Utilitaires de temps TS (`src/shared/time/`)
- Type `Flicks` brandé, conversions flicks ↔ secondes ↔ frames ↔ pixels (selon le zoom), formatage/parsing de timecode selon la cadence.
- **Parité avec Rust** : les tests TS rejouent `docs/test-vectors/timecode.json` et `flicks.json`.

### Composant Timeline (`features/timeline`)
- **En-têtes de pistes** : nom (renommage inline), icône du type, boutons muet/solo/verrou/masqué, hauteur redimensionnable, menu contextuel (ajouter au-dessus/en dessous, supprimer, renommer), réordonnancement par glisser.
- **Règle** : graduations adaptatives selon le zoom, libellés en timecode de la séquence, clic/glisser = déplacement de la tête de lecture.
- **Tête de lecture** : ligne + poignée déplaçable ; position stockée dans un store dédié (mises à jour fréquentes sans re-rendu global : abonnement ciblé et mise à jour directe du style).
- **Zoom** : Ctrl/⌘ + molette centrée sur le curseur, curseur de zoom, « ajuster à la fenêtre », raccourcis `+`/`-`.
- **Défilement** : horizontal (molette + Maj, trackpad), vertical ; **virtualisation** : seuls les éléments visibles sont rendus.
- **Éléments** :
  - clip vidéo : bande de vignettes depuis la planche du prompt 08 ;
  - clip audio : forme d'onde dessinée en canvas à partir des pics au niveau adapté au zoom ;
  - texte/graphique/sous-titre : couleur sémantique du type + libellé ;
  - nom, indicateur d'origine IA (petit badge), état sélectionné/survolé, composant `ClipShell` de `@se/ui`.
- **Sélection** : clic, Maj+clic, Ctrl/⌘+clic, rectangle de sélection ; store de sélection partagé (exposé par l'`index.ts` de la feature).
- **Plage in/out** : touches `I`/`O`, glisser sur la règle avec Maj, effacement `Alt+X` ; affichée en surbrillance ; exposée dans un store public (utilisée par la feature IA au prompt 20).
- **Marqueurs** : affichage sur la règle, ajout `M` à la tête de lecture, édition du libellé.
- **Onglets de séquences** si le projet en contient plusieurs (changement de séquence active).

### Raccourcis (`src/shared/shortcuts/`)
- Registre central : actions nommées, raccourcis par défaut (adaptés à l'OS), contextes (timeline, viewer, global), détection de conflits, affichage dans les tooltips via `KeyboardShortcut`.
- Navigation : Début/Fin, ←/→ une image, Maj+←/→ dix images, ↑/↓ point de montage précédent/suivant.

### Commandes du cœur utilisées
`AddTrack`, `RemoveTrack`, `UpdateTrack`, `ReorderTracks`, `AddMarker`, `UpdateMarker`, `RemoveMarker` (via des commandes du protocole à créer si absentes, une transaction par geste).

## Contraintes
- Objectif de performance : 500 éléments, défilement et zoom à 60 i/s sur la machine de dev.
- Aucune logique métier dans les composants : calculs de géométrie dans `services/` (purs, testés).

## Critères d'acceptation
- [ ] La séquence s'affiche avec vignettes et formes d'onde ; zoom et défilement restent fluides avec 500 éléments (projet de test généré).
- [ ] La plage in/out se définit au clavier et à la souris et reste visible.
- [ ] Les opérations de pistes produisent chacune une étape d'annulation.
- [ ] Les tests de parité timecode TS/Rust passent.

## Tests attendus
- Services purs : conversions temps ↔ pixels, choix du niveau de pics, graduations de la règle, virtualisation (éléments visibles).
- Composants : sélection (clic, Maj, rectangle), raccourcis I/O, renommage de piste.
- Script de génération d'un projet de test volumineux (réutilisé au prompt 29).

## Documentation à mettre à jour
- `apps/desktop/CLAUDE.md` (feature timeline, registre de raccourcis), `docs/glossary.md` si besoin.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
