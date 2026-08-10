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
