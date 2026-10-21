# 🖥️ Tableau de bord étudiant — application Windows (Rust + Tauri)

Depuis la version 4.0, le tableau de bord existe sous forme d'**application de bureau Windows autonome** :
un seul fichier `.exe`, sans Python, sans Docker, sans Node.js, sans serveur et sans navigateur.

Depuis la migration complète vers Rust, l'application de bureau et la version Web partagent le même frontend (Leptos, Rust/WebAssembly) et les mêmes types ; l'ancien code Python/Vue est conservé sur la branche `legacy/python-vue`.

## 1. Télécharger l'application

Chaque compilation de la CI publie deux fichiers dans l'onglet **Actions → « Build Windows (Tauri) » → Artifacts** (`TableauDeBordEtudiant-windows`), et dans **Releases** pour chaque tag `vX.Y.Z` :

| Fichier | Usage |
|---|---|
| `TableauDeBordEtudiant.exe` | **Version portable** : double-clic, aucune installation. |
| `TableauDeBordEtudiant-Setup-X.Y.Z.exe` | **Installateur NSIS** : raccourcis bureau + menu Démarrer, désinstallation propre. |
| `SHA256SUMS.txt` | Sommes de contrôle pour vérifier les téléchargements. |

> Windows SmartScreen peut afficher « Windows a protégé votre ordinateur » : les exécutables ne sont pas signés par un certificat de signature de code payant. Cliquez sur **Informations complémentaires → Exécuter quand même**.

### Dépendance système : WebView2

L'interface utilise **Microsoft Edge WebView2**, déjà présent sur Windows 11 et sur les Windows 10 à jour.

- **Installateur** : si WebView2 est absent, l'installateur le télécharge et l'installe silencieusement (`webviewInstallMode: downloadBootstrapper`).
- **Version portable** : si WebView2 est absent, installez le « Runtime Evergreen » depuis <https://developer.microsoft.com/microsoft-edge/webview2/>.

Aucune autre dépendance : ni Rust, ni Python, ni Node.js, ni Docker sur le PC de l'utilisateur final.

## 2. Premier lancement

1. Saisissez votre **lien iCal ADE** (HTTPS) dans l'assistant de bienvenue (ou plus tard dans *Configuration*).
2. *(Optionnel)* Saisissez votre **clé Google Gemini** : sans clé, le plan de révision fonctionne en **mode simulation**.
3. Dans *Configuration → Filtres*, cliquez sur **Analyser mon lien ADE** pour détecter vos groupes, matières et types de cours.

Il n'y a plus de compte ni de mot de passe : l'application est mono-utilisateur et locale.

## 3. Où sont mes données ?

| Donnée | Emplacement |
|---|---|
| Base SQLite (devoirs, réglages, dernier emploi du temps) | `%APPDATA%\fr.teamfliyou.tableaudebord\tableau-de-bord.db` |
| Sauvegardes automatiques (7 dernières, une par jour) | `%APPDATA%\fr.teamfliyou.tableaudebord\sauvegardes\` |
| Journaux (sans aucun secret) | `%LOCALAPPDATA%\fr.teamfliyou.tableaudebord\logs\` |
| Clé Gemini | **Gestionnaire d'identifiants Windows** (service `TableauDeBordEtudiant`) |

Les données sont **identiques pour la version portable et l'installée**, survivent aux redémarrages et aux mises à jour, et **ne sont pas supprimées à la désinstallation** (l'option « supprimer les données de l'application » du désinstalleur est décochée par défaut). Pour tout effacer : supprimez le dossier `%APPDATA%\fr.teamfliyou.tableaudebord` et l'entrée `TableauDeBordEtudiant` du Gestionnaire d'identifiants.

### Fiabilité de la base

- Schéma **versionné** (`PRAGMA user_version`) : les migrations sont appliquées automatiquement et transactionnellement ; une sauvegarde `avant-migration-vN` est faite avant toute montée de version.
- Mode WAL, `integrity_check` au démarrage : une base illisible est **mise de côté** (`corrompue-AAAAMMJJ-HHMMSS.db`, jamais supprimée) puis restaurée depuis la dernière sauvegarde.
- Une seule instance de l'application peut tourner à la fois (la seconde ramène la première au premier plan).

## 4. Migrer mes données depuis l'ancienne version

| Source | Méthode |
|---|---|
| Fichier JSON exporté de l'ancienne version | *Kanban → Importer* (même format, 500 devoirs / 2 Mo maximum, validation complète avant toute écriture). |
| Ancienne base SQLite `backend/app.db` | *Configuration → Données locales → Importer l'ancienne base (app.db)*. |

L'import de l'ancienne base :

- ouvre le fichier en **lecture seule** (jamais modifié) ;
- ajoute les devoirs sans créer de doublons (relançable sans risque) ;
- reprend le pseudo et le lien ADE du premier compte si les champs sont vides ;
- transfère l'éventuelle clé Gemini dans le Gestionnaire d'identifiants Windows (elle n'est jamais écrite dans la nouvelle base) ;
- fait une sauvegarde de la nouvelle base juste avant.

Les comptes e-mail / mots de passe de l'ancienne version ne sont pas repris (plus nécessaires).

## 5. Fonctionnalités

Toutes les fonctionnalités de la version Python sont reprises :

- **Emploi du temps ADE** : cours du jour et de la semaine (lundi → vendredi), navigation entre semaines, cours en cours / à venir / terminés, reconnaissance matières / groupes TD-TP / khôlles, filtres enregistrés, actualisation automatique toutes les 60 s, fuseau **Europe/Paris**, cache mémoire (5 min) et **dernier emploi du temps conservé pour la consultation hors ligne** (bandeau « Mode hors ligne »).
- **Kanban** : ajout, **modification**, suppression, statuts (à faire / en cours / terminé), échéances, retards et échéances proches, sauvegarde automatique, export / import JSON via les boîtes de dialogue Windows.
- **Plan de révision IA** (Gemini) : cours du jour + devoirs en attente → planning structuré, régénérable. Appel exécuté **côté Rust**. En cas d'erreur réseau/API (clé refusée, modèle introuvable, quota, service indisponible, délai dépassé, réponse invalide), un plan en mode simulation est affiché avec un **message d'explication**.
- **Paramètres** : pseudo, lien ADE, clé Gemini, modèle Gemini, style de navigation (*Modern Dock* / *Classic Sidebar*), **thème clair / sombre / automatique**, filtres.

> Le modèle Gemini par défaut reste `gemini-3.8-flash` (valeur de la version Python) ; modifiable dans *Configuration*.

## 6. Architecture

```
crates/frontend/            Frontend Leptos (Rust → WebAssembly) + Tailwind CSS, compilé par Trunk et embarqué dans l'exe
crates/shared/              Types et validations partagés (compilables en WASM)
crates/core/                Bibliothèque Rust `tdb-core` (sans dépendance à Tauri, testée seule)
  ├─ db.rs                  SQLite : migrations, sauvegardes, devoirs, réglages, import ancienne base
  ├─ ade.rs                 Validation d'URL anti-SSRF, téléchargement, parseur iCalendar, filtres
  ├─ planner.rs             Plan de révision : simulation locale + client Gemini
  ├─ service.rs             Orchestration, cache ICS (mémoire + SQLite), configuration
  └─ secrets.rs             Interface du stockage de secrets
crates/server/              Serveur Web Axum (version Web, non utilisé par l'application Windows)
src-tauri/src/              Application Tauri : commandes IPC, fenêtre, coffre Windows
```

Le frontend communique avec Rust **uniquement par commandes Tauri** (`invoke`) : aucun serveur HTTP local n'est ouvert.

| Ancienne route FastAPI | Commande Tauri |
|---|---|
| `GET /api/cours-du-jour` | `cours_du_jour` |
| `GET /api/cours-semaine` | `cours_semaine` |
| `GET /api/config/categories` | `analyser_categories` |
| `GET/POST /api/config` | `obtenir_config` / `enregistrer_config` |
| `GET/POST/PATCH/DELETE /api/devoirs` | `lister_devoirs` / `creer_devoir` / `modifier_devoir` / `supprimer_devoir` |
| `GET /api/devoirs/export`, `POST /api/devoirs/import` | `exporter_devoirs`, `importer_devoirs` (dialogues natifs côté Rust) |
| `POST /api/plan-revision` | `generer_plan` |
| `/api/auth/*` | supprimé (application mono-utilisateur) |
| — | `obtenir_preferences`, `enregistrer_preferences`, `migrer_ancienne_base`, `version_application` |

Bibliothèques : `tauri` 2, `rusqlite` (SQLite embarqué), `serde`, `tokio`, `reqwest` (TLS natif Windows), `ical` (parseur iCalendar), `chrono` + `chrono-tz`, `keyring` (Gestionnaire d'identifiants Windows).

## 7. Sécurité

- **Clé Gemini** : stockée uniquement dans le Gestionnaire d'identifiants Windows ; jamais renvoyée au frontend, jamais dans la base, les fichiers distribués ni les journaux ; transmise à Google dans l'en-tête `x-goog-api-key` (pas dans l'URL).
- **Anti-SSRF** pour le flux ADE : HTTPS obligatoire, pas d'identifiants dans l'URL, hôtes locaux/internes refusés, résolution DNS dont **toutes** les adresses doivent être publiques (IPv4 privées, loopback, link-local, CGNAT, IPv6 ULA / link-local / IPv4-mappées refusées), adresses ensuite **imposées** au client HTTP (anti DNS-rebinding), **redirections refusées**, délais (20 s) et taille maximale (10 Mo).
- **Fichiers importés** : sélectionnés côté Rust, 2 Mo maximum, 500 devoirs maximum, validation complète puis insertion dans une transaction unique.
- **Permissions Tauri minimales** : une seule capacité (`core:default`) ; l'interface n'a accès ni au disque, ni au réseau, ni au shell. Content-Security-Policy stricte (scripts du bundle uniquement).
- Journalisation des erreurs sans secrets (messages génériques pour le coffre et Gemini).
- Aucune opération bloquante sur l'interface : réseau et commandes s'exécutent hors du thread de l'interface.

## 8. Compiler soi-même

Prérequis (développement uniquement) : [Rust](https://rustup.rs) stable + *Build Tools for Visual Studio* (charge « Développement Desktop en C++ »), la cible WebAssembly, [Trunk](https://trunkrs.dev) et la CLI Tauri. Ni Node.js ni Python ne sont nécessaires.

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk tauri-cli --locked
cargo tauri dev                        # mode développement (fenêtre native)
cargo test --workspace                 # tests Rust
cargo tauri build --bundles nsis       # exe + installateur
```

Sorties : `target/release/TableauDeBordEtudiant.exe` (portable) et `target/release/bundle/nsis/*-setup.exe`.

Le même frontend fonctionne dans un navigateur avec le serveur Axum (voir le `README.md`).

### Intégration continue

`.github/workflows/build-windows.yml` (runner `windows-latest`) : build du frontend Leptos (`trunk build --release`) → `cargo fmt --check` → `clippy -D warnings` → `cargo test` → `tauri build` → **test de lancement du `.exe` portable** (fenêtre créée, base SQLite créée) → **test de l'installateur** (installation silencieuse, raccourcis, lancement, désinstallation, données conservées) → artifacts. Un tag `vX.Y.Z` crée en plus une GitHub Release.

```bash
git tag v4.0.0 && git push origin v4.0.0
```

## 9. Limitations connues

- Exécutables **non signés** (avertissement SmartScreen possible) ; pas de mise à jour automatique intégrée (réinstallez la nouvelle version par-dessus : les données sont conservées).
- Application Windows uniquement ; pour les autres systèmes, utilisez la version Web (serveur Axum, Docker).
- Les événements récurrents (`RRULE`) du flux iCal ne sont pas étendus — comme dans la version Python ; ADE exporte normalement chaque séance comme un événement distinct.
- Le modèle Gemini est librement saisissable : un nom inexistant donne un plan en mode simulation avec explication.
