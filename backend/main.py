import json
import os
from datetime import date, datetime, time, timedelta
from typing import Optional
from zoneinfo import ZoneInfo

import requests
from dotenv import load_dotenv
from fastapi import Depends, FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from ics import Calendar
from pydantic import BaseModel
from sqlalchemy.orm import Session

from database import Config, Devoir, SessionLocal

load_dotenv()

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

ZONE = ZoneInfo(os.getenv("APP_TIMEZONE", "Europe/Paris"))


def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()


# ---------------------------------------------------------------------------
#  Schémas
# ---------------------------------------------------------------------------
class DevoirCreate(BaseModel):
    titre: str
    matiere: str = ""
    echeance: str = ""
    type: str = "devoir"
    statut: str = "a_faire"


class DevoirUpdate(BaseModel):
    statut: Optional[str] = None


class ConfigUpdate(BaseModel):
    username: str = ""
    ade_url: str = ""
    gemini_key: str = ""


# ---------------------------------------------------------------------------
#  Emploi du temps (parsing iCal ADE réel)
# ---------------------------------------------------------------------------
def obtenir_url_ade(db: Session = None) -> str:
    """Renvoie l'URL ADE : config en base, sinon variable d'environnement."""
    url = None
    if db is not None:
        config = db.query(Config).first()
        if config and config.ade_url:
            url = config.ade_url
    if not url:
        url = os.getenv("ADE_ICS_URL")
    if not url:
        raise HTTPException(
            status_code=500,
            detail="Aucune URL ADE configurée : renseigne-la dans l'onglet Configuration ou ADE_ICS_URL dans .env",
        )
    return url


def telecharger_ics(url: str) -> str:
    """Télécharge le fichier .ics depuis l'URL ADE fournie."""
    try:
        reponse = requests.get(url, timeout=20)
        reponse.raise_for_status()
    except requests.RequestException as exc:
        raise HTTPException(
            status_code=502,
            detail=f"Impossible de télécharger le flux ADE ({url}) : {exc}",
        )
    return reponse.text


def parser_cours(contenu: str) -> list:
    """Parse un flux iCal (librairie ics) et renvoie la liste structurée des événements."""
    try:
        calendrier = Calendar(contenu)
    except Exception as exc:
        raise HTTPException(
            status_code=502,
            detail=f"Impossible de lire le fichier ICS : {exc}",
        )

    cours: list = []
    for evenement in calendrier.events:
        try:
            titre = str(evenement.name or "Cours sans titre").strip()
            debut = evenement.begin.datetime
            fin = evenement.end.datetime

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
                    "titre": titre,
                    "salle": str(evenement.location or "").strip(),
                    "enseignant": str(evenement.description or "").strip(),
                    "debut": debut.strftime("%H:%M"),
                    "fin": fin.strftime("%H:%M"),
                    "debut_iso": debut.isoformat(),
                    "fin_iso": fin.isoformat(),
                    "date": debut.date().isoformat(),
                }
            )
        except Exception:
            # On ignore les événements malformés pour ne pas bloquer tout le flux.
            continue

    cours.sort(key=lambda c: c["debut_iso"])
    return cours


def charger_cours(db: Session = None) -> list:
    """Télécharge et parse tous les cours du flux ADE."""
    url = obtenir_url_ade(db)
    return parser_cours(telecharger_ics(url))


def charger_cours_du_jour(db: Session = None, jour: date = None) -> dict:
    """Cours d'une journée précise (aujourd'hui par défaut) filtrés depuis le flux ADE."""
    tous_les_cours = charger_cours(db)
    jour = jour or date.today()
    cours = [c for c in tous_les_cours if c["date"] == jour.isoformat()]
    return {"date": jour.isoformat(), "nb_cours": len(cours), "cours": cours}


def charger_cours_semaine(db: Session = None, ref: date = None) -> dict:
    """Cours du lundi au vendredi de la semaine contenant `ref` (aujourd'hui par défaut)."""
    tous_les_cours = charger_cours(db)
    ref = ref or date.today()
    lundi = ref - timedelta(days=ref.weekday())
    vendredi = lundi + timedelta(days=4)
    cours = [
        c
        for c in tous_les_cours
        if lundi <= date.fromisoformat(c["date"]) <= vendredi
    ]
    return {"date_debut": lundi.isoformat(), "date_fin": vendredi.isoformat(), "cours": cours}


def parser_date(valeur: Optional[str]) -> Optional[date]:
    """Valide une date au format YYYY-MM-DD (retourne None si absente)."""
    if not valeur:
        return None
    try:
        return date.fromisoformat(valeur)
    except ValueError:
        raise HTTPException(
            status_code=400,
            detail=f"Date invalide : {valeur} (format attendu YYYY-MM-DD)",
        )


@app.get("/api/cours-du-jour")
def get_cours_du_jour(date: Optional[str] = None, db: Session = Depends(get_db)):
    return charger_cours_du_jour(db, parser_date(date))


@app.get("/api/cours-semaine")
def get_cours_semaine(date: Optional[str] = None, db: Session = Depends(get_db)):
    return charger_cours_semaine(db, parser_date(date))


# ---------------------------------------------------------------------------
#  Devoirs (SQLite via SQLAlchemy)
# ---------------------------------------------------------------------------
@app.get("/api/devoirs")
def lister_devoirs(db: Session = Depends(get_db)):
    return db.query(Devoir).all()


@app.post("/api/devoirs")
def creer_devoir(devoir: DevoirCreate, db: Session = Depends(get_db)):
    nouveau = Devoir(
        titre=devoir.titre,
        matiere=devoir.matiere,
        echeance=devoir.echeance,
        type=devoir.type,
        statut=devoir.statut,
    )
    db.add(nouveau)
    db.commit()
    db.refresh(nouveau)
    return nouveau


@app.patch("/api/devoirs/{devoir_id}")
def maj_devoir(devoir_id: int, mise_a_jour: DevoirUpdate, db: Session = Depends(get_db)):
    devoir = db.query(Devoir).filter(Devoir.id == devoir_id).first()
    if devoir is None:
        raise HTTPException(status_code=404, detail="Devoir introuvable")
    if mise_a_jour.statut is not None:
        devoir.statut = mise_a_jour.statut
    db.commit()
    db.refresh(devoir)
    return devoir


@app.delete("/api/devoirs/{devoir_id}")
def supprimer_devoir(devoir_id: int, db: Session = Depends(get_db)):
    devoir = db.query(Devoir).filter(Devoir.id == devoir_id).first()
    if devoir is None:
        raise HTTPException(status_code=404, detail="Devoir introuvable")
    db.delete(devoir)
    db.commit()
    return {"detail": "Devoir supprimé"}


# ---------------------------------------------------------------------------
#  Configuration
# ---------------------------------------------------------------------------
@app.get("/api/config")
def get_config(db: Session = Depends(get_db)):
    config = db.query(Config).first()
    if config is None:
        return {
            "username": os.getenv("USERNAME", ""),
            "ade_url": os.getenv("ADE_ICS_URL", ""),
            "gemini_key": os.getenv("GEMINI_API_KEY", ""),
        }
    return {
        "username": config.username,
        "ade_url": config.ade_url,
        "gemini_key": config.gemini_key,
    }


@app.post("/api/config")
def save_config(data: ConfigUpdate, db: Session = Depends(get_db)):
    config = db.query(Config).first()
    if config is None:
        config = Config(id=1, username=data.username, ade_url=data.ade_url, gemini_key=data.gemini_key)
        db.add(config)
    else:
        config.username = data.username
        config.ade_url = data.ade_url
        config.gemini_key = data.gemini_key
    db.commit()
    return {"detail": "Configuration enregistrée"}


# ---------------------------------------------------------------------------
#  Plan de révision IA (Gemini ou simulation)
# ---------------------------------------------------------------------------
def construire_prompt(cours: list, devoirs: list) -> str:
    cours_txt = "\n".join(
        f"- {c['titre']} ({c['debut']} -> {c['fin']}, salle {c.get('salle') or 'non renseignée'})"
        for c in cours
    ) or "- Aucun cours aujourd'hui."

    devoirs_txt = "\n".join(
        f"- {d.titre} | matière : {d.matiere or 'non précisée'} | type : {d.type} | "
        f"échéance : {d.echeance or 'non précisée'} | statut : {d.statut}"
        for d in devoirs
    ) or "- Aucun devoir enregistré."

    return f"""Tu es un coach de révision personnel pour un étudiant en école d'ingénieur.

Cours de la journée :
{cours_txt}

Devoirs enregistrés :
{devoirs_txt}

Génère un plan de révision du soir au format JSON strict (sans texte autour), avec cette structure EXACTE :
{{
  "conseil": "un conseil général sur la gestion de la soirée, en 2 ou 3 phrases",
  "planning": [
    {{
      "heure": "18:00 - 18:45",
      "matiere": "nom de la matière",
      "action": "tâche précise et réalisable"
    }}
  ]
}}

Consignes :
- Priorise les devoirs non terminés : les examens (exam) et DS d'abord, puis les IE, puis les devoirs.
- Parmi ceux-là, priorise ceux dont l'échéance est la plus proche (attention aux dates).
- Ne révise pas ce qui a déjà été vu aujourd'hui en cours si possible.
- Propose 3 à 5 créneaux le soir, avec des pauses de 5 à 10 minutes.
- Réponds UNIQUEMENT avec du JSON valide, aucune autre sortie."""


POIDS_TYPE = {"exam": 0, "ds": 1, "ie": 2, "devoir": 3}


def simuler_plan(cours: list, devoirs: list) -> dict:
    """Plan généré localement quand aucune clé Gemini n'est configurée."""
    # Devoirs non terminés, triés par priorité (type puis échéance la plus proche).
    non_termines = [d for d in devoirs if d.statut != "termine"]
    non_termines.sort(
        key=lambda d: (POIDS_TYPE.get(d.type, 3), d.echeance or "9999-12-31")
    )
    matieres = [d.titre for d in non_termines]

    # Complète avec les matières des cours du jour non déjà listées.
    for c in cours:
        if len(matieres) >= 4:
            break
        if c["titre"] not in matieres:
            matieres.append(c["titre"])

    planning = []
    heures = [("18:00", "18:45"), ("18:55", "19:40"), ("19:50", "20:35"), ("20:45", "21:30")]
    for i, matiere in enumerate(matieres[:4]):
        if i >= len(heures):
            break
        planning.append({
            "heure": f"{heures[i][0]} - {heures[i][1]}",
            "matiere": matiere,
            "action": f"Réviser {matiere} et faire des exercices ciblés",
        })

    if not planning:
        planning.append({
            "heure": "18:00 - 18:45",
            "matiere": "Revue de la journée",
            "action": "Relire tes notes de cours et anticiper la prochaine séance",
        })

    return {
        "conseil": (
            "Commence par une session de 45 minutes pour te mettre dans le bain, "
            "puis alterne révisions et courtes pauses. Termine en douceur en relisant "
            "tes notes plutôt qu'en attaquant un nouveau chapitre."
        ),
        "planning": planning,
    }


@app.post("/api/plan-revision")
def generer_plan_revision(db: Session = Depends(get_db)):
    """Génère un plan de révision du soir (Gemini si configuré, sinon simulation)."""
    cours = charger_cours_du_jour(db)["cours"]
    devoirs = db.query(Devoir).all()
    prompt = construire_prompt(cours, devoirs)

    api_key = os.getenv("GEMINI_API_KEY")
    config = db.query(Config).first()
    if config and config.gemini_key:
        api_key = config.gemini_key

    if api_key:
        try:
            import google.generativeai as genai

            genai.configure(api_key=api_key)
            modele = genai.GenerativeModel(
                os.getenv("GEMINI_MODEL", "gemini-1.5-flash"),
                generation_config={"response_mime_type": "application/json", "temperature": 0.7},
            )
            texte = modele.generate_content(prompt).text.strip()
            texte = texte.removeprefix("```json").removeprefix("```").removesuffix("```").strip()
            return json.loads(texte)
        except Exception:
            # Si l'appel échoue, on retombe sur la simulation locale.
            return simuler_plan(cours, devoirs)

    return simuler_plan(cours, devoirs)
