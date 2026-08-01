# ============================================================
#  Tableau de bord étudiant — Image Docker
#  Backend FastAPI + frontend statique servis sur un seul port.
# ============================================================

FROM python:3.11-slim

# Évite les fichiers .pyc et garde les logs en temps réel
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PYTHONPATH=/app/backend \
    DATABASE_URL=sqlite:////app/backend/app.db

WORKDIR /app

# 1) Dépendances d'abord (meilleur cache Docker)
COPY backend/requirements.txt backend/requirements.txt
RUN pip install --no-cache-dir -r backend/requirements.txt

# 2) Code applicatif
COPY backend/ backend/
COPY frontend/ frontend/

# 3) Port de l'API + du frontend (servi par FastAPI)
EXPOSE 8000

# 4) Lancement : API + SPA en un seul processus
CMD ["uvicorn", "backend.main:app", "--host", "0.0.0.0", "--port", "8000"]
