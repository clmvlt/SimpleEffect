# Guide Git et releases

Dépôt : https://github.com/clmvlt/SimpleEffect (public : les releases sont téléchargeables par tout le monde).

## Branches

| Branche | Rôle | Qui y écrit |
|---|---|---|
| `main` | Production | Toi uniquement |
| `recette` | Tests | Toi uniquement |
| `dev` | Intégration (dernière version en développement) | Claude (et toi) |
| `feature/<NN>-<nom>` | Une fonctionnalité | Claude, puis fusion dans `dev` |

Flux : `feature/*` → `dev` → `recette` → `main`.

Claude part toujours d'un `dev` à jour, crée une branche `feature/…`, la pousse puis la fusionne dans `dev`, sans PR. Il ne touche jamais `recette` ni `main` et ne crée jamais de tag.

## Voir ce qui va être promu

```bash
git fetch origin
git log --oneline origin/recette..origin/dev
git log --oneline origin/main..origin/recette
```

## Promouvoir `dev` → `recette`

```bash
git fetch origin
git checkout recette
git pull --ff-only origin recette
git merge --no-ff origin/dev -m "chore: promote dev to recette"
git push origin recette
```

## Promouvoir `recette` → `main`

```bash
git fetch origin
git checkout main
git pull --ff-only origin main
git merge --no-ff origin/recette -m "chore: promote recette to main"
git push origin main
```

## Redescendre un correctif (`main` → `recette` → `dev`)

Si tu corriges directement sur `recette` ou `main`, redescends le correctif pour que `dev` ne le perde pas :

```bash
git fetch origin
git checkout recette && git pull --ff-only origin recette
git merge --no-ff origin/main -m "chore: back-merge main into recette"
git push origin recette

git checkout dev && git pull --ff-only origin dev
git merge --no-ff origin/recette -m "chore: back-merge recette into dev"
git push origin dev
```

## Releases (GitHub Actions)

> Les workflows `.github/workflows/ci.yml` et `release.yml` sont créés par le prompt 01.

Pousser un tag lance la construction des installeurs Windows (`.exe`), macOS (`.dmg`) et Linux (`.AppImage`), puis publie une release GitHub. La version de l'app est lue dans le tag (format `vMAJEUR.MINEUR.CORRECTIF`).

**Pré-release de test** (depuis `recette`) :

```bash
git checkout recette && git pull --ff-only origin recette
git tag v0.1.0-rc.1
git push origin v0.1.0-rc.1
```

**Release de production** (depuis `main`) :

```bash
git checkout main && git pull --ff-only origin main
git tag v0.1.0
git push origin v0.1.0
```

**Build manuel sans release** (les installeurs sont attachés au run ; le workflow doit déjà être présent sur `main`) :

```bash
gh workflow run release.yml --ref recette
```

**Suivre et télécharger** :

```bash
gh run list --workflow release.yml
gh run watch
gh release list
gh release download v0.1.0 --dir ./downloads
```

Page des téléchargements : https://github.com/clmvlt/SimpleEffect/releases

**Supprimer un tag ou une release ratés** :

```bash
gh release delete v0.1.0 --yes
git push --delete origin v0.1.0
git tag -d v0.1.0
```

## Bon à savoir

- La CI (`ci.yml`) lance `pnpm check` à chaque push sur `dev`, `recette`, `main` et `feature/**` : `gh run list --branch dev`.
- Les installeurs ne sont pas signés : sous Windows, SmartScreen affiche un avertissement (« Informations complémentaires » → « Exécuter quand même ») ; sous macOS, fais clic droit → « Ouvrir » au premier lancement.
