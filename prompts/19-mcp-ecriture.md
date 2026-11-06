# 19 — Serveur MCP : outils d'écriture, portée, changesets ⚠️

> **Phase** : 4 — IA · **Dépend de** : 18 · **Parallélisable avec** : 21

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §5.4, §9.3, §9.4
- `docs/adr/0003-*`, `docs/adr/0005-*`
- `docs/ai-tools.md`, `CLAUDE.md` de `se-mcp`, `se-app`, `se-domain`

## Objectif
Les agents peuvent monter : ajouter des textes animés, couper, déplacer, régler le volume, poser des keyframes… uniquement dans la portée de leur tâche. Toutes leurs modifications s'appliquent en direct, sont attribuées à la tâche et forment un changeset annulable d'un coup.

## Périmètre
**Inclus**
- Outils d'écriture, contrôle de portée, concurrence optimiste, `apply_batch`, changesets liés aux tâches, commande d'annulation d'une tâche, instructions d'édition.

**Exclus**
- UX des tâches (panneau, overlays, bouton d'annulation) → 20 · coupes intelligentes → 22 · sous-titres → 23 · templates → 24.

## Spécifications

### `AgentEditService` (se-app)
- Point d'entrée unique des outils d'écriture : construit les opérations du domaine (ids via `IdGenerator`), vérifie la portée, commite via `ProjectActor` avec l'acteur `AiTask(task_id)`, rattache la transaction au changeset de la tâche.
- Les éléments créés ont `origin = AiTask(task_id)`.

### Contrôle de portée
- Écriture autorisée si l'élément (avant **et** après modification) est dans la plage de la portée et sur une piste autorisée ; création autorisée uniquement dans la portée ; lecture autorisée partout.
- Une tâche sans plage (portée = séquence entière) est possible si l'utilisateur l'a choisie.
- Erreur `OUT_OF_SCOPE` explicite (plage de la tâche, plage fautive, suggestion).

### Outils d'écriture (minimum)
`add_text`, `add_shape`, `add_image`, `update_item` (patch typé), `delete_items`, `move_item`, `trim_item`, `split_at`, `ripple_delete_range`, `insert_gap`, `set_keyframes`, `apply_animation_preset`, `set_volume` (gain + automation), `add_transition`, `add_marker`, et **`apply_batch`** (liste d'actions atomique : tout ou rien).
- Paramètre optionnel `expected_revision` sur les outils qui modifient un élément → `REVISION_CONFLICT` si l'utilisateur l'a changé entre-temps (l'agent doit relire).
- Sorties : confirmation brève, ids créés, nouvelles révisions.
- Erreurs : `OUT_OF_SCOPE`, `REVISION_CONFLICT`, `VALIDATION_ERROR` (invariant violé + message du domaine), `NOT_FOUND`, `LOCKED_TRACK`.

### Changesets et annulation
- Le `TaskScheduler` associe chaque run à un `Changeset` (créé au premier commit, persistant dans `project.db`).
- Commande protocole `RevertTask { task_id }` → `plan_revert` du domaine → commit utilisateur → réponse avec les conflits (éléments ignorés et pourquoi).
- Événement `TaskChangesetUpdated { task_id, touched_items }` pour que l'UI puisse surligner les modifications.

### Instructions des agents (nouveaux fichiers)
- `03-editing-rules.md` : rester dans la portée, préférer `apply_batch` pour un ensemble cohérent, vérifier visuellement avec `get_frames` après un changement visuel, gérer `REVISION_CONFLICT` en relisant, terminer par un résumé court de ce qui a été fait.
- `04-motion-basics.md` : presets vs keyframes, easing, typographie pour vidéos courtes, lisibilité, respect des safe areas, durées confortables de lecture.
- `05-audio-editing.md` : coupes propres (couper dans les silences), fondus courts pour éviter les clics, niveaux.

### Mise à jour de `docs/ai-tools.md`
Tous les outils d'écriture, avec exemples d'usage typiques (« De 51 s à 54 s, affiche “Abonne-toi” au centre avec un pop »).

## Contraintes
- Aucun outil n'écrit autrement que via `AgentEditService` → `ProjectActor`.
- Les messages d'erreur sont rédigés pour qu'un agent puisse se corriger seul.

## Critères d'acceptation
- [ ] Tâche réelle : « De 51 s à 54 s affiche le texte “Abonne-toi” en gros au centre avec une animation pop » → un élément texte éditable, avec keyframes, dans la plage.
- [ ] Une tentative hors portée est refusée et l'agent se corrige (visible dans le journal de la tâche).
- [ ] `RevertTask` annule toute la tâche ; avec un élément modifié entre-temps par l'utilisateur, le conflit est rapporté et le reste est annulé.
- [ ] `apply_batch` est atomique (un échec au milieu = aucune modification).

## Tests attendus
- Matrice de portée (dedans, à cheval, dehors, piste interdite, création).
- Atomicité de `apply_batch`, `expected_revision`.
- Rejeu bout-en-bout : fixture où l'agent appelle `add_text` puis `apply_animation_preset` → projet attendu ; `RevertTask` → projet initial.
- Snapshots des schémas.

## Documentation à mettre à jour
- `docs/ai-tools.md`, `crates/se-mcp/CLAUDE.md`, `crates/se-app/CLAUDE.md` (AgentEditService), `docs/architecture.md` §9.4.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
