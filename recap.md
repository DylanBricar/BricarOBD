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

Au premier passage du 27 octobre : lint, build TypeScript/Vite, 541 tests frontend, 4 tests des scripts de livraison, cargo fmt et cargo check --manifest-path src-tauri/Cargo.toml --lib --locked réussis. Audit npm/pnpm : zéro avis. La comparaison des audits Rust trouvait 12 avis sur le main de départ, puis 3 après les premiers correctifs compatibles. La correction ZIP ci-dessous remplace cet ancien état d'audit en échec. Aucun profil matériel réel ni installateur publié n'est validé par ces seuls contrôles.

## Consolidation de ce passage

Sept branches Dependabot ont été intégrées et publiées sur main, puis retirées du serveur. Les versions production et développement sont conservées ; Vite 8.3.4 demandé par le lot développement remplace le minimum 8.2.4 de l'ancienne branche, et les types Node sont alignés sur Node 24. Les dépendances de tests restent en Vitest 4.1.12. Les verrous transitifs brace-expansion et Zod sont corrigés. Les mises à jour compatibles quinn-proto/rustls/webpki avaient corrigé neuf des douze avis Rust préexistants.

## Correction ZIP demandée après le nettoyage

- Le plugin stable 2.14.0 décrit sa feature `zip` comme une compatibilité avec les bundles Tauri v1. BricarOBD utilise son updater Tauri 2 depuis son introduction (`dd7b385`), le même endpoint latest.json, et les formats v2 actuels. Le manifeste public v2.0.26 a été récupéré et ses dix entrées reliées aux véritables noms d'assets : MSI/NSIS directs sous Windows, AppImage/DEB directs sous Linux, `.app.tar.gz` sous macOS. Aucun ZIP ni AppImage compressé n'est consommé.
- La dépendance conserve `rustls-tls` et `system-proxy`, mais désactive les features par défaut pour retirer uniquement ce format hérité inutilisé. Le code tar/gzip macOS est inconditionnel dans le plugin et reste présent. Les installateurs et signatures actuels ne changent pas. Aucun patch de plugin vendorizé, passage à une alpha Tauri 3 ou exception d'audit n'est nécessaire.
- `Cargo.lock` ne contient plus zip 4.6.1 ; l'audit Rust retourne zéro vulnérabilité connue. Les deux informations historiques restantes, `proc-macro-error` non maintenu et l'avis d'unsoundness GLib via GTK, restent visibles : aucun avertissement n'est masqué.
- `scripts/verify-updater-artifacts.mjs` vérifie les formats, la version, les signatures présentes, HTTPS et l'appartenance des URLs aux assets de la release, y compris les URLs API sans extension. Les deux workflows de publication l'exécutent avant de rendre une release publique. Onze cas nouveaux couvrent formats conservés, formats hérités refusés, signature/asset manquants, HTTP et version incohérente ; les quinze tests des scripts passent.
- `src-tauri/tests/updater_security.rs` utilise le vrai plugin dans le MockRuntime Tauri : signature publique de fixture valide, corruption refusée avant installation, charge signée sans format MSI/PE refusée sous Windows. Aucun installateur, serveur ou véhicule n'est démarré. Les trois tests passent sous Windows.
- Le premier lancement de cette fixture a aussi révélé l'absence de manifeste Common Controls v6 dans les exécutables de tests qui lient `TaskDialogIndirect`. `build.rs` ajoute ce manifeste uniquement aux cibles de tests Windows, avec détection de la cible de compilation. Aucun réglage Windows global ni binaire utilisateur n'est modifié.
- Validation locale de l'arbre corrigé : cargo fmt, Clippy strict toutes cibles/features et 420 tests Rust réussis (412 unitaires, 5 régressions OBD, 3 fixtures updater). Aucun test n'est désactivé ou ignoré. L'audit Rust conserve sa politique et retourne zéro vulnérabilité.
- La publication automatique de BricarOBD a été explicitement autorisée dans cette discussion. Les résultats de compilation et de release sur le SHA livré sont consignés dans `~/Documents/Codex/fix-followup-2026-10-27/BricarOBD/` et complétés après publication.

Références primaires : [features du plugin stable](https://docs.rs/tauri-plugin-updater/2.14.0/tauri_plugin_updater/#cargo-features), [formats d'update Tauri](https://v2.tauri.app/plugin/updater/#building), [arguments de lien des tests Cargo](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-arg-tests).

## Restant à traiter ou à confirmer

- La validation matérielle réelle doit suivre docs/HARDWARE_VALIDATION.md ; un test de démonstration ne valide pas les ECU physiques.
- Suivre les deux informations amont GLib/proc-macro-error ; elles sont distinctes des trois vulnérabilités ZIP retirées avec le format hérité inutilisé. Une future évolution du packaging doit rester compatible avec les formats vérifiés avant publication.
- Les branches distantes ne sont supprimées qu'après preuve de publication de ces historiques sur main.
- auto-release.yml publie après un push main hors fichiers Markdown/workflows. L'autorisation de publication de cette correction est acquise ; aucun redémarrage de service, opération sur véhicule ou installation réelle n'est inclus.

## Reprendre sans perdre le contexte

Lire ce fichier puis les consignes AGENTS.md/CLAUDE.md, README et les documents de domaine
encore présents. Contrôler `git status`, `git branch -avv`, `git worktree list` et les
refs distantes actuelles avant de reprendre. Garder les sources, la CI, le déploiement,
le runtime chargé et la validation utilisateur comme preuves distinctes.
Aucun redémarrage/rechargement de service, activation de release ou changement de base
n'est autorisé par le seul nettoyage Git.
