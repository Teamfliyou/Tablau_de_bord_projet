#!/usr/bin/env bash
# ============================================================
#  Tableau de bord étudiant — Installation automatique en une ligne
#  curl -fsSL https://raw.githubusercontent.com/Teamfliyou/Tablau_de_bord_projet/main/install.sh | bash
# ============================================================

set -e

echo "======================================================="
echo "  Tableau de bord étudiant — Installation"
echo "======================================================="

# 1) Vérification / installation des dépendances système
echo
echo "[1/4] Vérification des dépendances système…"

NEEDED=(git python3 python3-pip python3-venv curl)
MISSING=()
for pkg in "${NEEDED[@]}"; do
  if ! command -v "$pkg" >/dev/null 2>&1; then
    MISSING+=("$pkg")
  fi
done

if [ ${#MISSING[@]} -gt 0 ]; then
  echo "  Paquets manquants : ${MISSING[*]}"
  if command -v apt >/dev/null 2>&1; then
    echo "  Installation via apt…"
    sudo apt update && sudo apt install -y git python3 python3-pip python3-venv curl
    echo "  ✓ Dépendances système installées"
  else
    echo "  ⚠ apt non disponible — installez manuellement :"
    echo "     ${MISSING[*]}"
  fi
else
  echo "  ✓ Toutes les dépendances système sont présentes"
fi

# 2) Récupération du code source
echo
echo "[2/4] Récupération du code source…"
DEST_DIR="$HOME/Tablau_de_bord_projet"

if [ -d "$DEST_DIR/.git" ]; then
  echo "  Projet déjà présent — mise à jour (git pull)…"
  cd "$DEST_DIR"
  git pull
elif [ -d "$DEST_DIR" ]; then
  echo "  Dossier $DEST_DIR existant mais non versionné — entrée dans le dossier…"
  cd "$DEST_DIR"
else
  echo "  Clonage du dépôt…"
  git clone https://github.com/Teamfliyou/Tablau_de_bord_projet.git "$DEST_DIR"
  cd "$DEST_DIR"
fi

# 3) Préparation du script de lancement
echo
echo "[3/4] Préparation du script de lancement…"
chmod +x lancer_app.sh
echo "  ✓ lancer_app.sh exécutable"

# 4) Lancement de l'application
echo
echo "[4/4] Lancement de l'application…"
./lancer_app.sh
