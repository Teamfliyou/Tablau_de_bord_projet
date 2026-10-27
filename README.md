# 🎓 Tableau de bord étudiant

Dashboard personnel pour visualiser son emploi du temps en temps réel (flux iCal **ADE Campus**), gérer ses devoirs dans un tableau **Kanban** et obtenir un **plan de révision intelligent** généré par IA (Google **Gemini**).

Le projet est **100 % Rust** : un même frontend (Leptos, compilé en WebAssembly) sert la **version Web** (serveur Axum) et l'**application Windows** (Tauri 2).

![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust&logoColor=white)
![Axum](https://img.shields.io/badge/Axum-0.8-blue)
![Leptos](https://img.shields.io/badge/Leptos-0.8-ef3939)
![Tauri](https://img.shields.io/badge/Tauri-2-24c8db?logo=tauri&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-003B57?logo=sqlite&logoColor=white)
![Licence](https://img.shields.io/badge/Licence-MIT-blue)

> L'ancienne version Python / FastAPI / Vue.js est conservée sur la branche [`legacy/python-vue`](../../tree/legacy/python-vue). Le détail de la migration est dans [MIGRATION_RUST.md](MIGRATION_RUST.md).

## ✨ Fonctionnalités

| | |
|---|---|
| 📅 **Synchronisation ADE** | Emploi du temps du jour et de la semaine depuis le flux iCal, actualisation toutes les 60 s, badge « EN COURS », navigation de semaine en semaine, mode hors ligne. |
| 🏷️ **Filtres** | Détection automatique des groupes (TP/TD/Khôlles), types (DS, IE, Exam) et matières. |
| 📋 **Devoirs (Kanban)** | Ajout, modification, statuts, échéances (J-X, retard), sous-tâches (to-do list) avec progression, tags, export / import JSON. |
| 🤖 **Plan de révision IA** | Gemini avec réponse JSON stricte ; mode simulation sans clé ou en cas d'erreur. |
| ⚙️ **Configuration** | Lien ADE, clé Gemini, modèle, dock ou barre latérale, thème clair / sombre / auto. |
| 🔐 **Comptes (Web)** | Inscription / connexion, données isolées par utilisateur. |

## 🏗️ Architecture

```
crates/
  shared/     types et validations partagés (compilables en WebAssembly)
  core/       logique métier : ADE (anti-SSRF), Gemini, SQLite, secrets
  server/     serveur Web Axum : API REST + authentification + fichiers statiques
  frontend/   interface Leptos (Rust → WASM) + Tailwind, compilée par Trunk
src-tauri/    application Windows (Tauri 2) : commandes IPC vers crates/core
```

* **Web** : navigateur → Axum (`/api/*`) → SQLite multi-utilisateurs.
* **Windows** : fenêtre Tauri → commandes IPC → `tdb-core` → SQLite local. Aucun port réseau n'est ouvert.

## 🌐 Version Web

### Docker (recommandé)

```bash
cp .env.example .env        # puis renseignez SECRET_KEY (openssl rand -base64 48)
docker compose up --build -d
```

Ouvrez <http://localhost:8000>. La base SQLite est conservée dans le volume Docker `app-data` ; la santé est vérifiée sur `/api/health`. L'image finale ne contient que le binaire Rust, le frontend WASM et `curl` (ni Python ni Node.js).

### Sans Docker (développement)

Prérequis : [Rust](https://rustup.rs) stable, la cible WebAssembly et [Trunk](https://trunkrs.dev).

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked

(cd crates/frontend && trunk build --release)   # compile l'interface dans crates/frontend/dist
cargo run -p tdb-server --release               # http://localhost:8000
```

### Variables d'environnement

Voir [.env.example](.env.example) : `SECRET_KEY`, `PORT` / `BIND_ADDR`, `DATABASE_PATH`, `FRONTEND_DIR`, `ALLOWED_ORIGINS` (CORS, vide = même origine), `ADE_ALLOWED_HOSTS`, `GEMINI_API_KEY`, `JWT_EXPIRATION_MINUTES`, `BCRYPT_COST`, `ICS_CACHE_TTL_SECONDS`, `RUST_LOG`.

### Migrer une ancienne base Python

Pointez `DATABASE_PATH` (ou l'ancienne `DATABASE_URL=sqlite:///…`) vers votre ancien `app.db`. Au démarrage, une sauvegarde est créée dans `sauvegardes/`, puis le schéma est complété sans rien supprimer. Définissez le même `SECRET_KEY` pour conserver les sessions ouvertes.

### Sous-tâches (to-do) et import / export JSON

Chaque carte du Kanban peut contenir une to-do list (*crayon → Sous-tâches* : ajouter, modifier, cocher, supprimer, monter / descendre). La carte affiche la progression (`2/5` + barre). Le fichier d'import / export est une liste de devoirs ; le champ `todos` est facultatif (un fichier sans `todos` s'importe comme avant) et `fait` vaut `false` par défaut. Limites : 100 sous-tâches par devoir, 200 caractères chacune. L'export n'écrit `todos` que pour les devoirs qui en ont, et se réimporte à l'identique.

```json
[
  {
    "titre": "TD Maths",
    "matiere": "Mathématiques",
    "echeance": "2026-11-10",
    "type": "devoir",
    "statut": "en_cours",
    "tags": [{ "nom": "TD", "couleur": "#6366f1" }],
    "todos": [
      { "texte": "Exercice 1", "fait": true },
      { "texte": "Exercice 2", "fait": false }
    ]
  }
]
```

Exemple prêt à importer : [docs/exemple-import.json](docs/exemple-import.json).

## 🖥️ Application Windows

Téléchargez l'installateur (`TableauDeBordEtudiant-Setup-<version>.exe`) ou le `.exe` portable dans **Actions → Build Windows (Tauri) → Artifacts**, ou dans **Releases** pour chaque tag `vX.Y.Z`. Détails : [docs/APPLICATION_WINDOWS.md](docs/APPLICATION_WINDOWS.md).

### Générer le `.exe` soi-même

Prérequis : Rust stable (cible MSVC) avec *Build Tools for Visual Studio*, la cible WebAssembly, Trunk et la CLI Tauri.

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk tauri-cli --locked
./scripts/preparer-serveur-mcp.ps1
cargo tauri build --bundles nsis --config '{"bundle":{"createUpdaterArtifacts":false,"externalBin":["binaries/tdb-mcp"]}}'
```

Sorties : `target/release/TableauDeBordEtudiant.exe`, `target/release/tdb-mcp.exe` et `target/release/bundle/nsis/*-setup.exe`. (Le paramètre `--config` désactive les fichiers de mise à jour signés, réservés à la CI qui détient la clé privée.)

Le workflow GitHub Actions peut aussi être lancé manuellement (*Actions → Build Windows (Tauri) → Run workflow*). Une Release n'est publiée que pour un tag stable `vX.Y.Z` :

```bash
git tag v5.1.0 && git push origin v5.1.0
```

### 🤖 Hermes Agent (MCP, depuis la 5.1.0)

L'installateur fournit `tdb-mcp.exe`, un serveur MCP local (stdio, sans port réseau) qui permet à [Hermes Agent](https://github.com/NousResearch/hermes-agent) de consulter tes devoirs : tâches du jour, retards, échéances, avancement par matière. Il est désactivé et en lecture seule par défaut, et les modifications passent par une confirmation. À activer dans **Configuration → Intégrations IA**, qui fournit la configuration Hermes à copier et un test de connexion. Guide : [docs/HERMES_MCP.md](docs/HERMES_MCP.md).

### 🔄 Mise à jour automatique (depuis la 4.2.0)

L'installation (NSIS) vérifie discrètement au lancement s'il existe une release stable plus récente sur GitHub, propose de la télécharger avec une barre de progression réelle, vérifie sa signature cryptographique puis l'installe sans toucher à vos données. Vous pouvez aussi la déclencher dans **Configuration → Mises à jour** (version installée, dernière version publiée et date). La 4.2.0 doit être installée une première fois manuellement ; les installations 4.1.0 et antérieures ne se mettent pas à jour seules. Préparation des clés et publication : voir [docs/APPLICATION_WINDOWS.md](docs/APPLICATION_WINDOWS.md#10-mise-à-jour-automatique).

## ✅ Tests et qualité

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo clippy -p tdb-frontend --target wasm32-unknown-unknown -- -D warnings
```

## 🔒 Sécurité

Mots de passe bcrypt + limitation des tentatives, sessions JWT, clés Gemini chiffrées au repos (Web) ou dans le Gestionnaire d'identifiants Windows, jamais journalisées ni renvoyées, anti-SSRF sur le flux ADE (HTTPS, adresses publiques uniquement, pas de redirection), requêtes SQL paramétrées, CORS restrictif, CSP stricte. Voir [MIGRATION_RUST.md](MIGRATION_RUST.md#4-sécurité).

## 📄 Licence

MIT
