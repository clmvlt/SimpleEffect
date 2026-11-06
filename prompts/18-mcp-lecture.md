# 18 — Serveur MCP : outils de lecture

> **Phase** : 4 — IA · **Dépend de** : 16, 17 · **Parallélisable avec** : 21

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §9.2, §9.4, §13
- `docs/adr/0003-ai-agents-via-cli-and-local-mcp.md`
- `CLAUDE.md` de `se-mcp`, `se-app`, `se-agents`, `se-core`, `packages/engine`

## Objectif
Les agents IA peuvent « voir » et « comprendre » le projet : `se-core` expose un serveur MCP local, chaque tâche reçoit un jeton lié à sa portée, et des outils de lecture renvoient la timeline, les analyses audio et **des images de la séquence** (avec les textes et animations composés) que Claude peut regarder.

## Périmètre
**Inclus**
- Serveur MCP (`rmcp`, HTTP streamable sur `/mcp`), jetons par tâche, outils de lecture, rendu d'images à la demande via le renderer, catalogue `docs/ai-tools.md`, instructions de base des agents.

**Exclus**
- Outils d'écriture et changesets → 19 · transcription réelle → 21 (outil présent mais renvoie « non disponible »).

## Spécifications

### Serveur et authentification
- Service `rmcp` en HTTP streamable monté sur `/mcp` dans le routeur axum de `se-core`.
- `TaskTokenRegistry` (se-app) : le `TaskScheduler` crée un jeton aléatoire au démarrage de chaque run et le révoque à la fin ; le jeton donne un `AgentContext { task_id, run_id, scope, sequence_id }`.
- Le provider Claude Code passe la config MCP en ligne (`--mcp-config` JSON `type: "http"` + en-tête `Authorization`), `--strict-mcp-config` et `--allowedTools "mcp__simpleeffect__*"`. `AgentRunSpec.mcp` devient obligatoire.
- Jetons masqués dans les logs.

### Conventions des outils (à documenter en tête de `docs/ai-tools.md`)
- Un outil = un fichier dans `crates/se-mcp/src/tools/`, schéma d'entrée via `schemars`, sortie concise pensée pour un LLM (texte structuré compact ou JSON court), pagination pour les grandes listes.
- Temps en entrée : secondes (nombre) ou timecode (`"00:00:51.000"`, `"51s"`, `"00:51"`) ; en sortie : `HH:MM:SS.mmm` + secondes. Conversion en flicks uniquement à la frontière.
- Les sorties incluent les ids et les révisions (pour la concurrence optimiste du prompt 19).
- Erreurs actionnables avec codes stables.

### Outils de lecture
- `get_project_overview` : séquences (format, durée), pistes résumées, nombre de médias, portée de la tâche, présence d'un brand kit.
- `get_timeline { sequence?, range?, tracks? }` : éléments qui intersectent la plage avec leurs propriétés clés.
- `get_item { item_id }` : détail complet (y compris keyframes, origine).
- `list_assets { kind?, query? }`.
- `get_audio_analysis { range | asset }` : silences, loudness, crêtes, ramenés au temps de la séquence quand une plage est fournie.
- `get_scene_changes { range | asset }`.
- `get_source_frames { asset, times[], max_width? }` : images du média source (ffmpeg, service du prompt 09), renvoyées comme **contenu image MCP**.
- `get_frames { times[] | range + count, max_width? }` : images **composées** de la séquence (textes, transformations, animations inclus). Mécanisme : le cœur envoie un événement `RenderFramesRequest` au renderer, qui rend hors écran via `@se/engine` et renvoie des JPEG par une route `/media/renders/...` ; délai maximal et erreur claire si aucune fenêtre n'est ouverte.
- `get_transcript { range }` : renvoie « transcription non disponible » jusqu'au prompt 21.
- `list_animation_presets`, `get_brand_kit` et `list_templates` (stubs explicites jusqu'aux prompts 24/25).

### Instructions des agents
- `crates/se-app/assets/agent-instructions/` : `00-role.md` (assistant de montage dans SimpleEffect, langue de réponse = langue du prompt), `01-time-and-scope.md` (formats de temps, portée), `02-reading-the-project.md` (quels outils utiliser pour comprendre avant d'agir, regarder les images).
- Fonction de composition : instructions + portée de la tâche + résumé du projet → `--append-system-prompt`. Neutres vis-à-vis du fournisseur.

### `docs/ai-tools.md`
Conventions + tableau des outils + pour chacun : description, schéma d'entrée, exemple de sortie, erreurs possibles. Ce document est le **contrat** avec les agents.

## Contraintes
- Aucun outil de lecture ne modifie quoi que ce soit.
- Les sorties restent courtes (budget de tokens) : préférer la pagination et des résumés.

## Critères d'acceptation
- [ ] Une vraie tâche Claude « Décris ce qu'il se passe entre 0 et 10 s » répond correctement et utilise `get_frames`.
- [ ] Un jeton révoqué ou inconnu est refusé.
- [ ] `docs/ai-tools.md` couvre tous les outils exposés.

## Tests attendus
- Handlers avec un faux projet ; snapshots insta des schémas d'outils.
- Intégration : client MCP `rmcp` réel contre le serveur (jeton valide/invalide, appel d'outils, contenu image).
- Rejeu : fixture d'une tâche appelant `get_timeline`.

## Documentation à mettre à jour
- `docs/ai-tools.md` (créé), `crates/se-mcp/CLAUDE.md` (recette « ajouter un outil »), `docs/architecture.md` §9.4.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
