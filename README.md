# 🎓 Tableau de bord étudiant

Dashboard personnel **Full-Stack** pour visualiser son emploi du temps en temps réel 📅 (flux iCal ADE Campus), gérer ses devoirs 📝 et générer un **plan de révision intelligent** 🧠 grâce à une IA (Google Gemini).

> Stack : **FastAPI** (backend Python) + **Vue.js 3 / Tailwind CSS** (frontend) + **SQLite** (stockage local) + **Gemini** (génération des plans).

---

## Badges

![FastAPI](https://img.shields.io/badge/FastAPI-0.110-009688?logo=fastapi&logoColor=white)
![Vue.js](https://img.shields.io/badge/Vue.js-3-42b883?logo=vue.js&logoColor=white)
![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-3.4-38bdf8?logo=tailwindcss&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-003B57?logo=sqlite&logoColor=white)
![Gemini](https://img.shields.io/badge/Gemini-1.5_Flash-8E75B2?logo=google&logoColor=white)
![Python](https://img.shields.io/badge/Python-3.10+-3776AB?logo=python&logoColor=white)
![Licence](https://img.shields.io/badge/Licence-MIT-blue)

---

## Fonctionnalités clés

| Fonctionnalité | Description |
|---|---|
| 🔄 **Synchronisation ADE live** | Emploi du temps du jour synchronisé avec le flux iCal de l'école (ADE Campus), auto-actualisation toutes les 60 s, badge "EN COURS" sur le créneau en cours et bouton de rafraîchissement manuel. |
| 📝 **Gestion de devoirs** | Ajout, modification (marquer comme fait), suppression. Persistance **SQLite**. Badges d'échéance intelligents (aujourd'hui, demain, en retard, dans X jours). |
| 🤖 **Plan de révision par IA** | Un clic → Gemini analyse les cours du jour + les devoirs et renvoie un plan de soirée **JSON strict** : conseil du soir + sessions structurées (matière, durée, objectif, technique). |
| 🎨 **Design Bento Box** | Interface moderne : fond `slate-50`, cartes blanches `rounded-2xl`, ombres douces, accents indigo/violet, carte IA sombre et futuriste en dégradé. |

---

## Architecture du projet

```
Tablau_de_bord_projet/
├── backend/
│   ├── main.py            # API FastAPI (routes + logique iCal + IA)
│   ├── database.py        # Connexion SQLite + modèle Devoir (SQLModel)
│   └── requirements.txt   # Dépendances Python
├── frontend/
│   └── index.html         # SPA Vue.js 3 + Tailwind (via CDN)
├── .env.example           # Modèle des variables d'environnement
├── .gitignore             # Fichiers exclus de Git (secrets, DB, caches)
└── README.md
```

**Déroulement d'une requête :**

```
Frontend (Vue) ──fetch──▶ FastAPI ──┬─▶ ADE Campus (téléchargement .ics, parsing)
                                    ├─▶ SQLite (devoirs, via SQLModel)
                                    └─▶ Google Gemini (prompt → plan JSON)
```

---

## Prérequis

- **Python 3.10+**
- Un navigateur web moderne (Chrome, Firefox, Edge)
- Une **URL iCal ADE** de ton école (voir ci-dessous)
- Une **clé API Gemini** gratuite (voir ci-dessous)

Aucun Node.js n'est requis : Vue.js et Tailwind sont chargés par CDN.

---

## Installation et exécution

### 1. Cloner / récupérer le projet

```bash
git clone <URL_DU_DEPOT> tableau-de-bord-etudiant
cd tableau-de-bord-etudiant
```

### 2. Backend — environnement virtuel et dépendances

```bash
cd backend
python -m venv .venv
source .venv/bin/activate        # Windows : .venv\Scripts\activate
pip install -r requirements.txt
```

### 3. Configuration — fichier `.env`

Copie le modèle puis renseigne tes valeurs (toujours depuis `backend/`) :

```bash
cp ../.env.example .env
```

### 📅 Comment récupérer son lien ADE Campus

1. Connecte-toi à l'application web ADE de ton établissement.
2. Ouvre ton **emploi du temps personnel**.
3. Cherche l'option d'export (*Export* / *Agenda* / *iCal* / icône calendrier).
4. Copie le lien généré (il contient généralement `data=` et se termine par `.ics`) et colle-le dans `ADE_ICS_URL` :

```env
ADE_ICS_URL=https://ton-ecole.example.com/ade/ics?data=ton-lien-complet
```

### 🔑 Comment récupérer sa clé API Gemini

1. Va sur <https://aistudio.google.com/apikey>.
2. Connecte-toi avec ton compte Google.
3. Clique sur **Create API key**, copie la clé et colle-la dans `GEMINI_API_KEY` :

```env
GEMINI_API_KEY=ta-cle-ia-ici
```

> **Sécurité :** le fichier `.env` contient des secrets. Il est **exclu de Git** via `.gitignore`. Ne le committe jamais.

### 4. Lancer le backend

```bash
cd backend
uvicorn main:app --reload --port 8000
```

- Documentation interactive de l'API : <http://localhost:8000/docs>
- Le serveur utilise la variable `PORT` du `.env` si elle est définie.

### 5. Lancer le frontend

Ouvre le fichier directement :

```bash
open ../frontend/index.html        # macOS
xdg-open ../frontend/index.html    # Linux
```

…ou sers-le via un mini serveur statique (recommandé) :

```bash
cd frontend
python -m http.server 5173
```

Puis ouvre <http://localhost:5173>.

---

## Endpoints de l'API

| Méthode | Endpoint | Description | Corps attendu |
|---|---|---|---|
| `GET` | `/api/cours-du-jour` | Cours du jour depuis le flux iCal ADE | — |
| `GET` | `/api/devoirs` | Liste des devoirs (urgents d'abord) | — |
| `POST` | `/api/devoirs` | Créer un devoir | `{"titre", "matiere", "echeance", "fait"}` |
| `PATCH` | `/api/devoirs/{id}` | Mettre à jour partiellement | `{"fait": true}` par ex. |
| `DELETE` | `/api/devoirs/{id}` | Supprimer un devoir | — |
| `POST` | `/api/plan-revision` | Générer le plan de révision IA | — |

**Exemple :**

```bash
curl http://localhost:8000/api/cours-du-jour
curl -X POST http://localhost:8000/api/devoirs \
  -H "Content-Type: application/json" \
  -d '{"titre": "TP Physique", "matiere": "Physique", "echeance": "2026-08-10"}'
curl -X POST http://localhost:8000/api/plan-revision
```

---

## Publication sur GitHub

### 1. Initialiser le dépôt Git

```bash
cd tableau-de-bord-etudiant
git init -b main
```

### 2. Vérifier ce qui sera publié

```bash
git status
```

Le fichier `.env` (avec tes secrets) et la base `tableau_bord.db` **ne doivent pas apparaître** : ils sont déjà exclus par le `.gitignore`.

### 3. Premier commit

```bash
git add .
git commit -m "Initial commit : dashboard étudiant FastAPI + Vue.js + Gemini"
```

### 4. Créer le dépôt distant

**Option A — avec GitHub CLI (`gh`) :**

```bash
gh repo create tableau-de-bord-etudiant --public --source=. --push
```

**Option B — via l'interface GitHub :**
1. Crée un dépôt vide sur <https://github.com/new> (sans fichier README pour éviter les conflits).
2. Ajoute le remote puis pousse :

```bash
git remote add origin https://github.com/<TON_USER>/tableau-de-bord-etudiant.git
git branch -M main
git push -u origin main
```

### 5. Mises à jour suivantes

```bash
git add .
git commit -m "Amélioration : ..."
git push
```

---

## Sécurité

- Les clés (`GEMINI_API_KEY`) et URLs sensibles (`ADE_ICS_URL`) ne vivent **que** dans `.env`, jamais dans Git.
- Le CORS autorise actuellement toutes les origines pour un usage local. **En production**, restreins `allow_origins` dans `backend/main.py` à ton domaine.
- Ajoute un système d'authentification avant toute mise en production publique.

---

## Améliorations possibles

- Docker + `docker-compose` pour un déploiement one-command.
- Authentification (JWT) et multi-utilisateurs.
- Notifications (email / push) pour les échéances proches.
- Historique des plans de révision et suivi de progression.
- PWA (installable sur téléphone, notifications hors ligne).

---

## Licence

Distribué sous licence MIT. Libre de le forker, modifier et réutiliser.
