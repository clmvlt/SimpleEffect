# 20 — UX IA : composeur, tâches parallèles, annulation

> **Phase** : 4 — IA · **Dépend de** : 19 · **Parallélisable avec** : 21

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §9.3
- `docs/adr/0003-*`, `docs/adr/0005-*`
- `CLAUDE.md` de l'app et de `@se/ui` (AiTaskCard, PromptComposer)

## Objectif
L'expérience centrale du logiciel : sélectionner un passage (ou écrire « de 51 s à 54 s… »), lancer un prompt, continuer à monter pendant que l'IA travaille, lancer d'autres prompts en parallèle, voir chaque tâche avancer en direct, puis garder, itérer ou annuler son résultat.

## Périmètre
**Inclus**
- Composeur de prompt avec puces de portée, panneau des tâches, overlays sur la timeline, annulation de tâche avec conflits, suites, « Refaire avec l'IA », notifications.

**Exclus**
- Outils spécialisés (coupes intelligentes, sous-titres, templates) → 22 à 26.

## Spécifications

### Composeur (`features/ai`)
- Basé sur `PromptComposer` : zone de texte, envoi Ctrl/⌘+Entrée, historique des prompts (↑ dans un champ vide).
- **Portée** sous forme de puces :
  - automatique depuis la plage in/out de la timeline, ou depuis les éléments sélectionnés (plage englobante + pistes) ;
  - **détectée dans le texte** : « de 51s à 54s », « entre 1:02 et 1:10 », « 00:51-00:54 », « à partir de 2 min », « les 10 premières secondes » → puce proposée (service de parsing FR pur et testé, avec la cadence de la séquence) ;
  - choix des pistes concernées et de la séquence ; option explicite « toute la séquence ».
- Sélecteurs de modèle et d'effort (capacités du provider), désactivés si l'IA est indisponible (avec raison).

### Panneau des tâches
- Liste (en cours d'abord, puis récentes) de `AiTaskCard` : statut, portée (cliquable → sélectionne la plage sur la timeline), durée écoulée, flux en direct (texte de l'assistant + appels d'outils en lignes compactes : « add_text “Abonne-toi” 00:51→00:54 »), coût/tokens.
- Actions : annuler (en cours/en file), **suite** (champ inline → reprise de la session), **annuler les modifications** (`RevertTask` ; dialogue listant les conflits éventuels), relancer, voir le journal complet.
- Le panneau reste utilisable pendant le montage ; aucune action utilisateur n'est bloquée par une tâche.

### Timeline
- Pour chaque tâche en cours : zone de portée ombrée de la couleur `ai` avec libellé (« IA #3 en cours »).
- Éléments créés/modifiés par une tâche : pulsation brève à l'arrivée du patch ; badge d'origine au survol (« Créé par la tâche #3 — voir »).
- Si l'utilisateur modifie un élément dans la portée d'une tâche en cours : autorisé, avec un indicateur discret (l'agent recevra `REVISION_CONFLICT` s'il le touche).

### « Refaire avec l'IA »
- Menu contextuel d'un élément (créé par l'IA ou non) → composeur prérempli avec la portée de l'élément et une référence à son id ; si l'élément vient d'une tâche, proposition de **reprendre sa session** (l'agent garde le contexte de ce qu'il a fait).

### Notifications
- Toast à la fin d'une tâche (succès/échec) avec action « Voir » ; notification système Electron si la fenêtre n'a pas le focus.

## Contraintes
- Toute la logique (parsing de timecodes, dérivation de la portée, regroupement des événements) est dans des services purs testés.
- Textes de l'UI via i18n ; les messages d'erreur du provider sont traduits par code.

## Critères d'acceptation
- [ ] Lancer 3 prompts sur 3 plages différentes, continuer à monter ailleurs : les 3 aboutissent, chacune est visible en direct et annulable séparément.
- [ ] Écrire « de 51s à 54s mets un titre » propose automatiquement la bonne puce de portée.
- [ ] « Refaire avec l'IA » sur un texte créé par l'IA reprend la session et modifie ce texte.
- [ ] Annuler une tâche dont un élément a été retouché à la main affiche le conflit et annule le reste.

## Tests attendus
- Parsing des timecodes en français (table de cas, y compris ambigus et invalides).
- Dérivation de la portée (sélection, in/out, texte).
- Store des tâches (application des événements, ordre), composants (AiTaskCard branché, composeur) avec `ReplayProvider` côté cœur pour un test d'intégration.

## Documentation à mettre à jour
- `apps/desktop/CLAUDE.md` (feature ai), `docs/glossary.md` si besoin.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
