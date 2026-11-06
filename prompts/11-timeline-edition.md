# 11 — Timeline B : édition

> **Phase** : 3 — Montage · **Dépend de** : 10 · **Parallélisable avec** : 17

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §5.4, §5.6
- `docs/adr/0005-single-writer-operations-changesets.md`
- `crates/se-domain/CLAUDE.md`, `apps/desktop/CLAUDE.md`

## Objectif
Monter à la souris et au clavier : glisser des médias depuis le panneau Médias vers la timeline, déplacer, rogner, couper, supprimer (avec ou sans ripple), dupliquer, copier/coller, aimanter, gérer les blancs — chaque geste formant exactement une étape d'annulation.

## Périmètre
**Inclus**
- Toutes les interactions d'édition de base sur les éléments et les blancs, état éphémère pendant les gestes, menus contextuels.

**Exclus**
- Volume, fondus, transitions → 14 · textes/formes → 15.

## Spécifications

### Glisser depuis le panneau Médias
- Aperçu fantôme pendant le survol de la timeline, placement sur la piste survolée (création d'une piste si on dépose sous la dernière).
- Un média vidéo avec audio crée deux éléments liés (`link_group`) : vidéo sur une piste V, audio sur une piste A alignée.
- Dépôt en mode insertion (Ctrl/⌘ : décale la suite) ou écrasement (par défaut : refuse ou rogne selon la règle choisie et documentée).

### Gestes d'édition
- **Déplacer** (même piste ou autre piste compatible), les éléments liés suivent ; Alt+glisser = dupliquer.
- **Rogner** par les bords (respect des bornes de la source) ; avec modificateur = rognage ripple (décale la suite).
- **Couper** à la tête de lecture : `S` (et Ctrl/⌘+K) sur les éléments sélectionnés ou sous la tête de lecture.
- **Supprimer** : `Suppr` (laisse un blanc) ; `Maj+Suppr` = suppression ripple.
- **Copier/couper/coller** (presse-papiers interne, collage à la tête de lecture sur la piste active).
- **Décaler** d'une image : `,` / `.` ; de dix images avec Maj.
- **Lier/délier** des éléments (menu contextuel).
- **Blancs** : clic dans un blanc = sélection du blanc ; `Suppr` sur un blanc = fermer le blanc (ripple) ; **poignée de blanc** que l'on glisse pour **allonger ou raccourcir le blanc** (insère ou retire du temps et décale la suite) — c'est la fonction « rallonger un blanc » demandée.
- **Aimantation** (`N` pour basculer) : tête de lecture, bords d'éléments, marqueurs, in/out ; indicateur visuel du point d'aimantation.

### Architecture des gestes
- Pendant un geste : état **éphémère** local à la feature (positions prévisualisées), aucune commande envoyée.
- Au relâchement : **une** commande (une transaction) ; l'état éphémère est effacé à l'arrivée du patch correspondant (ou à l'erreur, avec toast traduit).
- Calculs purs dans `services/` : aimantation, placement au dépôt, collisions, résolution des éléments liés.
- Commandes du protocole à ajouter si absentes (ex. `ApplyEdit` générique portant une liste d'opérations du domaine, ou commandes par intention — choisis et documente, en gardant l'acteur `User`).

### Menu contextuel des éléments
Couper ici, supprimer, supprimer avec ripple, dupliquer, délier/lier, propriétés (ouvre l'inspecteur, prompt 15), afficher dans le panneau Médias.

## Contraintes
- Les pistes verrouillées ne sont jamais modifiées.
- Toutes les règles de validation restent dans le domaine Rust : le renderer prévisualise mais le cœur décide.

## Critères d'acceptation
- [ ] Chaque geste (déplacer, rogner, couper, supprimer, coller, allonger un blanc…) = une seule étape d'annulation.
- [ ] Les éléments vidéo/audio liés bougent ensemble et se délient à la demande.
- [ ] L'aimantation fonctionne sur tous les points listés.
- [ ] Allonger un blanc décale correctement tout ce qui suit sur les pistes concernées.

## Tests attendus
- Services : aimantation, placement au dépôt, collisions, éléments liés.
- Interactions (Testing Library + user-event pointeur) : déplacer, rogner, couper, suppression ripple, poignée de blanc.
- Intégration cœur : chaque commande produit la transaction attendue (tests Rust côté use case).

## Documentation à mettre à jour
- `apps/desktop/CLAUDE.md` (architecture des gestes), `docs/architecture.md` si une commande générique est introduite.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
