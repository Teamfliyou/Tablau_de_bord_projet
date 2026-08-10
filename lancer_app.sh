#!/usr/bin/env bash
# ============================================================
#  Tableau de bord étudiant — Lancement Web App locale
#  Démarre le backend FastAPI + le frontend statique, puis
#  ouvre l'application dans Google Chrome en mode autonome.
# ============================================================

set -e

# Chemin absolu du projet (racine de ce script)
PROJET_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$PROJET_DIR/backend"
FRONTEND_DIR="$PROJET_DIR/frontend"
VENV="$BACKEND_DIR/.venv"
URL_APP="http://localhost:8000"
PORT_BACKEND=8000

echo "======================================================="
echo "  Tableau de bord étudiant — Démarrage"
echo "======================================================="

# 1) Environnement virtuel + dépendances
echo
echo "[1/4] Environnement virtuel…"
if [ ! -d "$VENV" ]; then
  echo "  Création de .venv dans backend/…"
  python3 -m venv "$VENV"
fi
source "$VENV/bin/activate"
pip install -r "$BACKEND_DIR/requirements.txt" -q
echo "  ✓ .venv prêt"

# 2) Backend uvicorn sur le port 8000 (sert l'API ET le frontend)
echo
echo "[2/3] Backend FastAPI (API + frontend)…"
if curl -s -o /dev/null -m 1 "http://localhost:$PORT_BACKEND/"; then
  echo "  ✓ uvicorn déjà lancé sur le port $PORT_BACKEND"
else
  echo "  Démarrage de uvicorn en arrière-plan…"
  cd "$BACKEND_DIR"
  setsid nohup uvicorn main:app --host 0.0.0.0 --port "$PORT_BACKEND" > "$BACKEND_DIR/uvicorn.log" 2>&1 < /dev/null &
  cd "$PROJET_DIR"
  sleep 2
  if curl -s -o /dev/null -m 1 "http://localhost:$PORT_BACKEND/"; then
    echo "  ✓ Backend lancé sur http://localhost:$PORT_BACKEND"
  else
    echo "  ⚠ Backend non détecté — consulte backend/uvicorn.log"
  fi
fi

# 3) Ouverture dans Chrome en mode application
echo
echo "[3/3] Ouverture de l'application…"
sleep 1

CHROME=""
for c in google-chrome google-chrome-stable chromium chromium-browser; do
  if command -v "$c" >/dev/null 2>&1; then
    CHROME="$c"
    break
  fi
done

if [ -n "$CHROME" ]; then
  setsid nohup "$CHROME" --app="$URL_APP" >/dev/null 2>&1 < /dev/null &
  echo "  ✓ Chrome ouvert en mode application : $URL_APP"
elif command -v garcon-url-handler >/dev/null 2>&1; then
  setsid nohup garcon-url-handler "$URL_APP" >/dev/null 2>&1 < /dev/null &
  echo "  ✓ Ouverture via garcon-url-handler : $URL_APP"
elif command -v xdg-open >/dev/null 2>&1; then
  setsid nohup xdg-open "$URL_APP" >/dev/null 2>&1 < /dev/null &
  echo "  ✓ Ouverture via xdg-open : $URL_APP"
else
  echo "  Aucun navigateur détecté — ouvre manuellement $URL_APP"
fi

echo
echo "======================================================="
echo "  Application prête. Bonne révision ! 📚"
echo "======================================================="
