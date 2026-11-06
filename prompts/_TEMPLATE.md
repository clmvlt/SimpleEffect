# NN — Titre de la fonctionnalité

> **Phase** : … · **Dépend de** : NN, NN · **Parallélisable avec** : NN

## À lire avant de commencer
- `CLAUDE.md`, puis `docs/architecture.md` (sections concernées : §…)
- ADR concernées : `docs/adr/NNNN-….md`
- Les `CLAUDE.md` des crates/packages que tu vas modifier
- Les dernières entrées de `prompts/PROGRESS.md`

## Objectif
Une ou deux phrases : ce que l'utilisateur pourra faire à la fin de ce prompt.

## Périmètre
**Inclus**
- …

**Exclus** (traité ailleurs)
- … → prompt NN

## Spécifications
Détails fonctionnels et techniques : modèle de données, commandes du protocole, outils MCP, composants UI, comportements clavier, cas limites.

## Contraintes
- Rappels spécifiques (performance, sécurité, plateforme, licences…).

## Critères d'acceptation
- [ ] Comportement observable et vérifiable n° 1
- [ ] …

## Tests attendus
- Unitaires : …
- Intégration : …
- UI / golden frames : …

## Documentation à mettre à jour
- `docs/…`, `CLAUDE.md` du module, ADR si une décision est prise, `docs/glossary.md` si nouveau terme.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine : `pnpm check` vert, prompt coché dans `prompts/README.md`, entrée ajoutée dans `prompts/PROGRESS.md`.
