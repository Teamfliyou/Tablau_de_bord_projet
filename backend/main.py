"""API FastAPI du tableau de bord étudiant.

Endpoints :
    GET   /api/cours-du-jour    -> emploi du temps du jour, synchronisé ADE (iCal)
    GET   /api/devoirs          -> liste des devoirs (SQLite)
    POST  /api/devoirs          -> création d'un devoir
    PATCH /api/devoirs/{id}     -> mise à jour d'un devoir (ex. marquer fait)
    DELETE /api/devoirs/{id}    -> suppression d'un devoir
    POST  /api/plan-revision    -> plan de révision du soir généré par Gemini

La configuration est chargée depuis le fichier `.env` (python-dotenv).
"""

import json
import os
import re
from contextlib import asynccontextmanager
from datetime import date, datetime, time
from typing import List
from zoneinfo import ZoneInfo

import httpx
import uvicorn
from dotenv import load_dotenv
from fastapi import Depends, FastAPI, HTTPException, Response
from fastapi.middleware.cors import CORSMiddleware
from icalendar import Calendar
from sqlmodel import Session, select

from database import Devoir, DevoirUpdate, create_db_and_tables, get_session

load_dotenv()

API_PREFIX = "/api"
ZONE = ZoneInfo(os.getenv("APP_TIMEZONE", "Europe/Paris"))
MODELE_IA = os.getenv("GEMINI_MODEL", "gemini-1.5-flash")


@asynccontextmanager
async def lifespan(_app: FastAPI):
    """Crée les tables SQLite au démarrage de l'application."""
    create_db_and_tables()
    yield


app = FastAPI(
    title="Tableau de bord étudiant",
    description=(
        "API personnelle : emploi du temps ADE en temps réel, "
        "gestion des devoirs et plan de révision généré par IA."
    ),
    version="1.0.0",
    lifespan=lifespan,
)

# CORS : autorise le frontend (servi en statique ou via un serveur local).
# Ajuste la liste des origines si tu déploies sur un domaine précis.
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=False,
    allow_methods=["*"],
    allow_headers=["*"],
)


# ---------------------------------------------------------------------------
#  Helpers iCal (ADE Campus)
# ---------------------------------------------------------------------------
def telecharger_ics(url: str) -> bytes:
    """Télécharge le fichier .ics depuis l'URL ADE fournie."""
    with httpx.Client(follow_redirects=True, timeout=20.0) as client:
        reponse = client.get(url)
        reponse.raise_for_status()
        return reponse.content


def parser_cours(contenu: bytes) -> List[dict]:
    """Parse un flux iCal et renvoie la liste structurée des événements.

    Chaque événement est converti en dict : titre, salle, enseignant,
    heures début/fin (HH:MM), timestamps ISO et date du jour.
    """
    calendrier = Calendar.from_ical(contenu)
    cours: List[dict] = []

    for composant in calendrier.walk():
        if composant.name != "VEVENT":
            continue

        raw_debut = composant.get("DTSTART")
        raw_fin = composant.get("DTEND")
        if raw_debut is None or raw_fin is None:
            continue

        debut = raw_debut.dt
        fin = raw_fin.dt

        # Gestion des événements "journée entière" (sans horaire).
        if not isinstance(debut, datetime):
            debut = datetime.combine(debut, time.min)
        if not isinstance(fin, datetime):
            fin = datetime.combine(fin, time.min)

        # Normalisation dans le fuseau horaire configuré.
        debut = debut.astimezone(ZONE) if debut.tzinfo else debut.replace(tzinfo=ZONE)
        fin = fin.astimezone(ZONE) if fin.tzinfo else fin.replace(tzinfo=ZONE)

        cours.append(
            {
                "titre": str(composant.get("SUMMARY") or "Cours sans titre").strip(),
                "salle": str(composant.get("LOCATION") or "").strip(),
                "enseignant": str(composant.get("DESCRIPTION") or "").strip(),
                "debut": debut.strftime("%H:%M"),
                "fin": fin.strftime("%H:%M"),
                "debut_iso": debut.isoformat(),
                "fin_iso": fin.isoformat(),
                "date": debut.date().isoformat(),
            }
        )

    cours.sort(key=lambda c: c["debut_iso"])
    return cours


def charger_cours_du_jour() -> dict:
    """Fonction interne : cours d'aujourd'hui (utilisée par la route et l'IA)."""
    url = os.getenv("ADE_ICS_URL")
    if not url:
        raise HTTPException(
            status_code=500, detail="ADE_ICS_URL est absente du fichier .env"
        )

    try:
        tous_les_cours = parser_cours(telecharger_ics(url))
    except Exception as exc:
        raise HTTPException(
            status_code=502, detail=f"Impossible de lire le flux ADE : {exc}"
        )

    aujourd_hui = datetime.now(ZONE).date().isoformat()
    cours = [c for c in tous_les_cours if c["date"] == aujourd_hui]
    return {"date": aujourd_hui, "nb_cours": len(cours), "cours": cours}


# ---------------------------------------------------------------------------
#  Helpers IA (Google Gemini)
# ---------------------------------------------------------------------------
def construire_prompt(cours: List[dict], devoirs: List[dict], heure_debut: str, heure_fin: str) -> str:
    """Construit le prompt envoyé à l'IA à partir des données du jour."""
    cours_txt = "\n".join(
        f"- {c['titre']} ({c['debut']} -> {c['fin']}, salle {c['salle'] or 'non renseignée'})"
        for c in cours
    ) or "- Aucun cours aujourd'hui."

    devoirs_txt = "\n".join(
        f"- {d['titre']} | matière : {d['matiere']} | échéance : {d['echeance']} | statut : {'fait' if d['fait'] else 'à faire'}"
        for d in devoirs
    ) or "- Aucun devoir enregistré."

    return f"""Tu es un coach de révision personnel pour un étudiant en école d'ingénieur.

Cours de la journée :
{cours_txt}

Devoirs enregistrés (avec statut) :
{devoirs_txt}

Plage de travail disponible ce soir : {heure_debut} -> {heure_fin}.

Génère un plan de révision du soir au format JSON strict (sans texte autour), avec cette structure EXACTE :
{{
  "conseil_soir": "un conseil de méthode en 2 ou 3 phrases",
  "sessions": [
    {{
      "matiere": "nom de la matière",
      "duree_minutes": 30,
      "objectif": "objectif concret et réalisable",
      "technique": "technique de travail (ex: Pomodoro, flashcards, exercices chronométrés)"
    }}
  ]
}}

Consignes :
- Ignore les devoirs marqués comme faits.
- Priorise les devoirs dont l'échéance est la plus proche.
- Propose 2 à 4 sessions, avec des pauses de 5 à 10 minutes entre elles.
- Ne révise pas en priorité ce qui a déjà été vu aujourd'hui en cours.
- Réponds UNIQUEMENT avec du JSON valide, aucune autre sortie."""


def generer_plan_avec_gemini(prompt: str) -> dict:
    """Interroge l'API Gemini et renvoie le plan de révision parsé en JSON."""
    api_key = os.getenv("GEMINI_API_KEY")
    if not api_key:
        raise HTTPException(
            status_code=500, detail="GEMINI_API_KEY est absente du fichier .env"
        )

    try:
        import google.generativeai as genai
    except ImportError:
        raise HTTPException(
            status_code=500,
            detail="Paquet 'google-generativeai' manquant : pip install -r requirements.txt",
        )

    genai.configure(api_key=api_key)
    modele = genai.GenerativeModel(
        MODELE_IA,
        generation_config={"response_mime_type": "application/json", "temperature": 0.7},
    )
    reponse = modele.generate_content(prompt)

    texte = reponse.text.strip()
    texte = re.sub(r"^```(?:json)?\s*|\s*```$", "", texte)

    try:
        return json.loads(texte)
    except json.JSONDecodeError as exc:
        raise HTTPException(
            status_code=502, detail=f"L'IA n'a pas renvoyé de JSON valide : {exc}"
        )


# ---------------------------------------------------------------------------
#  Emploi du temps (ADE)
# ---------------------------------------------------------------------------
@app.get(f"{API_PREFIX}/cours-du-jour")
def get_cours_du_jour() -> dict:
    """Renvoie les cours d'aujourd'hui depuis le flux iCal ADE."""
    return charger_cours_du_jour()


# ---------------------------------------------------------------------------
#  Devoirs (SQLite)
# ---------------------------------------------------------------------------
@app.get(f"{API_PREFIX}/devoirs", response_model=List[Devoir])
def lister_devoirs(session: Session = Depends(get_session)) -> List[Devoir]:
    """Liste tous les devoirs : urgents d'abord, terminés en dernier."""
    devoirs = session.exec(select(Devoir)).all()
    return sorted(devoirs, key=lambda d: (d.fait, d.echeance or "9999-12-31"))


@app.post(f"{API_PREFIX}/devoirs", response_model=Devoir, status_code=201)
def creer_devoir(devoir: Devoir, session: Session = Depends(get_session)) -> Devoir:
    """Crée un nouveau devoir."""
    session.add(devoir)
    session.commit()
    session.refresh(devoir)
    return devoir


@app.patch(f"{API_PREFIX}/devoirs/{{id}}", response_model=Devoir)
def maj_devoir(
    id: int, mise_a_jour: DevoirUpdate, session: Session = Depends(get_session)
) -> Devoir:
    """Met à jour partiellement un devoir (ex. le marquer comme fait)."""
    devoir = session.get(Devoir, id)
    if devoir is None:
        raise HTTPException(status_code=404, detail="Devoir introuvable")

    for champ, valeur in mise_a_jour.model_dump(exclude_unset=True).items():
        setattr(devoir, champ, valeur)

    session.add(devoir)
    session.commit()
    session.refresh(devoir)
    return devoir


@app.delete(f"{API_PREFIX}/devoirs/{{id}}", status_code=204)
def supprimer_devoir(id: int, session: Session = Depends(get_session)) -> Response:
    """Supprime un devoir."""
    devoir = session.get(Devoir, id)
    if devoir is None:
        raise HTTPException(status_code=404, detail="Devoir introuvable")

    session.delete(devoir)
    session.commit()
    return Response(status_code=204)


# ---------------------------------------------------------------------------
#  Plan de révision IA
# ---------------------------------------------------------------------------
@app.post(f"{API_PREFIX}/plan-revision")
def generer_plan_revision(session: Session = Depends(get_session)) -> dict:
    """Génère un plan de révision du soir grâce à Gemini (JSON strict)."""
    donnees_cours = charger_cours_du_jour()

    devoirs_db = session.exec(select(Devoir)).all()
    devoirs = [
        {
            "titre": d.titre,
            "matiere": d.matiere or "non précisée",
            "echeance": d.echeance or "non précisée",
            "fait": d.fait,
        }
        for d in devoirs_db
    ]

    heure_debut = os.getenv("HEURE_DEBUT_ETUDES", "19:00")
    heure_fin = os.getenv("HEURE_FIN_ETUDES", "22:00")
    prompt = construire_prompt(donnees_cours["cours"], devoirs, heure_debut, heure_fin)

    try:
        plan = generer_plan_avec_gemini(prompt)
    except HTTPException:
        raise
    except Exception as exc:
        raise HTTPException(status_code=502, detail=f"Erreur lors de l'appel à l'IA : {exc}")

    return {"date": donnees_cours["date"], "modele": MODELE_IA, "plan": plan}


if __name__ == "__main__":
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=int(os.getenv("PORT", "8000")),
        reload=True,
    )
