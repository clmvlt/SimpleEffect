# 14 — Édition audio, transitions, blancs

> **Phase** : 3 — Montage · **Dépend de** : 13 · **Parallélisable avec** : 15, 17

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §5.3, §5.4, §8
- `crates/se-domain/CLAUDE.md`, `packages/engine/CLAUDE.md`

## Objectif
Régler finement le son (gain, courbe de volume, fondus, panoramique), faire des fondus enchaînés audio et vidéo entre clips adjacents, et visualiser silences et blancs pour les ajuster à la main après une coupe (y compris une coupe faite par l'IA).

## Périmètre
**Inclus**
- Extensions du domaine (automation de volume, fondus, transitions), UI sur la timeline et dans l'inspecteur audio, rendu dans le moteur (audio et fondu vidéo), visualisation des silences.

**Exclus**
- Suppression automatique des silences et montage par le texte → 22.

## Spécifications

### Domaine
- Volume de clip en dB `Animatable<f32>` (keyframes = courbe de volume), fondus d'entrée/sortie (durée + forme : linéaire, puissance constante, logarithmique), volume et pan de piste.
- **Transitions** entre deux éléments adjacents d'une même piste : `Transition { id, kind: CrossDissolve | AudioCrossfade, left_item, right_item, duration, alignment: Center | Start | End }`, utilisant les « poignées » (média disponible au-delà des bords) ; validation : poignées suffisantes, éléments adjacents.
- Opérations : `AddTransition`, `UpdateTransition`, `RemoveTransition` avec inverses et tests de propriétés ; mise à jour de `docs/glossary.md`.

### UI
- Sur les clips audio : ligne de volume (« rubber band ») affichée, glisser la ligne = gain global, Ctrl/⌘+clic = ajouter une keyframe, glisser une keyframe, suppression.
- Poignées de fondu aux coins des clips (audio et vidéo pour l'opacité).
- Création d'une transition : glisser depuis le point de coupe, ou raccourci (`Ctrl/⌘+T`) au point de coupe sous la tête de lecture ; durée par défaut réglable ; affichage dédié sur la timeline.
- En-têtes de pistes audio : volume (`ScrubInput` en dB) et pan.
- Inspecteur (section Audio d'un clip) : gain, fondus, forme des fondus.
- **Silences** : plages de silence (prompt 09) affichées en hachures sur les clips audio (bascule d'affichage).
- **Blancs** : visualisation renforcée des blancs entre éléments, avec durée affichée au survol ; la poignée « allonger/raccourcir le blanc » du prompt 11 reste l'outil principal.

### Moteur
- Automation de volume convertie en rampes Web Audio (`linearRampToValueAtTime` / courbes) à partir des keyframes en dB ; fondus et fondus enchaînés audio.
- Fondu enchaîné vidéo dans le compositeur (deux sources pendant la transition).
- Golden frames pour le fondu enchaîné ; tests unitaires de la conversion dB → gain → rampes.

## Contraintes
- La logique de conversion et d'évaluation est pure et testée ; le graphe Web Audio n'est que l'exécution.

## Critères d'acceptation
- [ ] On dessine une courbe de volume sur un clip et on l'entend à la lecture.
- [ ] Fondus et fondus enchaînés audio/vidéo fonctionnent et s'annulent en une étape chacun.
- [ ] Les silences sont visibles et l'on peut allonger un blanc après une coupe.

## Tests attendus
- Domaine : opérations de transition (proptests d'inverse, validation des poignées).
- Moteur : conversion de l'automation, fondu enchaîné (golden frame).
- UI : ajout/déplacement de keyframe de volume, création de transition.

## Documentation à mettre à jour
- `docs/glossary.md`, `docs/architecture.md` §5.3 (transitions), `CLAUDE.md` du domaine et du moteur.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
