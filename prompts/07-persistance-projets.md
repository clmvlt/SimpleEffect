# 07 — Persistance et gestion des projets

> **Phase** : 1 — Domaine et projets · **Dépend de** : 02, 04, 06 · **Parallélisable avec** : —

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §3, §5.5, §5.6, §11
- `docs/adr/0005-single-writer-operations-changesets.md`, `docs/adr/0006-project-storage-layout.md`
- `CLAUDE.md` de `se-app`, `se-store`, `se-core`, `apps/desktop`

## Objectif
Créer, ouvrir, lister, renommer, dupliquer et mettre à la corbeille des projets ; toute modification passe par le `ProjectActor` (écrivain unique), est journalisée, sauvegardée automatiquement et récupérable après un crash. L'écran d'accueil devient fonctionnel et l'éditeur reçoit le projet en temps réel.

## Périmètre
**Inclus**
- Ports et cas d'usage dans `se-app`, `ProjectActor`, adaptateur `se-store`, commandes/événements du protocole, écran d'accueil, store miroir du projet dans le renderer, annuler/rétablir.

**Exclus**
- Import de médias → 08 · timeline → 10.

## Spécifications

### `se-app`
- Ports : `ProjectRepository` (create, load, save_snapshot, append_journal, read_journal_since, trash), `AppLibraryRepository` (projets récents, réglages clé/valeur), `EventSink`, `Clock`, `IdGenerator`. Implémentations factices derrière la feature `test-support`.
- `ProjectActor` : une tâche tokio par projet ouvert, boîte de réception (`Apply(Transaction)`, `Undo`, `Redo`, `Snapshot`, `Close`), flux de commit décrit dans l'architecture §5.5, sauvegarde du snapshot en différé (2 s après le dernier commit) et à la fermeture, lecteurs servis par `Arc<Project>`.
- `ProjectService` : `create_project(name, parent_dir, format)` → crée une séquence par défaut avec pistes V1, V2, A1, A2 ; `open_project(path)` (avec reprise du journal) ; `close_project` ; `list_recent` ; `rename_project` ; `duplicate_project` ; `trash_project` (corbeille de l'OS via la crate `trash`, jamais de suppression définitive) ; `create_sequence(name, format)` ; `undo` ; `redo`.

### `se-store`
- Dossier `<Nom>.seproj/` : `project.json` (écriture atomique : fichier temporaire + fsync + rename ; JSON indenté), `project.db` (SQLite WAL, migrations versionnées — tables `op_journal`, `tasks`, `task_events`, `transcripts` + FTS5, `analyses`, créées dès maintenant), `cache/`, `exports/`.
- Migrations du document : `schema_version` + fonctions pures `migrate_vN_to_vN+1(serde_json::Value)` ; v1 = version de base ; fixture de test.
- Reprise après crash : chargement du snapshot puis rejeu des entrées du journal de révision supérieure ; troncature du journal après un snapshot réussi ; conservation des 3 derniers snapshots (`project.json.bak.N`).
- Données de l'app via `dirs` : `library.db` (projets récents, réglages).

### Protocole
- Commandes : `CreateProject`, `OpenProject`, `CloseProject`, `ListRecentProjects`, `RenameProject`, `DuplicateProject`, `TrashProject`, `GetProjectSnapshot`, `CreateSequence`, `Undo`, `Redo`.
- Événements : `ProjectOpened { snapshot }`, `ProjectPatched { revision, upserts, deletes }` (entités entières : items, tracks, sequences, assets, markers…), `ProjectClosed`, `HistoryChanged { can_undo, can_redo, undo_label, redo_label }`.
- Nouveaux codes d'erreur (`PROJECT_NOT_FOUND`, `PROJECT_LOCKED`, `SCHEMA_TOO_NEW`…).
- Verrou : un fichier `.lock` empêche d'ouvrir le même projet deux fois.

### Renderer
- `src/shared/project-store/` : store miroir (Zustand) alimenté par le snapshot puis les patchs ; détection de trou de révision → `GetProjectSnapshot` ; sélecteurs typés ; **lecture seule pour les features**.
- `features/projects` :
  - Accueil : grille des projets récents (nom, badge de format, date d'ouverture, chemin), recherche, menu contextuel (renommer, dupliquer, mettre à la corbeille, afficher dans le dossier).
  - Dialogue « Nouveau projet » : nom, emplacement (dialogue natif via le preload), format via `AspectRatioPicker`, résolution, cadence (liste des cadences du domaine).
  - « Ouvrir un projet » : sélection d'un dossier `.seproj`.
- Éditeur : charge le projet de l'URL, affiche son nom et la séquence active ; raccourcis globaux Ctrl/⌘+Z et Ctrl/⌘+Maj+Z (via un registre de raccourcis minimal, enrichi au prompt 10).

## Contraintes
- Aucune écriture du document hors `ProjectActor`.
- Toutes les erreurs remontées à l'UI sont traduites (clés i18n par code d'erreur).

## Critères d'acceptation
- [ ] Créer un projet → il apparaît dans les récents et s'ouvre dans l'éditeur.
- [ ] Renommer, dupliquer, mettre à la corbeille fonctionnent (le dossier part dans la corbeille de l'OS).
- [ ] Tuer `se-core` juste après une modification → à la réouverture, la modification est là (journal).
- [ ] Ouvrir deux fois le même projet est refusé avec un message clair.
- [ ] Annuler/rétablir fonctionne au clavier et l'état est cohérent après reconnexion.

## Tests attendus
- `se-store` : intégration avec dossiers temporaires (écriture atomique, reprise du journal, migrations, verrou).
- `se-app` : tests de l'acteur avec les faux ports (ordre des commits, événements émis, debounce avec horloge simulée).
- Renderer : store miroir (application de patchs, trou de révision), dialogue de création (validation).

## Documentation à mettre à jour
- `docs/architecture.md` §11 si le format diffère, `CLAUDE.md` des modules touchés.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
