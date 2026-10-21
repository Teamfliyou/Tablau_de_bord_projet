# Migration complète vers Rust

Ce document décrit l'état du dépôt **avant** la migration, l'architecture **cible**, les choix techniques (et leurs écarts avec la demande initiale), les risques et les étapes réalisées.

L'ancien code (Python / FastAPI / Vue.js) est conservé, intact, sur la branche **`legacy/python-vue`** pour comparaison et retour arrière. `main` ne contient plus que le code Rust.

---

## 1. Inventaire de l'existant (audit du dépôt)

Le dépôt contenait **deux applications parallèles** :

| | Version Web (historique) | Application Windows |
|---|---|---|
| Backend | Python 3.11, FastAPI, SQLAlchemy, Alembic, passlib/bcrypt, PyJWT | **Rust** : crate `tdb-core` (rusqlite, reqwest, ical, chrono-tz) |
| Frontend | `frontend/index.html` (Vue 3 + Tailwind via CDN, 1 738 lignes) | `src/App.vue` (Vue 3 + Vite + Tailwind, 1 564 lignes) |
| Coque | navigateur | Tauri 2 (commandes IPC, aucun port réseau) |
| Compte | multi-utilisateurs (e-mail + mot de passe, JWT) | mono-utilisateur local |
| Données | SQLite `backend/app.db` | SQLite `%APPDATA%\…\tableau-de-bord.db` |
| Secrets | clé Gemini en clair dans la base | Gestionnaire d'identifiants Windows |
| Tests | 5 fichiers pytest | 30 tests Rust dans `tdb-core` |
| CI | `ci.yml` (pytest, alembic, uvicorn) | `build-windows.yml` (npm + Tauri → NSIS) |
| Déploiement | `Dockerfile` Python, `docker-compose.yml`, `Procfile`, `install.sh`, `lancer_app.sh` | — |

### Routes FastAPI à préserver

| Méthode | Route | Rôle |
|---|---|---|
| GET | `/api/health` | santé |
| POST | `/api/auth/register`, `/api/auth/login` | création de compte / connexion (JWT HS256, bcrypt) |
| GET | `/api/auth/me` | profil |
| GET / POST | `/api/config` | pseudo, lien ADE, clé Gemini (jamais relue) |
| GET | `/api/config/categories` | groupes / types / matières détectés dans le flux ADE |
| GET | `/api/cours-du-jour`, `/api/cours-semaine` | emploi du temps (+ filtres `groupes`, `types`, `matieres`) |
| GET / POST | `/api/devoirs` | liste / création |
| PATCH / DELETE | `/api/devoirs/{id}` | statut / suppression |
| GET / POST | `/api/devoirs/export`, `/api/devoirs/import` | sauvegarde JSON (500 devoirs max) |
| POST | `/api/plan-revision` | plan de révision (Gemini ou simulation) |

### Fonctionnalités de l'interface (identiques dans les deux frontends Vue)

Dashboard (statistiques, emploi du temps du jour avec badge « EN COURS » et actualisation 60 s, devoirs rapides, plan IA avec copie), Semaine (navigation, jours, « aujourd'hui »), Kanban (types, statuts, échéances J-X, édition, export / import), Configuration (ADE, Gemini, modèle, apparence dock/sidebar, thème clair/sombre/auto, filtres), assistant de première configuration, mode hors ligne, notifications.

### Intégrations

* **ADE / iCal** : téléchargement HTTPS, analyse des événements (matière, groupes TP/TD/Khôlles, types DS/IE/Exam, enseignants), cache, anti-SSRF.
* **Gemini** : prompt, réponse JSON stricte, repli en *simulation* sans clé ou en cas d'erreur.

---

## 2. Architecture cible

```
/
├── Cargo.toml                 workspace
├── crates/
│   ├── shared/   (tdb-shared)    types + règles de validation, compilables en WASM
│   ├── core/     (tdb-core)      logique métier : ADE, Gemini, SQLite mono-utilisateur, secrets
│   ├── server/   (tdb-server)    serveur Web Axum : API REST, auth multi-utilisateurs, statique
│   └── frontend/ (tdb-frontend)  interface Leptos (Rust → WebAssembly), Tailwind, Trunk
├── src-tauri/    (application Windows Tauri 2, appelle tdb-core via des commandes IPC)
├── .github/workflows/            ci.yml (Web) et build-windows.yml (.exe + NSIS)
├── Dockerfile · docker-compose.yml
└── MIGRATION_RUST.md · README.md · docs/
```

* **Un seul frontend** pour le Web et Windows. `crates/frontend/src/api.rs` détecte `window.__TAURI_INTERNALS__` : dans Tauri il appelle les commandes IPC (aucun port réseau), dans le navigateur il appelle l'API HTTP d'Axum. Aucun JavaScript écrit à la main : les liaisons passent par `wasm-bindgen` / `web-sys`.
* **Logique partagée** : `tdb-shared` définit les contrats (devoirs, cours, plan…) et leurs validations ; `tdb-core` fournit l'analyse ADE, l'anti-SSRF et le planificateur Gemini, réutilisés tels quels par le serveur Axum.
* **Sécurité** par conception : voir §4.

### Écarts assumés avec la demande initiale

| Demande | Réalisé | Justification |
|---|---|---|
| SQLx pour SQLite | **rusqlite** (SQLite embarqué) | Le cœur Rust existant (837 lignes, 30 tests : migrations, sauvegardes, restauration après corruption) repose sur rusqlite et fonctionne. Le réécrire avec SQLx n'apporte aucune fonctionnalité et ajoute du risque ; rusqlite évite aussi le runtime asynchrone côté Tauri. Le passage à SQLx reste possible sans toucher à l'API publique de `Store`/`Db`. |
| Structure `api/ ade/ ai/ database/ desktop/` | `shared/ core/ server/ frontend/` + `src-tauri/` | ADE, IA et base partagent le même cœur et ses tests ; les séparer n'apportait que des dépendances croisées. `src-tauri/` garde l'emplacement attendu par la CLI Tauri. |
| `migrations/` à la racine | migrations versionnées dans le code (`PRAGMA user_version`) | Les migrations sont transactionnelles et précédées d'une sauvegarde `VACUUM INTO`, ce que ne permet pas un dossier de fichiers SQL seul. |
| Tailwind | utilisé via la compilation Trunk | Tailwind (CLI autonome, binaire, sans Node.js) génère le CSS ; seul le résultat CSS est livré. |

---

## 3. Données existantes

Aucune donnée n'est jamais supprimée automatiquement.

* **Version Web** : une base `app.db` de l'ancienne version Python est ouverte directement. Au premier démarrage, `tdb-server` détecte le schéma historique, crée une sauvegarde `sauvegardes/avant-migration-rust-<date>.db`, puis complète le schéma de façon additive (colonnes `gemini_model`, `preferences`, index) et passe `user_version` à 2. Les comptes, les mots de passe bcrypt (`$2b$`), les devoirs, les liens ADE et les clés Gemini sont conservés. Les clés Gemini en clair sont lues telles quelles ; elles sont chiffrées au prochain enregistrement. *Test : `migration_d_une_base_python_existante`.*
* **Application Windows** : inchangée (schéma, sauvegardes quotidiennes, import `app.db` depuis *Configuration → Données locales*).
* **JWT** : si `SECRET_KEY` est identique, les anciennes sessions restent valides (même algorithme et mêmes revendications).

---

## 4. Sécurité

* Mots de passe : bcrypt (coût configurable), vérification sur un thread dédié, hachage factice si l'e-mail est inconnu ; limitation des échecs de connexion (10 / 15 min par e-mail → HTTP 429).
* Clés Gemini : jamais renvoyées par l'API ; chiffrées au repos (AES-256-GCM, clé dérivée du secret serveur) ; transmises à Gemini dans l'en-tête `x-goog-api-key`, jamais dans une URL ni un journal.
* Secret serveur : `SECRET_KEY`, ou secret aléatoire créé une fois à côté de la base si absent (les sessions survivent aux redémarrages).
* ADE : HTTPS obligatoire, résolution DNS vérifiée (aucune adresse privée, locale, CGNAT, métadonnées cloud), adresses imposées au client HTTP (anti DNS-rebinding), pas de redirection, taille et durée limitées, liste blanche optionnelle `ADE_ALLOWED_HOSTS`.
* Requêtes SQL toutes paramétrées ; isolation des données par utilisateur testée.
* CORS : même origine par défaut ; origines explicites via `ALLOWED_ORIGINS`.
* En-têtes : CSP stricte (hachage SHA-256 des scripts en ligne générés par Trunk, `wasm-unsafe-eval`), `X-Frame-Options`, `nosniff`, `Referrer-Policy`, `Cache-Control: no-store` sur l'API. CSP Tauri : `script-src 'self' 'wasm-unsafe-eval'`, pas de `connect-src` vers l'extérieur.
* Pas de `unwrap()` / `expect()` dans les chemins applicatifs (erreurs typées, messages sans détails internes).

**Changement de contrat** : les mots de passe sont limités à 72 octets (limite de bcrypt) au lieu de 128 caractères, pour éviter une troncature silencieuse.

---

## 5. Risques et limites

| Risque | Mesure |
|---|---|
| Jeton de session conservé dans `localStorage` (exposé en cas de XSS) | CSP stricte, aucun HTML injecté, aucune dépendance JS tierce. Un cookie `HttpOnly` est une évolution possible. |
| Compilation Windows non vérifiable hors Windows/MSVC | Workflow GitHub Actions dédié ; l'état réel des compilations locales est consigné dans le rapport de migration. |
| Parité visuelle Vue → Leptos | Mêmes classes Tailwind et même structure DOM ; vérification visuelle recommandée après le premier build. |
| Tailwind sans Node.js | CLI autonome téléchargée par Trunk ; seule la sortie CSS est utilisée à l'exécution. |

---

## 6. Étapes réalisées

1. Audit et branche `legacy/python-vue` (sauvegarde de l'ancien code).
2. Workspace Cargo ; extraction des types dans `tdb-shared` ; `tdb-core` déplacé dans `crates/core` (30 tests inchangés).
3. Serveur Axum `tdb-server` : toutes les routes FastAPI, auth, config, ADE, devoirs, import / export, plan IA, préférences, migration de la base Python (17 tests).
4. Frontend Leptos : Dashboard, Semaine, Kanban, Configuration, onboarding, connexion, thème, filtres, hors ligne.
5. Tauri 2 branché sur le frontend Leptos (CSP, `trunk build`).
6. Workflows GitHub Actions, Dockerfile, docker-compose.
7. Nettoyage : suppression de Python, Vue, npm, Alembic, scripts shell obsolètes.
