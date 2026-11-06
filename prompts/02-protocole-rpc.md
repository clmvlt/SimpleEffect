# 02 — Protocole typé et couche RPC

> **Phase** : 0 — Fondations · **Dépend de** : 01 · **Parallélisable avec** : 03, 05

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §5.6, §6, §13
- `docs/adr/0010-typed-protocol-codegen.md`
- `crates/se-protocol/CLAUDE.md`, `crates/se-core/CLAUDE.md`, `packages/protocol/CLAUDE.md`

## Objectif
Remplacer le ping écrit à la main par un protocole complet et typé de bout en bout : les types Rust génèrent les types TypeScript, un client `CoreClient` robuste (corrélation, timeouts, reconnexion, événements) est utilisé par le renderer.

## Périmètre
**Inclus**
- Enveloppes, commandes, résultats, événements et erreurs dans `se-protocol` ; génération TS ; vérification de fraîcheur.
- Dispatcher de commandes et diffusion d'événements dans `se-core` ; logs structurés.
- `CoreClient` dans `@se/protocol` ; intégration minimale dans le renderer.

**Exclus**
- Commandes métier (projets, médias…) → ajoutées par les prompts suivants.

## Spécifications

### `se-protocol`
- `RequestEnvelope { id: u64, command: Command }`, `ResponseEnvelope { id, outcome: Ok(CommandResult) | Err(ProtocolError) }`, `EventEnvelope { event: Event }`, sérialisés avec un champ `type` (`request` | `response` | `event`).
- `Command` et `CommandResult` : enums internes taggés `kind` en camelCase. Commandes initiales : `Ping`, `GetCoreInfo` (version, plateforme, dossier de données de l'app, dossier de logs).
- `Event` : `CoreReady`, `CoreShuttingDown`.
- `ProtocolError { code: ErrorCode, message: String, details: Option<serde_json::Value> }` avec `ErrorCode` stable (`INVALID_REQUEST`, `UNKNOWN_COMMAND`, `INTERNAL`, `TIMEOUT`… à compléter par les prompts suivants).
- Tous les types dérivent `ts_rs::TS`.

### Codegen
- `pnpm codegen` exporte les types de `se-protocol` (et plus tard `se-domain`) vers `packages/protocol/src/generated/` (un fichier par type + `index.ts`). Choisis le mécanisme ts-rs le plus simple (tests d'export ou petit binaire `se-codegen`), documente-le.
- `pnpm codegen --check` génère dans un dossier temporaire et compare : échec si différent. Ajouté à `pnpm check`.
- En-tête « GENERATED — DO NOT EDIT » dans chaque fichier généré.

### `se-core`
- `RpcRouter` : désérialise, dispatch vers un handler par commande (un fichier par groupe de commandes), sérialise la réponse ; erreurs de désérialisation → `INVALID_REQUEST` avec message utile.
- Diffusion d'événements : `tokio::sync::broadcast` vers toutes les connexions WebSocket ; la file d'une connexion lente ne doit pas bloquer les autres (événements perdus → la connexion reçoit un événement `ResyncRequired`).
- Auth : jeton en query `?t=` + contrôle `Host` (déjà fait au 01, à garder et tester).
- Logs : `tracing` + `tracing-appender`, fichiers journaliers dans `<app data>/logs/` (rétention 7 jours) + stderr en dev ; niveau via `SE_LOG`. Les jetons ne sont jamais logués.

### `@se/protocol`
- `CoreClient` : `connect({ url, token })`, `send<K extends Command['kind']>(command): Promise<ResultFor<K>>` (le type `ResultFor` est dérivé des types générés, sans table écrite à la main si possible), timeout par requête (défaut 30 s, surchargeable), `on(eventKind, handler)` → fonction de désabonnement, état de connexion observable (`connecting | open | reconnecting | closed`), reconnexion avec backoff exponentiel plafonné, rejet propre des requêtes en vol à la déconnexion.
- Aucune dépendance à React.

### Renderer
- `src/shared/core/` : création du client à partir de `getCoreConnection()`, contexte React, hooks `useCoreClient()`, `useConnectionStatus()`, `useCoreEvent(kind, handler)`.
- Le ping du prompt 01 utilise désormais le client typé ; afficher aussi `GetCoreInfo`.

## Contraintes
- Les messages restent du JSON lisible (débogage facile, outils IA).
- Aucune logique métier dans `se-protocol`.

## Critères d'acceptation
- [ ] Ajouter un champ à un type Rust sans relancer `pnpm codegen` fait échouer `pnpm check`.
- [ ] Le renderer affiche les infos du cœur via `GetCoreInfo`.
- [ ] Couper puis relancer le cœur : le client se reconnecte seul, les requêtes en vol échouent avec une erreur explicite.
- [ ] Les logs apparaissent dans `<app data>/logs/`.

## Tests attendus
- Rust : sérialisation aller-retour de chaque enveloppe ; dispatcher (commande inconnue, JSON invalide) ; intégration WebSocket avec 2 clients recevant le même événement.
- TS : `CoreClient` avec un faux WebSocket (corrélation des ids, timeout, reconnexion, désabonnement).

## Documentation à mettre à jour
- `docs/architecture.md` §6 si le format diffère ; `CLAUDE.md` de `se-protocol`, `se-core`, `packages/protocol` (comment ajouter une commande : recette pas à pas).

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
