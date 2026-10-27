# Récapitulatif temporaire - BricarOBD

Mise à jour : 27 octobre 2026. Ce fichier permet de reprendre le projet dans un nouveau chat.
Il est temporaire et peut être supprimé après transmission.

## Portée de cette discussion

La demande actuelle porte sur la transmission, le nettoyage justifié, les commits/pushes
et la consolidation des branches vers main. Les informations de produit ci-dessous
sont tirées des sources, de la documentation et de l'historique du dépôt ; elles ne
prétendent pas reconstituer toutes les conversations antérieures.

Révision de départ : `55bc50721c15ca116ce78b22db4069854c22d2b3`. Inventaire initial : 253 fichiers suivis.

## Produit et fonctionnement

Application de diagnostic automobile OBD-II locale avec interface FR/EN, mode démonstration et connexion ELM327 série/Wi-Fi/BLE.

React 19, TypeScript 6, Vite 8 et Tailwind CSS 4 côté interface ; Tauri 2/Rust, serialport et rusqlite côté natif. Les appels invoke() passent par src-tauri/src/commands/. Les modules obd/ contrôlent les transports, protocoles, PID, DTC et la sécurité. La grande base SQLite est fournie séparément, pas dans Git.

## Choix de design à préserver

Tableau de bord automobile sombre à surfaces vitrées, jauges et graphiques temps réel. Huit pages fonctionnelles, notifications explicites, console développeur bornée et traductions react-i18next. Ne jamais mélanger données démo et connexion réelle.

## Déjà réalisé avant ce nettoyage

- Les transports et SafetyGuard imposent la lecture seule hors opérations explicitement autorisées.
- Les écritures ECU, tests actionneurs et régénérations restent désactivés sans profil véhicule/ECU vérifié.
- Les mises à jour npm/Cargo/Actions précédentes sont intégrées dans 55bc507.

## Nettoyage et préservation

Conserver la base SQLite locale, les licences, les tests de sécurité OBD, les ressources embarquées et docs/HARDWARE_VALIDATION.md. Les schémas sous src-tauri/gen/schemas/ sont générés par Tauri et déjà ignorés : les anciennes copies suivies ont été retirées pour éviter la dérive machine.

L'inventaire distingue fichiers suivis, non suivis, ignorés et worktrees. Les candidats
à suppression sont vérifiés par rôle et consommateurs ; aucune suppression massive
de tests, documentation, médias, dépendances, secrets ou données personnelles.
Les sauvegardes des fichiers retirés et les diffs initiaux sont hors des dépôts, dans
le dossier local `Documents/Codex/cleanup-2026-10-27/apps/BricarOBD`.
Une liste de fichiers ne constitue pas une revue comportementale de chaque ligne.

## Vérification et livraison

Commandes de référence : pnpm lint ; pnpm build ; pnpm test ; pnpm test:release-scripts ; cargo fmt --manifest-path src-tauri/Cargo.toml -- --check ; vérification Cargo avec le lockfile. Ne jamais utiliser un vrai véhicule pour valider un nettoyage.

Vérifié le 27 octobre : cargo fmt --manifest-path src-tauri/Cargo.toml -- --check et git diff --cached --check réussis. Les schémas générés retirés étaient déjà exclus par .gitignore ; ils sont régénérés par le build Tauri. La compilation native et les validations matérielles sont des preuves distinctes.

## Restant à traiter ou à confirmer

- La validation matérielle réelle doit suivre docs/HARDWARE_VALIDATION.md ; un test de démonstration ne valide pas les ECU physiques.
- Examiner les branches Dependabot et la cohérence package.json/pnpm-lock.yaml/Cargo.lock avant de les publier.
- auto-release.yml publie après un push main hors fichiers Markdown/workflows : les commits source restent locaux tant que cette activation n'est pas explicitement autorisée.

## Reprendre sans perdre le contexte

Lire ce fichier puis les consignes AGENTS.md/CLAUDE.md, README et les documents de domaine
encore présents. Contrôler `git status`, `git branch -avv`, `git worktree list` et les
refs distantes actuelles avant de reprendre. Garder les sources, la CI, le déploiement,
le runtime chargé et la validation utilisateur comme preuves distinctes.
Aucun redémarrage/rechargement de service, activation de release ou changement de base
n'est autorisé par le seul nettoyage Git.
