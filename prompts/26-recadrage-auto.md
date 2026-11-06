# 26 — Recadrage automatique 16:9 → 9:16

> **Phase** : 6 — Motion design avancé · **Dépend de** : 16, 19 · **Parallélisable avec** : 25

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §5.3, §7, §14
- `docs/adr/0007-*`, `docs/adr/0012-*` (unités spatiales)
- `CLAUDE.md` de `se-vision`, `se-media`, du domaine, du moteur, de l'app

## Objectif
Créer en un clic une version portrait d'un montage paysage dont le cadre suit automatiquement le sujet (visage/personne), avec un mouvement fluide, entièrement ajustable à la main ensuite.

## Périmètre
**Inclus**
- Crate `se-vision` (ONNX Runtime), modèle de détection, job d'analyse, lissage de trajectoire, génération de keyframes de recadrage, intégration UI et MCP.

**Exclus**
- Diarisation / détection du locuteur actif par l'audio → backlog.

## Spécifications

### `se-vision`
- ONNX Runtime via la crate `ort`, stratégie de chargement documentée par plateforme (bibliothèque embarquée ou chargement dynamique), accélération GPU si disponible (CUDA/DirectML/CoreML) avec repli CPU. Mettre à jour `ci.yml` et `release.yml` pour que les builds des trois plateformes embarquent le runtime.
- Modèle de détection de visages **sous licence permissive** (ex. YuNet de l'OpenCV Zoo, ou équivalent) ; **vérifie et documente la licence** ; exclure tout modèle sous licence copyleft forte (AGPL…). Téléchargement avec consentement + empreinte, stockage `<app data>/models/vision/`.
- Port `SubjectDetector` : `detect(frames) -> détections (boîte, score)`.

### Job `SubjectDetection`
- Échantillonnage d'images basse résolution via ffmpeg (ex. 5 i/s), détection, choix du sujet principal (persistance + taille + centralité), interpolation entre échantillons.
- Calcul de la fenêtre de recadrage au ratio cible, contrainte dans l'image, marges de cadrage (règle des tiers verticale configurable).
- **Lissage** (filtre One-Euro ou Kalman, paramètres réglables) puis **simplification** de la trajectoire en keyframes (Ramer–Douglas–Peucker) pour rester éditable.
- Changements de plan (prompt 09) : réinitialisation du suivi à chaque coupe (pas de glissement entre deux plans).
- Résultat : `Reframe::Tracked { keyframes }` sur le clip.

### UI
- Dialogue « Créer une version… » (prompt 16) : case « Recadrage automatique (suivi du sujet) ».
- Par clip : « Recadrer automatiquement » / « Relancer », visualisation de la trajectoire dans le viewer, ajustement en déplaçant la fenêtre (crée/modifie des keyframes), réglage de la fluidité.

### MCP
- `auto_reframe { sequence | items, target_format, smoothing? }` (crée une nouvelle séquence ou applique aux clips de la portée) ; mise à jour de `docs/ai-tools.md`.

## Contraintes
- Toute la géométrie (fenêtre, lissage, simplification) est pure et testée, indépendante du modèle.
- Aucune image ne quitte la machine.

## Critères d'acceptation
- [ ] Une interview paysage devient une version 9:16 où le visage reste cadré, sans à-coups, avec des keyframes modifiables.
- [ ] Aux changements de plan, le cadre saute proprement au nouveau sujet.
- [ ] Un agent peut déclencher le recadrage sur sa portée.

## Tests attendus
- Calcul de la fenêtre (bords, ratios), lissage, simplification, réinitialisation aux coupes — sur des détections simulées.
- Intégration du détecteur sur une image de test (si le modèle est présent ; sinon ignoré avec message).

## Documentation à mettre à jour
- `crates/se-vision/CLAUDE.md` (modèle, licence, chargement), `docs/ai-tools.md`, `docs/glossary.md`.

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
