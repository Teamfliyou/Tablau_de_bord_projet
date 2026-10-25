#!/usr/bin/env python3
"""Convertir un export Kanri (colonnes/cartes/tasks) en devoirs JSON.

Usage:
    python scripts/convertir_kanri.py export-kanri.json import-devoirs.json
"""
import json
import re
import sys
from datetime import datetime
from html import unescape
from pathlib import Path
from zoneinfo import ZoneInfo

STATUTS = {
    "à faire": "a_faire", "a faire": "a_faire",
    "en cours": "en_cours", "finis": "termine",
    "fini": "termine", "terminé": "termine", "terminés": "termine",
}

def nettoyer_html(texte):
    texte = re.sub(r"(?i)<\s*br\s*/?>|</\s*p\s*>|</\s*li\s*>", "\n", texte or "")
    return unescape(re.sub(r"<[^>]*>", "", texte)).strip()[:2000]

def convertir(data):
    if not isinstance(data, dict) or not isinstance(data.get("columns"), list):
        raise ValueError("Ce JSON ne contient pas un tableau Kanri (columns).")
    sortie = []
    for colonne in data["columns"]:
        statut = STATUTS.get(str(colonne.get("title", "")).strip().casefold())
        if not statut:
            raise ValueError("Colonne Kanri inconnue : " + str(colonne.get("title")))
        for carte in colonne.get("cards", []):
            titre = str(carte.get("name", "")).strip()
            if not titre:
                continue
            tags = []
            for etiquette in carte.get("tags", []):
                nom = str(etiquette.get("text", "")).strip()[:30]
                couleur = str(etiquette.get("color", "#6366f1")).lower()
                if not re.fullmatch(r"#[0-9a-f]{6}", couleur):
                    couleur = "#6366f1"
                if nom and len(tags) < 20:
                    tags.append({"nom": nom, "couleur": couleur})
            tags_noms = {t["nom"].casefold() for t in tags}
            matiere = "Cours magistral" if "cm" in tags_noms else "Travaux dirigés" if "td" in tags_noms else ""
            echeance = ""
            if carte.get("dueDate"):
                instant = datetime.fromisoformat(carte["dueDate"].replace("Z", "+00:00"))
                echeance = instant.astimezone(ZoneInfo("Europe/Paris")).date().isoformat()
            sortie.append({
                "titre": titre[:200], "matiere": matiere, "echeance": echeance,
                "type": "devoir", "statut": statut,
                "tags": tags,
                "todos": [
                    {"texte": str(t.get("name", "")).strip()[:200], "fait": bool(t.get("finished", False))}
                    for t in carte.get("tasks", [])
                    if str(t.get("name", "")).strip()
                ][:100],
                "description": nettoyer_html(carte.get("description", "")),
            })
    if len(sortie) > 500:
        raise ValueError("Limite : 500 devoirs par fichier.")
    return sortie

if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    source, destination = map(Path, sys.argv[1:])
    resultat = convertir(json.loads(source.read_text(encoding="utf-8")))
    destination.write_text(json.dumps(resultat, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{len(resultat)} devoir(s), {sum(len(d['todos']) for d in resultat)} sous-tâche(s) convertis.")
