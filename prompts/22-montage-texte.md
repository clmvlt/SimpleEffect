# 22 — Montage par le texte et coupes intelligentes

> **Phase** : 5 — Transcription et audio intelligent · **Dépend de** : 14, 19, 21 · **Parallélisable avec** : —

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §5.4, §9.4
- `docs/ai-tools.md`, `CLAUDE.md` de `se-app`, `se-mcp`, de l'app (features transcript et timeline)

## Objectif
Couper la vidéo en éditant le texte (supprimer des mots = supprimer le passage), retirer en un clic les « euh » et les blancs, et offrir ces mêmes capacités aux agents IA. Toutes les coupes produites restent des coupes normales, ajustables à la main (allonger un blanc, rogner…).

## Périmètre
**Inclus**
- Montage par le texte, détection des mots de remplissage, suppression des silences paramétrable, outils MCP correspondants, instructions agents.

**Exclus**
- Sous-titres → 23.

## Spécifications

### Service de coupes (se-app, pur + use cases)
- `plan_cuts(word_ranges | silence_ranges, params) -> Vec<TimeRange>` : fusion des plages proches, marge avant/après (padding), durée minimale de coupe, respect des bornes des clips, alignement à l'image.
- Application : une **seule transaction** de suppressions ripple sur les éléments liés vidéo + audio des pistes concernées (pistes de graphiques non concernées par défaut, option documentée).

### Montage par le texte (`features/transcript`)
- Sélectionner des mots puis `Suppr` → coupe ripple correspondante ; annulable en une étape.
- Mode « afficher les passages coupés » : les mots retirés de la séquence apparaissent barrés/grisés (comparaison transcription du média ↔ couverture de la séquence) ; action « Restaurer » sur un passage barré (réinsertion à sa place logique si possible, sinon explication).

### Mots de remplissage
- Listes configurables FR (euh, heu, hum, bah, ben…) et EN (um, uh, you know…), détection sur la transcription + durée max.
- Panneau d'aperçu listant les occurrences (écoute de chacune, case à cocher) → « Supprimer la sélection ».

### Suppression des silences
- Dialogue : seuil (dBFS), durée minimale du silence, marge conservée avant/après, plage (sélection in/out ou séquence entière) ; **aperçu** des coupes sur la timeline (marqueurs temporaires) avant application.
- Après application, l'utilisateur peut allonger/raccourcir chaque blanc restant avec la poignée de blanc (prompt 11).

### MCP
- `remove_silences { range, threshold_db?, min_silence?, padding? }`, `remove_words { ranges: [{start, end}] }` (plages de mots issues de `get_transcript`), `remove_fillers { range, words? }` — même service que l'UI, contrôle de portée, attribution à la tâche.
- Instructions `06-smart-cuts.md` : utiliser transcription + analyse audio, garder des marges naturelles, ne jamais couper au milieu d'un mot, pour « garder la meilleure prise » comparer les répétitions dans la transcription, résumer les coupes faites (nombre, durée gagnée).
- Mise à jour de `docs/ai-tools.md`.

## Contraintes
- Une action utilisateur ou un appel d'outil = une transaction.
- Aucune coupe ne casse la synchronisation des éléments liés.

## Critères d'acceptation
- [ ] Supprimer une phrase dans la transcription coupe la vidéo et l'audio au bon endroit ; Ctrl/⌘+Z la restaure.
- [ ] « Supprimer les silences » avec aperçu produit des coupes propres, ensuite ajustables.
- [ ] Un agent à qui l'on demande « enlève les blancs et les euh entre 0 et 2 min » le fait dans sa portée et résume.

## Tests attendus
- `plan_cuts` : fusion, marges, durée minimale, bornes, alignement image (table de cas + proptests).
- Intégration : suppression ripple sur éléments liés (Rust).
- UI : sélection de mots → commande envoyée ; aperçu des silences.

## Documentation à mettre à jour
- `docs/ai-tools.md`, `CLAUDE.md` de `se-app` et de l'app.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
