# 17 — Runtime des agents IA (générique + Claude Code) ⚠️

> **Phase** : 4 — IA · **Dépend de** : 07 · **Parallélisable avec** : 08 → 16

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §9.1 à §9.3, §13, §14
- `docs/adr/0003-ai-agents-via-cli-and-local-mcp.md`
- `CLAUDE.md` de `se-app`, `se-agents`, `se-core`

## Objectif
Lancer des tâches IA en arrière-plan via le CLI installé sur la machine (Claude Code d'abord), en parallèle et avec une limite de concurrence, derrière une abstraction générique qui permettra d'ajouter d'autres CLI plus tard. L'app détecte si le CLI est installé et connecté, sans jamais demander de login.

## Périmètre
**Inclus**
- Ports `AgentProvider` & co., `TaskScheduler`, persistance des tâches, provider Claude Code, provider de rejeu pour les tests, réglages IA, indicateur d'état dans la barre d'état, validation empirique des flags.

**Exclus**
- Serveur MCP et outils → 18/19 · UX complète des tâches (composeur, panneau, révision) → 20.

## Spécifications

### Ports (`se-app::ports`)
Tels que décrits dans l'architecture §9.1 : `AgentProvider` (`id`, `capabilities`, `detect`, `auth_status`, `start`), `AgentRunSpec`, `AgentRun` (flux d'`AgentEvent` + `CancelHandle`), `AgentEvent`, `AgentFailureKind`, `ProviderCapabilities` (reprise de session, entrée image, liste des modèles/alias, niveaux d'effort), `AuthStatus` (`LoggedIn { account_kind }`, `LoggedOut`, `Unknown(reason)`).

### `TaskScheduler` et `TaskService` (`se-app`)
- `Task` (architecture §9.3) persistée dans `project.db` (`tasks`, `task_events`) ; statuts `Queued → Running → Completed | Failed | Cancelled`.
- File FIFO, sémaphore de concurrence (réglage 1–8, défaut 3), annulation (en file ou en cours), relance, **suite** (`follow_up(task, prompt)` → nouveau run qui reprend la session de l'agent).
- Dossier de travail vide et propre par run dans `<app data>/agent-workdirs/<task-id>/`, supprimé à la fin.
- Événements protocole : `TaskUpdated { task }`, `TaskEventAppended { task_id, event }`, `AiStatusChanged { provider, installation, auth }`.
- Commandes : `SubmitTask { prompt, scope, provider?, model?, effort? }`, `CancelTask`, `RetryTask`, `FollowUpTask`, `ListTasks`, `GetAiStatus`, `RefreshAiStatus`.
- À ce stade, pas encore de MCP : `AgentRunSpec.mcp` est optionnel (rendu obligatoire au prompt 18).

### Provider Claude Code (`crates/se-agents/src/providers/claude_code/`)
- Résolution du binaire : chemin configuré → crate `which` (gère PATHEXT ; sous Windows `claude` est souvent un shim `.cmd`, et sur la machine de dev il se trouve dans `~/.claude-shim/`).
- `detect` : `claude --version` (parse de la version) ; `auth_status` : `claude auth status --json` (champs `loggedIn`, `authMethod`, `subscriptionType` ; ne **jamais** journaliser l'e-mail ni l'organisation).
- `start` : construit la commande de l'architecture §9.2 (`-p`, `--output-format stream-json`, `--verbose`, `--tools ""`, `--permission-prompts none`, `--append-system-prompt`, `--model`, `--effort`, `--session-id <uuid>` pour un nouveau run, `--resume <uuid>` pour une suite), lance sans fenêtre console, lit stdout ligne par ligne.
- Parser stream-json → `AgentEvent` : init système (id de session), messages assistant (texte, tool_use), résultats d'outils, message final `result` (coût, durée, tokens, sous-type succès/erreur). Lignes inconnues : journalisées en debug et ignorées (compatibilité future).
- Classification des échecs : non installé, non connecté, limite d'usage atteinte, crash, annulé, erreur de protocole — avec messages utilisateur traduits côté UI.
- Annulation : arrêt de **l'arbre** de processus (Windows : job object ou `taskkill /T /F` ; Unix : groupe de processus).

### Validation empirique (obligatoire)
Avec l'accord de l'utilisateur (cela consomme un peu de quota), dans un dossier temporaire :
1. Lance quelques `claude -p` courts pour **enregistrer des fixtures** stream-json réelles (succès, texte long, erreur de session si possible) dans `crates/se-agents/tests/fixtures/claude_code/`.
2. Vérifie l'isolation : le `CLAUDE.md` global de l'utilisateur est-il chargé malgré le dossier vide ? Que changent `--setting-sources`, `--safe-mode`, `--bare` (attention : `--bare` exige une clé API et est donc exclu) ? Choisis la combinaison la plus isolée **compatible avec la session abonnement**.
3. Consigne les résultats (version du CLI, flags retenus, comportements observés) dans le `CLAUDE.md` du provider.

### Provider de rejeu (tests)
`ReplayProvider` : rejoue une fixture avec des délais simulés ; utilisé par tous les tests de `TaskScheduler` et plus tard de l'UX.

### UI
- `features/settings` (crée la feature et l'écran de réglages s'ils n'existent pas encore) section « IA » : CLI détecté (chemin, version), état de connexion avec bouton « Actualiser », chemin personnalisé, modèle par défaut (alias des capacités), effort par défaut, nombre max de tâches simultanées.
- Barre d'état : pastille IA verte (connecté), orange (déconnecté — infobulle « Ouvre un terminal, lance `claude`, puis tape `/login` »), grise (non installé — lien vers l'aide).
- Écran de développement caché (dev uniquement) « Tester une tâche » qui envoie un prompt simple et affiche le flux d'événements.

## Contraintes
- Seul le dossier `providers/claude_code/` connaît Claude Code ; tout le reste utilise le trait.
- L'app ne lit, ne stocke et ne transmet jamais d'identifiants.

## Critères d'acceptation
- [ ] Les réglages affichent la version du CLI et l'état de connexion réel.
- [ ] Déconnecter le CLI (`claude auth logout` dans un terminal, à faire par l'utilisateur) → pastille orange après actualisation ; fonctions IA désactivées.
- [ ] 5 tâches soumises avec une limite de 3 → 3 en cours, 2 en file ; l'annulation d'une tâche en cours tue bien le processus.
- [ ] Une suite (`follow_up`) reprend la même session (vérifié via l'id de session).

## Tests attendus
- Parser sur les fixtures réelles et sur des lignes malformées.
- `TaskScheduler` avec `ReplayProvider` : concurrence, ordre, annulation, relance, suite, persistance/reprise après redémarrage du cœur (les tâches « en cours » deviennent « échouées : interrompues »).
- Parsing de `auth status`.

## Documentation à mettre à jour
- `crates/se-agents/CLAUDE.md` + `providers/claude_code/CLAUDE.md` (résultats empiriques), `docs/architecture.md` §9.2 si les flags changent, `docs/glossary.md`.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
