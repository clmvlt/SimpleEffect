# 13 — Moteur B : lecture temps réel, audio, synchro A/V ⚠️

> **Phase** : 3 — Montage · **Dépend de** : 12 · **Parallélisable avec** : 17

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §8
- `docs/adr/0002-web-rendering-engine.md`
- `packages/engine/CLAUDE.md`, `prompts/PROGRESS.md`

## Objectif
Lire la séquence en temps réel avec le son, sans dérive audio/vidéo, avec les commandes de transport d'un vrai logiciel de montage (JKL, boucle, image par image) et des VU-mètres.

## Périmètre
**Inclus**
- `PlaybackController`, `AudioEngine` (Web Audio), synchronisation, préchargement, scrub audio, VU-mètres, HUD de performance, barre de transport.

**Exclus**
- Automation de volume, fondus, transitions → 14.

## Spécifications

### Lecture
- `PlaybackController` : lecture/pause (Espace), stop, JKL (vitesses −4, −2, −1, 1, 2, 4 ; K = pause ; K+J/L = image par image), boucle sur la plage in/out, aller au début/à la fin.
- L'horloge maître est `AudioContext.currentTime` ; la vidéo affiche l'image correspondant à l'horloge audio et **saute** les images en retard (compteur d'images perdues).
- Préchargement vidéo d'une fenêtre devant la tête de lecture (taille adaptative), priorité à l'image courante.
- La timeline suit la tête de lecture (défilement par page) ; mises à jour de position sans tempête de re-rendus React.

### Audio (`packages/engine/src/audio/`)
- `AudioEngine` : graphe Web Audio (clip → gain de clip → gain/pan de piste → master), respect muet/solo/volume des pistes.
- Décodage par fenêtres via Mediabunny (échantillons audio) et ordonnancement avec anticipation (~200 ms) de `AudioBufferSourceNode`s alignés à l'échantillon près.
- Gestion des changements de vitesse (JKL) : lecture audio uniquement à 1× (silence aux autres vitesses), documenté.
- **Scrub audio** : courts extraits lors du déplacement de la tête de lecture (désactivable).
- **VU-mètres** : AudioWorklet calculant crête/RMS par canal → composant `Meter` du master dans la barre de transport.

### UI
- `TransportBar` branchée (viewer), timecode courant et durée, bouton boucle, bascule scrub audio.
- HUD de performance (bascule dans le viewer) : i/s réel, images perdues, file de décodage, mémoire du cache.

## Contraintes
- Rien de lourd sur le thread principal pendant la lecture ; pas d'allocation par image évitable.
- Le comportement doit être identique sur Windows, macOS et Linux (pas d'API spécifique).

## Critères d'acceptation
- [ ] Une séquence 1080p multi-pistes se lit à la cadence de la séquence sans dérive mesurable sur 10 minutes.
- [ ] JKL, boucle in/out et image par image fonctionnent.
- [ ] Muet/solo/volume de piste s'appliquent immédiatement pendant la lecture.
- [ ] Les VU-mètres réagissent en temps réel.

## Tests attendus
- Unitaires : ordonnanceur audio avec horloge simulée (alignement, anticipation, seek), sélection des images à afficher selon l'horloge (saut des images en retard).
- Test navigateur : média de test « flash + bip » chaque seconde, mesure de l'écart A/V (ou, si non automatisable, checklist manuelle documentée dans `packages/engine/CLAUDE.md`).

## Documentation à mettre à jour
- `packages/engine/CLAUDE.md` (horloge, ordonnancement, limites connues).

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
