# 06 — Domaine : opérations, transactions, historique, changesets ⚠️

> **Phase** : 1 — Domaine et projets · **Dépend de** : 05 · **Parallélisable avec** : 03, 04

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §5.4, `docs/glossary.md`
- `docs/adr/0005-single-writer-operations-changesets.md`
- `crates/se-domain/CLAUDE.md`

## Objectif
Toutes les modifications du document deviennent des opérations pures, inversibles et validées, regroupées en transactions atomiques, avec historique d'annulation de l'utilisateur et changesets annulables par tâche IA (y compris après des modifications intercalées).

## Périmètre
**Inclus**
- `Operation`, `apply`, inverses, transactions, révisions, historique utilisateur, changesets, plan de revert avec conflits.

**Exclus**
- L'acteur tokio et la persistance → 07 · transitions et automation audio détaillée → 14 · presets d'animation → 15.

## Spécifications

### Principes
- `apply(&Project, &Operation) -> Result<Applied, DomainError>` est **pur et déterministe** : les identifiants nécessaires (ex. le nouvel élément créé par un split) sont **fournis dans l'opération** par l'appelant.
- `Applied { project, inverse: Vec<Operation>, touched: TouchedEntities }`.
- Après chaque opération, le projet doit satisfaire `validate_project` (sinon erreur et aucun changement).

### Opérations (minimum)
- Éléments : `AddItem`, `RemoveItem`, `MoveItem { item, track, start }` (refuse les chevauchements), `TrimItem { item, edge: Start|End, to }` (ajuste `source_start` des médias, respecte les bornes source), `SplitItem { item, at, new_item_id }`, `UpdateItem { item, patch: ItemPatch }` où `ItemPatch` est un enum de patchs **typés** par type d'élément (champs `Option`), `SetKeyframes { item, property: AnimatableProperty, keyframes }`, `LinkItems`, `UnlinkItems`.
- Temps : `RippleDeleteRange { sequence, range, tracks: Option<Vec<TrackId>> }` (supprime/raccourcit ce qui est dans la plage et décale la suite vers la gauche sur les pistes concernées), `InsertGap { sequence, at, duration, tracks }` (découpe les éléments traversant `at`, ids fournis, et décale à droite).
- Pistes : `AddTrack`, `RemoveTrack`, `UpdateTrack` (patch), `ReorderTracks`.
- Séquences : `AddSequence`, `RemoveSequence`, `UpdateSequence` (nom, format), `AddMarker`, `RemoveMarker`, `UpdateMarker`.
- Médias : `AddAsset`, `UpdateAsset`, `RemoveAsset` (refuse si utilisé).
- Projet : `RenameProject`.

### Transactions et révisions
- `Transaction { id, actor: Actor, label, ops }`, `Actor = User | AiTask(TaskId)`.
- `commit(&Project, &Transaction) -> Result<Committed { project, inverse: Transaction, touched }, DomainError>` : tout ou rien.
- Chaque élément touché voit sa `revision` incrémentée ; la révision du projet augmente de 1 par transaction.
- Option `expected_revision` par opération sur un élément → `DomainError::RevisionConflict { item, expected, actual }`.

### Historique utilisateur
- `History` : piles undo/redo des transactions de l'acteur `User` uniquement (profondeur max configurable), libellés pour l'UI, `undo(&Project) -> Result<(Project, …)>`, `redo`. Une nouvelle transaction utilisateur vide la pile redo.

### Changesets
- `Changeset { id, task_id, entries: Vec<ChangesetEntry { transaction, inverse, touched_revisions }> }`.
- `plan_revert(&Project, &Changeset) -> RevertPlan { ops, conflicts: Vec<Conflict> }` : pour chaque élément touché par la tâche, si sa révision actuelle ≠ la dernière révision produite par la tâche, c'est un **conflit** (modifié depuis par quelqu'un d'autre) → ses opérations inverses sont ignorées et le conflit est rapporté ; les autres inverses sont appliquées dans l'ordre inverse.
- Le revert est ensuite commité comme une transaction `User` (donc annulable).

### Erreurs
`DomainError` (thiserror) avec variantes précises et messages actionnables en anglais (ex. `"item 0193… would overlap item 0194… on track V1 between 00:00:51.000 and 00:00:52.500"`), car ils seront relayés aux agents IA.

## Contraintes
- Pur : pas d'IO, pas d'horloge, pas d'aléatoire.
- Un fichier par opération (ou par petit groupe cohérent) ; `apply` dispatch vers des fonctions courtes.

## Critères d'acceptation
- [ ] Toutes les opérations listées sont implémentées avec leur inverse.
- [ ] Aucune opération ne peut produire un projet invalide.
- [ ] Revert d'un changeset sans modification intercalée → projet identique à l'état d'avant la tâche (hors compteurs de révision).
- [ ] Revert avec un élément modifié entre-temps → conflit rapporté, le reste est annulé.

## Tests attendus
- Unitaires : cas limites de chaque opération (bords de plage, éléments liés, pistes verrouillées si règle définie, bornes source).
- Propriétés (proptest, ≥ 1000 cas) : séquences aléatoires d'opérations valides → invariants toujours vrais ; `apply(op)` puis `apply(inverse)` = identité ; undo/redo cohérents ; revert de changeset sans interférence = état initial.
- Snapshot de quelques transactions sérialisées (format stable pour le journal du prompt 07).

## Documentation à mettre à jour
- `docs/architecture.md` §5.4 (liste réelle des opérations), `docs/glossary.md`, `crates/se-domain/CLAUDE.md` (recette « ajouter une opération »).

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
