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
| `TableauDeBordEtudiant-Setup-X.Y.Z.exe.sig` | Signature de l'installateur (utilisée par la mise à jour automatique). |
| `latest.json` | Manifeste de mise à jour lu par l'application. |
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
crates/mcp/                 Serveur MCP local `tdb-mcp.exe` pour Hermes Agent (stdio, même base SQLite) — voir docs/HERMES_MCP.md
src-tauri/src/              Application Tauri : commandes IPC, fenêtre, coffre Windows, mise à jour, réglages MCP
```

Le frontend communique avec Rust **uniquement par commandes Tauri** (`invoke`) : aucun serveur HTTP local n'est ouvert. Le serveur MCP (`tdb-mcp.exe`, installé à côté de l'application) dialogue avec Hermes Agent sur l'entrée/sortie standard, sans port réseau.

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
| — | `verifier_mise_a_jour`, `installer_mise_a_jour`, `progression_mise_a_jour` (mise à jour) |
| — | `etat_mcp`, `definir_mcp`, `tester_mcp` (Intégrations IA — Hermes Agent / MCP) |

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
./scripts/preparer-serveur-mcp.ps1    # compile tdb-mcp et le place dans src-tauri/binaries
cargo tauri build --bundles nsis --config '{"bundle":{"createUpdaterArtifacts":false,"externalBin":["binaries/tdb-mcp"]}}'
```

Sorties : `target/release/TableauDeBordEtudiant.exe` (portable), `target/release/tdb-mcp.exe` et `target/release/bundle/nsis/*-setup.exe` (qui contient les deux). Le serveur MCP n'est déclaré (`externalBin`) qu'au moment de construire l'installateur : `cargo build`, `clippy` et `cargo test` n'en dépendent pas. `scripts/compiler-exe-windows.bat` fait tout cela en une fois.

Le même frontend fonctionne dans un navigateur avec le serveur Axum (voir le `README.md`).

### Intégration continue

`.github/workflows/build-windows.yml` (runner `windows-latest`) : build du frontend Leptos (`trunk build --release`) → `cargo fmt --check` → `clippy -D warnings` → `cargo test` → compilation du serveur MCP → `tauri build` → **test de lancement du `.exe` portable** (fenêtre créée, base SQLite créée) et **dialogue MCP réel avec `tdb-mcp.exe`** (`scripts/tester-serveur-mcp.ps1`) → **test de l'installateur** (installation silencieuse, raccourcis, `tdb-mcp.exe` présent et fonctionnel après réinstallation par-dessus, lancement, désinstallation, données conservées) → artifacts. Un tag stable `vX.Y.Z` signe en plus la mise à jour, génère `latest.json` et crée la GitHub Release (voir section 10).

## 9. Limitations connues

- Exécutables **non signés** (avertissement SmartScreen possible) . La mise à jour automatique n'existe qu'à partir de la 4.2.0 (installateur) ; le `.exe` portable ne se met pas à jour seul.
- Application Windows uniquement ; pour les autres systèmes, utilisez la version Web (serveur Axum, Docker).
- Les événements récurrents (`RRULE`) du flux iCal ne sont pas étendus — comme dans la version Python ; ADE exporte normalement chaque séance comme un événement distinct.
- Le modèle Gemini est librement saisissable : un nom inexistant donne un plan en mode simulation avec explication.

## 10. Mise à jour automatique

### Fonctionnement (utilisateur)

1. Quelques secondes après le lancement, l'application interroge en arrière-plan `https://github.com/Teamfliyou/Tablau_de_bord_projet/releases/latest/download/latest.json`. GitHub n'y expose que la **dernière release stable** : brouillons et préversions sont ignorés. Hors connexion, l'application continue normalement, sans message.
2. Si une version **strictement supérieure** existe, une fenêtre « Mise à jour disponible » affiche version actuelle, nouvelle version et améliorations. Boutons : **Mettre à jour maintenant** / **Plus tard**.
3. Le téléchargement affiche la progression réelle (octets reçus / taille annoncée). Tauri vérifie la **signature minisign** avec la clé publique intégrée ; si elle est invalide, rien n'est installé. L'installateur NSIS est alors lancé (mode passif) **par-dessus l'installation existante** (pas de seconde installation) et l'application redémarre.
4. **Configuration → Mises à jour** affiche la version installée, la dernière version publiée et sa date de publication. **Rechercher des mises à jour** déclenche une vérification manuelle (résultat ou erreur affichés) ; un échec de la vérification automatique au démarrage y est également signalé. En cas d'échec d'installation, un bouton **Réessayer** est proposé.
5. Le serveur MCP (`tdb-mcp.exe`) est inclus dans l'installateur et remplacé avec l'application ; les serveurs lancés par Hermes sont arrêtés juste avant l'installation pour que Windows ne verrouille pas le fichier.

Les données (`%APPDATA%\fr.teamfliyou.tableaudebord` : base SQLite, devoirs, préférences, liens ADE, sauvegardes) ne sont jamais touchées ; la clé Gemini reste dans le Gestionnaire d'identifiants Windows. L'identifiant `fr.teamfliyou.tableaudebord` ne doit jamais changer.

**Pourquoi une version n'arrive pas** : seule une **release publiée** est visible. Une version présente sur `main` mais sans tag (ce fut le cas de la 5.0.1), ou un tag dont le workflow a échoué (la 4.4.0), n'est jamais proposée. Vérifier dans *Releases* que la version y figure avec `latest.json`.

**Limites** : la 4.1.0 et les versions antérieures n'ont pas ce mécanisme : la 4.2.0 s'installe une première fois à la main. Le `.exe` **portable** peut détecter une nouvelle version mais refuse de l'installer (l'installateur créerait une seconde installation et on ne remplace pas un exécutable en cours d'exécution) : il affiche un message invitant à télécharger le nouvel installateur.

### Préparer les clés de signature (une seule fois, action humaine)

```bash
cargo tauri signer generate -w "$HOME/.tauri/tableau-de-bord.key"
```

Choisissez un mot de passe fort. La commande affiche la clé **publique** et écrit la clé privée dans le fichier indiqué (et `.key.pub`).

1. Collez la clé publique (contenu du fichier `.key.pub`) dans `src-tauri/tauri.conf.json` → `plugins.updater.pubkey` (déjà renseignée pour la 4.2.0 ; ne la changez que si la clé est remplacée, auquel cas les anciennes installations ne pourront plus se mettre à jour).
2. Dans GitHub : *Settings → Secrets and variables → Actions → New repository secret* :
   - `TAURI_SIGNING_PRIVATE_KEY` : **contenu** du fichier de clé privée ;
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` : le mot de passe choisi.
3. Sauvegardez la clé privée et son mot de passe dans un gestionnaire de mots de passe. Elle ne doit **jamais** être commitée (le workflow ne l'écrit ni sur disque ni dans les journaux). Si elle est perdue, plus aucune mise à jour ne pourra être publiée pour les installations existantes.

### Publier une nouvelle version

1. Mettre à jour la version **partout** : `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` et les `Cargo.toml` des crates, dont `crates/mcp` (puis `cargo update -w --offline` pour régénérer `Cargo.lock`).
2. Fusionner dans `main` quand la CI est verte, puis créer le tag **sur ce commit de `main`** (la publication n'a lieu qu'à ce moment-là : un simple push sur `main` ne publie rien) :
   ```bash
   git tag v5.1.0 && git push origin v5.1.0
   ```
3. Le workflow (tag stable `vX.Y.Z` uniquement) : vérifie tag/version (application et serveur MCP)/secrets → fmt, clippy, tests → compile frontend + NSIS signé → **vérifie la signature avec la clé publique intégrée** → génère et valide `latest.json` → teste lancement et installation (y compris réinstallation par-dessus) → crée la release en **brouillon**, téléverse tout, contrôle que chaque fichier est présent, **puis seulement** la publie. Toute signature, installateur ou fichier manquant fait échouer le workflow : `latest.json` n'est donc jamais visible avant ses artefacts.

### Sécurité de la mise à jour

HTTPS uniquement (GitHub) ; signature obligatoire ; clé privée dans les seuls secrets GitHub ; aucune version identique, antérieure ou préversion n'est proposée (`doit_proposer` dans `src-tauri/src/maj.rs`, testé) ; les commandes sont des commandes Rust dédiées et le plugin n'expose aucune permission supplémentaire à l'interface.

### Tests

- Automatisés : `cargo test --workspace` (comparaison de versions, détection portable/installé, validité de la clé publique et de l'endpoint, identifiant de données) ; en CI sur tag : vérification de la signature réelle de l'installateur, validation de `latest.json`, réinstallation par-dessus sans doublon et données conservées.
- **À faire manuellement avant d'annoncer le mécanisme** (non automatisé) : installer la 4.2.0, publier un tag `v4.2.1`, lancer l'application installée et valider la mise à jour de bout en bout ; tester aussi hors connexion et avec une base contenant des données.
