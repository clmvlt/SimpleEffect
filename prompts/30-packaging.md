# 30 — Packaging multi-plateforme

> **Phase** : 8 — Finition · **Dépend de** : 29 · **Parallélisable avec** : —

## À lire avant de commencer
- `CLAUDE.md`, `docs/architecture.md` §2, §13, §14
- `docs/adr/0001-*`, `docs/adr/0007-*`

## Objectif
Installer SimpleEffect comme une vraie application sur Windows, macOS et Linux, avec un assistant de premier lancement qui vérifie le CLI IA, les binaires et propose les modèles optionnels.

## Périmètre
**Inclus**
- Configuration `electron-builder`, build release de `se-core` par plateforme, embarquement des binaires, notices de licences, assistant de premier lancement, écran « À propos », export de diagnostics.

**Exclus**
- Mise à jour automatique (aucun appel réseau) · signature/notarisation payantes (usage personnel).

## Spécifications

### Builds
- `electron-builder` (configuré au prompt 01) finalisé : Windows NSIS (x64), macOS dmg (arm64 et x64, signature ad hoc ; documente le contournement Gatekeeper pour un usage personnel), Linux AppImage + deb, icônes, métadonnées.
- `.github/workflows/release.yml` (créé au prompt 01) finalisé : `se-core` release et tous les binaires/runtimes embarqués pour chaque plateforme, mac x64 ajouté si absent, nom des fichiers d'installeur stables, notes de release lisibles. Les commandes de `GUIDE-GIT.md` doivent rester exactes.
- Binaires ffmpeg/ffprobe/whisper de la plateforme embarqués depuis `resources/bin/<platform>/` ; résolution des chemins en mode empaqueté testée.

### Licences
- Dossier `LICENSES/` et écran dans « À propos » : ffmpeg (GPL/LGPL selon le build), whisper.cpp, ONNX Runtime, modèles (licences vérifiées aux prompts 21 et 26), dépendances npm et cargo (génération automatisée d'un fichier de notices).

### Premier lancement
- Assistant : détection du CLI Claude (installé ? connecté ? avec instructions sinon), vérification des binaires, résumé des capacités GPU/codecs, téléchargements optionnels des modèles (Whisper, détection de visages) **avec consentement**, dossier par défaut des projets.

### À propos et diagnostics
- Versions : app, `se-core`, moteur, Electron, Chromium, CLI IA détecté.
- « Exporter un diagnostic » : zip des logs et infos système, **sans contenu de projet ni identifiants**.

### Checklist de validation par OS
- `docs/release-checklist.md` : installation, premier lancement, création de projet, import, lecture, tâche IA, export vidéo, export Premiere, désinstallation.

## Critères d'acceptation
- [ ] Un installeur Windows fonctionnel est produit et passe la checklist sur la machine de dev.
- [ ] Les procédures macOS et Linux sont documentées (et testées si l'utilisateur dispose des machines).
- [ ] L'assistant de premier lancement guide correctement quand le CLI Claude est absent ou déconnecté.

## Tests attendus
- Test de résolution des chemins de ressources en mode empaqueté, test du contenu du zip de diagnostic (absence de données sensibles).

## Documentation à mettre à jour
- `README.md` (installation), `docs/release-checklist.md`, `CLAUDE.md` racine (commandes de build).

## Clôture
Applique la *Definition of done* du `CLAUDE.md` racine.
