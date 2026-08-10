# 🎓 Tableau de bord étudiant

Dashboard personnel **Full-Stack** pour visualiser son emploi du temps en temps réel (flux iCal ADE Campus), gérer ses devoirs via un tableau Kanban et bénéficier d'un **plan de révision intelligent** généré par IA (Google Gemini).

> **Stack** : **FastAPI** (backend Python) + **Vue.js 3 / Tailwind CSS** (frontend sans build Node.js) + **SQLite / SQLAlchemy** (stockage local) + **Gemini** (génération de plans de révision).

---

## 🏷️ Badges

![FastAPI](https://img.shields.io/badge/FastAPI-0.110+-009688?logo=fastapi&logoColor=white)
![Vue.js](https://img.shields.io/badge/Vue.js-3-42b883?logo=vue.js&logoColor=white)
![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-3-38bdf8?logo=tailwindcss&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-003B57?logo=sqlite&logoColor=white)
![Gemini](https://img.shields.io/badge/Gemini-1.5_Flash-8E75B2?logo=google&logoColor=white)
![Python](https://img.shields.io/badge/Python-3.11+-3776AB?logo=python&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?logo=docker&logoColor=white)
![Licence](https://img.shields.io/badge/Licence-MIT-blue)

---

## ✨ Fonctionnalités clés

| Fonctionnalité | Description |
|---|---|
| 📅 **Synchronisation ADE live** | Emploi du temps du jour et de la semaine synchronisé avec le flux iCal de l'école (ADE Campus). Auto-actualisation toutes les 60 s, badge "EN COURS" en temps réel et navigation de semaine en semaine. |
| 🏷️ **Filtres & Personnalisation** | Analyse automatique du flux ADE pour détecter et filtrer les cours selon tes groupes (TP/TD/Khôlles), types d'épreuves (DS, IE, Exam) ou matières. |
| 📋 **Gestion de devoirs (Kanban)** | Ajout, modification de statut (*À faire*, *En cours*, *Terminé*), suppression et badges d'échéance intelligents (aujourd'hui, demain, retard, J-X). Persistance SQLite. |
| 🤖 **Plan de révision par IA** | Analyse simultanée des cours du jour et des devoirs par Google Gemini pour créer un plan de soirée structuré (format JSON strict). Mode simulation disponible sans clé API. |
| ⚙️ **Configuration intégrée** | Saisie de l'URL ADE, de la clé Gemini et du pseudo directement depuis l'interface web (sauvegarde en base SQLite localement). |
| 🎨 **Interface moderne & Bento Box** | Design fluide (Liquid Glass / Bento), mode d'affichage au choix (Modern Dock en bas ou Classic Sidebar) et responsive. |

---

## 🛠️ Guide d'installation complet des dépendances

### ⚡ Installation ultra-rapide en une ligne

Le script `install.sh` vérifie et installe automatiquement les dépendances système (`git`, `python3`, `python3-pip`, `python3-venv`, `curl`), clone le dépôt, rend `lancer_app.sh` exécutable puis lance l'application.

```bash
curl -fsSL https://raw.githubusercontent.com/Teamfliyou/Tablau_de_bord_projet/main/install.sh | bash
```

Avant de pouvoir utiliser l'application, tu dois installer les outils de base sur ta machine. Suis la section correspondant à ton système d'exploitation.

---

### 🌐 Étape 1 : Préparer l'environnement système

#### 🐧 Sur Linux (Ubuntu / Debian / Mint)
Ouvre ton terminal et exécute :
```bash
# 1. Mettre à jour les paquets
sudo apt update && sudo apt upgrade -y

# 2. Installer Git, Python3, Pip et le module venv
sudo apt install -y git python3 python3-pip python3-venv curl

# 3. (Optionnel mais recommandé) Installer Docker & Docker Compose
sudo apt install -y docker.io docker-compose-v2
sudo usermod -aG docker $USER
# ⚠️ Déconnecte-toi puis reconnecte-toi pour appliquer la permission Docker
```

#### 🍏 Sur macOS
Si tu n'as pas **Homebrew**, installe-le en ouvrant le Terminal :
```bash
/bin/bash -c "$(curl -fsSL [https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh](https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh))"
```
Puis installe les dépendances nécessaires :
```bash
# 1. Installer Git et Python
brew install git python

# 2. (Optionnel) Installer Docker Desktop via Homebrew
brew install --cask docker
```

#### 🪟 Sur Windows
1. **Python** : Télécharge et installe Python (v3.10+) depuis [python.org](https://www.python.org/downloads/).  
   ⚠️ **CRUCIAL** : Coche bien la case **"Add python.exe to PATH"** lors de l'installation !
2. **Git** : Télécharge et installe Git depuis [git-scm.com](https://git-scm.com/).
3. **Docker (Optionnel)** : Télécharge et installe [Docker Desktop pour Windows](https://www.docker.com/products/docker-desktop/).

---

### 📥 Étape 2 : Récupérer le code source

Ouvre un terminal (PowerShell sur Windows, Terminal sur Linux/macOS) :

```bash
git clone [https://github.com/Teamfliyou/Tablau_de_bord_projet.git](https://github.com/Teamfliyou/Tablau_de_bord_projet.git)
cd Tablau_de_bord_projet
```

---

### 🚀 Étape 3 : Lancer l'application (au choix)

Procède selon la méthode de ton choix :

---

#### 🐳 Méthode A : Avec Docker (Recommandée & Sans configuration Python)

Cette méthode ne nécessite pas de créer d'environnement virtuel Python sur ton PC.

```bash
# Lancer le conteneur en arrière-plan
docker compose up -d
```

* **Accéder à l'application web** : [http://localhost:8000](http://localhost:8000)
* **Consulter la documentation Swagger de l'API** : [http://localhost:8000/docs](http://localhost:8000/docs)

---

#### ⚡ Méthode B : Lancement automatique par script (Linux / macOS / WSL)

Un script automatise la création de l'environnement `.venv`, l'installation des paquets Python et le démarrage des serveurs.

```bash
# Rendre le script exécutable
chmod +x lancer_app.sh

# Lancer le script
./lancer_app.sh
```

---

#### 🛠️ Méthode C : Installation manuelle pas à pas (Tout OS)

##### 1. Créer et activer l'environnement virtuel Python
```bash
cd backend

# Créer le venv
python3 -m venv .venv
# (Sur Windows si 'python3' ne marche pas, tape : python -m venv .venv)

# Activer le venv :
# -> Sur Linux / macOS :
source .venv/bin/activate

# -> Sur Windows (PowerShell) :
# .venv\Scripts\Activate.ps1

# -> Sur Windows (CMD) :
# .venv\Scripts\activate.bat
```

##### 2. Installer les dépendances Python du projet
```bash
pip install --upgrade pip
pip install -r requirements.txt
```

*(Les dépendances installées sont : `fastapi`, `uvicorn`, `sqlalchemy`, `ics`, `requests`, `python-dotenv`, `google-generativeai`, `pydantic`)*.

##### 3. Créer le fichier de configuration `.env`
À la racine du projet ou dans `backend/` :
```bash
cp ../.env.example .env
```

##### 4. Démarrer le serveur backend
```bash
uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

L'application web est directement accessible sur **[http://localhost:8000](http://localhost:8000)** (FastAPI sert directement le frontend HTML/Vue.js).

---

## ⚙️ Configuration (`.env` ou Interface Web)

Vous pouvez configurer l'application soit via les paramètres de l'onglet **Configuration** de l'interface, soit via le fichier `.env` :

```env
# URL d'export iCal de votre emploi du temps ADE
ADE_ICS_URL=[https://votre-ecole.example.com/ade/ics?data=votre-lien-ics](https://votre-ecole.example.com/ade/ics?data=votre-lien-ics)

# Clé API Google Gemini (Obtenable gratuitement sur [https://aistudio.google.com/apikey](https://aistudio.google.com/apikey))
GEMINI_API_KEY=votre_cle_gemini_ici

# Modèle Gemini utilisé (Facultatif, par défaut gemini-1.5-flash)
GEMINI_MODEL=gemini-1.5-flash

# Fuseau horaire (Facultatif, par défaut Europe/Paris)
APP_TIMEZONE=Europe/Paris

# Port d'écoute du serveur
PORT=8000
```

---

## 📁 Architecture du projet

```text
Tablau_de_bord_projet/
├── backend/
│   ├── main.py            # API FastAPI (routes + parsing ADE + intégration Gemini + SPA)
│   ├── database.py        # Modèles SQLAlchemy (Devoir, Config) & connexion SQLite
│   ├── requirements.txt   # Dépendances Python
│   └── app.db             # Base de données SQLite (créée au lancement)
├── frontend/
│   └── index.html         # SPA Vue.js 3 + Tailwind CSS + Lucide Icons (chargés via CDN)
├── Dockerfile             # Multi-stage image : Python 3.11-slim (API + Frontend)
├── docker-compose.yml     # Lancement one-command avec persistance du volume app.db
├── lancer_app.sh          # Script Bash local (venv + uvicorn + frontend + Chrome app)
├── .env.example           # Modèle des variables d'environnement
├── .gitignore             # Exclusion des secrets, caches et bases de données
└── README.md
```

**Déroulement d'une requête :**
1. **Frontend (Vue.js 3)** fait des appels `fetch` vers l'API FastAPI.
2. **FastAPI** interroge :
   - **ADE Campus** : Téléchargement et parsing du flux `.ics` via la librairie Python `ics`.
   - **SQLite (`app.db`)** : Gestion des devoirs et persistance de la configuration.
   - **Google Gemini API** : Génération du plan de révision du soir selon un prompt structuré.

---

## 📡 Endpoints de l'API REST

| Méthode | Endpoint | Description |
|---|---|---|
| `GET` | `/api/cours-du-jour` | Récupère les cours de la journée (filtrables par `groupes`, `types`, `matieres`) |
| `GET` | `/api/cours-semaine` | Récupère les cours de la semaine pour une date donnée |
| `GET` | `/api/devoirs` | Liste l'ensemble des devoirs stockés en base |
| `POST` | `/api/devoirs` | Crée un nouveau devoir |
| `PATCH` | `/api/devoirs/{id}` | Met à jour le statut d'un devoir (*a_faire*, *en_cours*, *termine*) |
| `DELETE` | `/api/devoirs/{id}` | Supprime un devoir |
| `POST` | `/api/plan-revision` | Génère le plan de révision IA à partir des cours et devoirs |
| `GET` | `/api/config` | Récupère la configuration utilisateur enregistrée |
| `POST` | `/api/config` | Enregistre le pseudo, le lien ADE et la clé Gemini en base SQLite |
| `GET` | `/api/config/categories` | Analyse le flux ADE pour extraire les groupes, types et matières disponibles |

---

## 🐳 Commandes Docker utiles

```bash
docker compose down            # Arrête le conteneur (les données restent sauvegardées dans backend/app.db)
docker compose up -d --build   # Reconstruit l'image après une modification du code
docker compose logs -f app     # Affiche les logs du backend en temps réel
```

---

## 🔒 Sécurité

- Les clés d'API et URLs ADE sont stockées localement dans la base SQLite ou dans le fichier `.env` non versionné (`.gitignore`).
- En production publique, ajustez le Middleware CORS (`allow_origins`) dans `backend/main.py`.

---

## 📄 Licence

Projet distribué sous licence **MIT**. Réutilisation, modification et distribution libres.
