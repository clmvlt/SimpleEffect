# 24 — Modèles de motion générés par l'IA ⚠️

> **Phase** : 6 — Motion design avancé · **Dépend de** : 19 · **Parallélisable avec** : 23

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §8, §10, §13
- `docs/adr/0011-motion-templates.md`
- `docs/ai-tools.md`, `CLAUDE.md` du moteur, de `se-mcp`, de l'app

## Objectif
Permettre le motion design « complet » : l'agent écrit un module d'animation (code) accompagné d'un schéma de paramètres ; le module s'exécute en sandbox, se rend à l'identique en preview et à l'export, et l'utilisateur modifie textes, couleurs et timings depuis un inspecteur généré automatiquement — sans toucher au code.

## Périmètre
**Inclus**
- Contrat des modules, compilation et validation, runtime sandboxé en Worker, élément `MotionTemplate` dans le moteur, inspecteur généré, outils MCP, instructions agents avec exemples.

**Exclus**
- Bibliothèque globale et brand kit → 25.

## Spécifications

### Contrat d'un template (TypeScript)
```ts
export const meta = { name: string, description: string, defaultDuration: number /* seconds */ };
export const params = { /* sous-ensemble de JSON Schema */ };
export function render(ctx: TemplateRenderContext, time: number /* seconds, item-relative */, params: Params): void;
```
- `TemplateRenderContext` : `width`, `height`, `duration`, API de dessin **restreinte** (sous-ensemble Canvas2D : formes, chemins, dégradés, texte avec helper de mise en page multi-ligne, images d'assets autorisés via des handles, transformations, `globalAlpha`, modes de fusion), helpers `ease.*`, `lerp`, `clamp`, `random(seed)` déterministe.
- Interdits : `Date`, `Math.random`, `fetch`, timers, DOM, accès réseau — **le rendu doit être déterministe** (même entrée = même image) pour garantir la parité preview/export.
- Types de paramètres : `string` (dont multiligne), `number` (min/max/pas, animable), `color` (animable), `boolean`, `enum`, `font`, `asset`.

### Compilation et validation (renderer)
- Transpilation TS → JS avec un transpileur embarqué léger (ex. `sucrase` ; pas de réseau).
- Validation : exports présents, schéma valide, rendu d'essai de quelques instants (exceptions, budget de temps par image), détection de non-déterminisme (deux rendus du même instant comparés).
- Résultat renvoyé à l'agent : OK + vignette, ou erreurs avec numéros de ligne et message clair pour qu'il corrige.
- Le cœur passe par le même aller-retour renderer que `get_frames` (prompt 18).

### Sandbox
- Worker dédié par template (créé depuis un blob), CSP sans connexion, globals dangereux supprimés/neutralisés (`fetch`, `XMLHttpRequest`, `WebSocket`, `importScripts`, `indexedDB`, `caches`…), `OffscreenCanvas` enveloppé par l'API restreinte.
- Budget par image (ex. 50 ms) : dépassement → Worker terminé, erreur rapportée.
- Sortie : `ImageBitmap` transféré au compositeur ; cache des images rendues par (template, version, params, instant).

### Domaine et moteur
- `MotionTemplate { id, name, version, params_schema, source, default_duration, thumbnail }` dans le projet ; `ItemKind::MotionTemplate { template_id, params }` avec paramètres numériques/couleurs `Animatable`.
- `evaluate` résout les paramètres à l'instant `t` ; le compositeur dessine le bitmap du runtime.

### Inspecteur généré
- Contrôles créés depuis le schéma : texte → Input/Textarea, nombre → `ScrubInput` (+ keyframes), couleur → `ColorField` (+ keyframes), booléen → Switch, enum → Select, police → sélecteur de polices, asset → sélecteur de médias.
- Bouton « Voir le code » (lecture seule) et « Demander une modification à l'IA » (composeur prérempli).

### MCP
- `create_motion_template { name, description, source, params_schema, default_duration }` → résultat de validation.
- `update_motion_template { template_id, source?, params_schema? }` (nouvelle version ; les éléments existants peuvent être migrés si le schéma est compatible).
- `instantiate_template { template_id, range, track?, params }`, `update_template_params { item_id, params }`.
- Instructions `08-motion-templates.md` : référence de l'API, règles de déterminisme, bonnes pratiques (timing, easing, lisibilité), 4 exemples complets (lower third, titre animé, barre de progression, call-to-action), toujours vérifier avec `get_frames`.
- Mise à jour de `docs/ai-tools.md`.

## Contraintes
- Le code généré n'a **aucun** accès au système, au réseau ni au protocole.
- Un template invalide n'est jamais ajouté au projet.

## Critères d'acceptation
- [ ] Tâche réelle : « crée un lower third animé avec mon nom et mon titre de 0:05 à 0:10 » → template valide, élément éditable ; changer le nom dans l'inspecteur met à jour le rendu.
- [ ] Un template qui tente `fetch` ou une boucle infinie est rejeté avec une erreur claire renvoyée à l'agent.
- [ ] Le rendu d'un template est identique entre deux exécutions (déterminisme vérifié).

## Tests attendus
- Sandbox (mode navigateur) : accès interdits, dépassement de budget, déterminisme.
- Validation (exports manquants, schéma invalide, exception à l'exécution).
- Mapping schéma → contrôles ; golden frame d'un template d'exemple.

## Documentation à mettre à jour
- `docs/ai-tools.md`, `docs/architecture.md` §10, `CLAUDE.md` du moteur (runtime) et de `se-mcp`, `docs/glossary.md`.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
