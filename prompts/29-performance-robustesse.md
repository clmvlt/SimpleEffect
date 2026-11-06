# 29 — Performance et robustesse

> **Phase** : 8 — Finition · **Dépend de** : 28 · **Parallélisable avec** : —

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` (en entier), `prompts/PROGRESS.md` (toutes les dettes notées)

## Objectif
Mesurer, puis rendre le logiciel rapide et incassable sur de vrais projets : longues timelines, footage 4K, plusieurs agents en parallèle, crashs et fichiers corrompus.

## Périmètre
**Inclus**
- Bancs de mesure, profilage, tests de charge, scénarios de panne, corrections des principaux goulets, budgets de performance documentés.

**Exclus**
- Nouvelles fonctionnalités.

## Spécifications

### Mesures (à automatiser autant que possible)
- Preview : i/s et images perdues en 4K multi-pistes (avec et sans proxies), temps de scrub (latence de la première image).
- Timeline : latence des interactions sur un projet de 1 h avec 500 éléments (script de génération du prompt 10).
- Mémoire : renderer et cœur après 30 minutes d'utilisation (fuites de `VideoFrame`, caches).
- Cœur : latence des commandes, des outils MCP, temps d'ouverture d'un gros projet, taille du journal.
- Export : vitesse (x temps réel) en 1080p et 4K.
- IA : 5 tâches parallèles + montage simultané (fluidité de l'UI).

### Outils
- Traces de performance Chromium (script documenté), spans `tracing` avec durées, `tokio-console` en dev, benchmarks Rust (`criterion`) sur `apply`/`commit` et la sérialisation.

### Robustesse
- Tuer `se-core` en pleine édition → reconnexion, aucune perte (journal).
- `project.json` corrompu → reprise depuis le dernier `.bak` + journal, message clair.
- Binaires manquants, modèle manquant, disque plein pendant un export, CLI IA qui plante ou se déconnecte en cours de tâche → erreurs claires, aucun état incohérent.
- Fermeture de l'app pendant un export ou une tâche IA → arrêt propre, tâches marquées interrompues.

### Livrables
- Corrections des goulets majeurs identifiés (avec mesures avant/après dans PROGRESS.md).
- `docs/performance.md` : budgets (ex. scrub < 100 ms, 0 image perdue en 1080p30 à 1×, interaction timeline < 16 ms), méthodes de mesure, résultats.

## Critères d'acceptation
- [ ] Les budgets de `docs/performance.md` sont tenus sur la machine de dev (ou les écarts sont documentés avec un plan).
- [ ] Tous les scénarios de panne listés se terminent sans perte de données.

## Tests attendus
- Benchmarks Rust, tests de charge automatisés (Playwright + trace) quand c'est possible, tests d'intégration des scénarios de panne côté cœur.

## Documentation à mettre à jour
- `docs/performance.md` (créé), `CLAUDE.md` des modules optimisés (règles de perf à respecter).

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
