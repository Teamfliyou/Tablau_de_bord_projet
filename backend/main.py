import asyncio
import json
import os
import re
import time
from datetime import date, datetime, timedelta, timezone
from pathlib import Path
from typing import Optional
from zoneinfo import ZoneInfo

import httpx
import jwt
from dotenv import load_dotenv
from fastapi import Depends, FastAPI, HTTPException, Query
from fastapi.middleware.cors import CORSMiddleware
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
from fastapi.staticfiles import StaticFiles
from ics import Calendar
from passlib.context import CryptContext
from pydantic import BaseModel
from sqlalchemy.orm import Session

from database import Devoir, SessionLocal, User

load_dotenv()

app = FastAPI()

_allowed_origins_raw = os.getenv("ALLOWED_ORIGINS", "*")
_allowed_origins = [o.strip() for o in _allowed_origins_raw.split(",") if o.strip()]

app.add_middleware(
    CORSMiddleware,
    allow_origins=_allowed_origins,
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
#  Authentification : mots de passe (bcrypt) & tokens JWT
# ---------------------------------------------------------------------------
SECRET_KEY = os.getenv("SECRET_KEY", "cle-de-developpement-a-changer-en-production")
JWT_ALGORITHME = "HS256"
JWT_EXPIRATION_MINUTES = 60 * 24  # 24 heures

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")
bearer_scheme = HTTPBearer(auto_error=False)


def hacher_mdp(mot_de_passe: str) -> str:
    return pwd_context.hash(mot_de_passe)


def verifier_mdp(mot_de_passe: str, hash_: str) -> bool:
    try:
        return pwd_context.verify(mot_de_passe, hash_)
    except Exception:
        return False


def creer_token(user: User) -> str:
    """Génère un token JWT signé contenant l'identifiant de l'utilisateur."""
    payload = {
        "sub": str(user.id),
        "exp": datetime.now(timezone.utc) + timedelta(minutes=JWT_EXPIRATION_MINUTES),
    }
    return jwt.encode(payload, SECRET_KEY, algorithm=JWT_ALGORITHME)


def obtenir_utilisateur_actuel(
    credentials: HTTPAuthorizationCredentials = Depends(bearer_scheme),
    db: Session = Depends(get_db),
) -> User:
    """Dépendance FastAPI : décode le Bearer token et renvoie l'utilisateur connecté."""
    if credentials is None:
        raise HTTPException(status_code=401, detail="Authentification requise")
    try:
        payload = jwt.decode(credentials.credentials, SECRET_KEY, algorithms=[JWT_ALGORITHME])
        user_id = int(payload.get("sub"))
    except (jwt.PyJWTError, ValueError, TypeError):
        raise HTTPException(status_code=401, detail="Token invalide ou expiré")
    user = db.query(User).filter(User.id == user_id).first()
    if user is None:
        raise HTTPException(status_code=401, detail="Utilisateur introuvable")
    return user


def profil_utilisateur(user: User) -> dict:
    """Sérialise un utilisateur (sans jamais exposer le hash du mot de passe)."""
    return {
        "id": user.id,
        "email": user.email,
        "username": user.username or user.email.split("@")[0],
        "ade_url": user.ade_ics_url or "",
        "gemini_key": user.gemini_api_key or "",
        "created_at": user.created_at.isoformat() if user.created_at else None,
    }


# ---------------------------------------------------------------------------
#  Schémas
# ---------------------------------------------------------------------------
class UtilisateurInscription(BaseModel):
    email: str
    password: str
    username: str = ""


class UtilisateurConnexion(BaseModel):
    email: str
    password: str


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

# Cache mémoire temporaire du flux iCal ADE (TTL 5 min) : évite de re-télécharger
# le flux à chaque requête, accélère les temps de réponse et réduit la charge du serveur ADE.
_CACHE_ICS = {"url": None, "contenu": None, "stocke_a": 0.0}
TTL_CACHE_ICS = 300  # secondes (5 minutes)


def obtenir_url_ade(user: User = None) -> Optional[str]:
    """Renvoie l'URL ADE de l'utilisateur connecté, sinon la variable d'environnement.

    Renvoie ``None`` au lieu de lever une exception quand aucune URL n'est
    configurée, afin que les routes puissent renvoyer une réponse propre
    (ex. tableau vide) plutôt qu'une erreur 500.
    """
    url = None
    if user is not None and user.ade_ics_url:
        url = user.ade_ics_url
    if not url:
        url = os.getenv("ADE_ICS_URL")
    return url or None


async def _telecharger_ics_async(url: str) -> str:
    """Téléchargement asynchrone du flux .ics via httpx.AsyncClient."""
    async with httpx.AsyncClient(timeout=20) as client:
        reponse = await client.get(url)
        reponse.raise_for_status()
        return reponse.text


def telecharger_ics(url: str) -> str:
    """Télécharge le fichier .ics depuis l'URL ADE fournie (avec cache mémoire de 5 min).

    Si le téléchargement échoue (timeout, 502, …), le contenu en cache s'il
    existe encore est renvoyé au lieu de lever une exception — cela rend
    l'application bien plus résiliente face aux problèmes temporaires du
    serveur ADE.
    """
    maintenant = time.monotonic()
    if _CACHE_ICS["url"] == url and (maintenant - _CACHE_ICS["stocke_a"]) < TTL_CACHE_ICS:
        return _CACHE_ICS["contenu"]

    try:
        contenu = asyncio.run(_telecharger_ics_async(url))
    except httpx.HTTPError:
        # Si le téléchargement échoue mais qu'on a un cache valide, on l'utilise.
        if _CACHE_ICS["url"] == url and _CACHE_ICS["contenu"]:
            return _CACHE_ICS["contenu"]
        raise HTTPException(
            status_code=502,
            detail="Impossible de télécharger le flux ADE et aucun cache n'est disponible.",
        )

    # On ne met en cache que les téléchargements réussis : un échec
    # laisse donc la version précédente disponible jusqu'à expiration.
    _CACHE_ICS.update(url=url, contenu=contenu, stocke_a=maintenant)
    return contenu


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
            description = str(evenement.description or "").strip()
            debut = evenement.begin.datetime
            fin = evenement.end.datetime

            # Gestion des événements "journée entière" (sans horaire).
            if not isinstance(debut, datetime):
                debut = datetime.combine(debut, datetime.min.time())
            if not isinstance(fin, datetime):
                fin = datetime.combine(fin, datetime.min.time())

            # Normalisation dans le fuseau horaire configuré.
            debut = debut.astimezone(ZONE) if debut.tzinfo else debut.replace(tzinfo=ZONE)
            fin = fin.astimezone(ZONE) if fin.tzinfo else fin.replace(tzinfo=ZONE)

            enseignants = extraire_enseignants(description)

            cours.append(
                {
                    "titre": titre,
                    "salle": str(evenement.location or "").strip(),
                    "enseignant": ", ".join(enseignants),
                    "enseignants": enseignants,
                    "debut": debut.strftime("%H:%M"),
                    "fin": fin.strftime("%H:%M"),
                    "debut_iso": debut.isoformat(),
                    "fin_iso": fin.isoformat(),
                    "date": debut.date().isoformat(),
                    # Analyse automatique (regex) pour le filtrage par groupe.
                    "groupes": detecter_groupes(titre, description),
                    "types": detecter_types(titre, description),
                    "matiere": extraire_matiere(titre),
                }
            )
        except Exception:
            # On ignore les événements malformés pour ne pas bloquer tout le flux.
            continue

    cours.sort(key=lambda c: c["debut_iso"])
    return cours


# ---------------------------------------------------------------------------
#  Analyse automatique du flux ADE (regex) : groupes, types, matières
# ---------------------------------------------------------------------------
RE_GROUPES = [
    re.compile(r"\b(?:TP|TD)[ ]*(\d+|[A-Z])\b", re.IGNORECASE),        # TP1, TP 1, TD2
    re.compile(r"\bGr(?:oupe)?s?[ ]+(?:\d+|[A-Z])\b", re.IGNORECASE),  # Groupe 1, Gr A
    re.compile(r"\bParcours[ ]+([A-Z0-9]+)\b", re.IGNORECASE),         # Parcours GEE
    re.compile(r"\bPromo[ ]+([A-Z]+)\b", re.IGNORECASE),               # Promo DK / FISE
    re.compile(r"\b(CP[12]|ING[12])\b", re.IGNORECASE),                # CP1, CP2, ING1, ING2
]
RE_TYPES = [
    re.compile(r"\b(Kh[oô]lle)\b", re.IGNORECASE),
    re.compile(r"\b(DS)\b"), re.compile(r"\b(CM)\b"), re.compile(r"\b(TD)\b"),
    re.compile(r"\b(TP)\b"), re.compile(r"\b(IE)\b"), re.compile(r"\b(CC)\b"),
    re.compile(r"\b(TRAVAUX[ ]+PRATIQUES)\b", re.IGNORECASE),
    re.compile(r"\b(TRAVAUX[ ]+DIRIGES)\b", re.IGNORECASE),
    re.compile(r"\b(COURS[ ]+MAGISTRAL)\b", re.IGNORECASE),
    re.compile(r"\b(PROJET|ATELIER|AMPHI)\b", re.IGNORECASE),
    re.compile(r"\b(Examen|Exam)\b", re.IGNORECASE),
    re.compile(r"\b(Contr[oô]le)\b", re.IGNORECASE),
    re.compile(r"\b(Oral)\b", re.IGNORECASE),
    re.compile(r"\b(PRESENTATION)\b", re.IGNORECASE),
    re.compile(r"\b(REUNION)\b", re.IGNORECASE),
    re.compile(r"\b(RENTREE)\b", re.IGNORECASE),
    re.compile(r"\b(OLYMPIADES)\b", re.IGNORECASE),
    re.compile(r"\b(TEDS)\b", re.IGNORECASE),
    re.compile(r"\b(WEC)\b", re.IGNORECASE),
]
# Marqueurs retirés d'un titre de cours pour ne garder que la matière.
RE_MARQUEURS_GROUPE = re.compile(
    r"\((?:TP|TD|CM|DS)[^)]*\)|Gr(?:oupe)?s?[ ]+(?:\d+|[A-Z])|"
    r"Parcours[ ]+[A-Z0-9]+|Promo[ ]+[A-Z]+|\b(?:CP[12]|ING[12])\b",
    re.IGNORECASE,
)
RE_NUMERO_TRAILING = re.compile(r"\s+\d+(?:\.\d+)*\s*$")


def extraire_matches(texte: str, regexes: list) -> list:
    """Applique des regex et renvoie les captures normalisées, uniques et triées."""
    resultats = []
    for rx in regexes:
        for m in rx.finditer(texte):
            valeur = (m.group(1) if m.lastindex else m.group(0)).strip().upper()
            if valeur and valeur not in resultats:
                resultats.append(valeur)
    return resultats


def detecter_groupes(titre: str, description: str) -> list:
    return extraire_matches(f"{titre} {description}", RE_GROUPES)


def detecter_types(titre: str, description: str) -> list:
    return extraire_matches(f"{titre} {description}", RE_TYPES)


def extraire_matiere(titre: str) -> str:
    """Nettoie un titre de cours pour en extraire la matière unique."""
    s = re.sub(r"\([^)]*\)", " ", titre)      # (ALGEBRE) -> supprimé
    s = s.split(" - ")[0]                      # "MECANIQUE DU SOLIDE - CINETIQUE" -> avant le tiret
    s = RE_MARQUEURS_GROUPE.sub(" ", s)        # groupes / types retirés
    s = RE_NUMERO_TRAILING.sub("", s)          # numéro de version en fin retiré
    s = re.sub(r"\s+", " ", s).strip()
    return s.upper()


# Lignes de "groupe" typiques d'un export ADE (CP2 PROMO DK, ING1 GEE - PROMO…).
RE_LIGNE_GROUPE = re.compile(
    r"^(?:[A-Z]{2,3}\s*\d|ING\d|CP[12]|TD|TP|CM|PROMO|PARCOURS|FISE|GEE|GI|DK)\b",
    re.IGNORECASE,
)
# Une ligne enseignant ressemble à "NOM Prénom" (nom en MAJUSCULES + prénom).
RE_LIGNE_PROF = re.compile(
    r"^[A-ZÀ-ÖØ-Þ][A-ZÀ-ÖØ-Þ'’\- ]{1,} +[A-ZÀ-ÖØ-Þ][A-ZÀ-ÖØ-Þa-zà-öø-ÿ'’\- ]+$"
)


def extraire_enseignants(description: str) -> list:
    """Extrait les noms des enseignants depuis la description d'un événement ADE.

    Format ADE typique : des lignes de groupes (CP2 PROMO DK, ING1 GEE - PROMO…)
    suivies d'une ligne "NOM Prénom" pour chaque enseignant, puis "(Exporté le:…)".
    """
    enseignants = []
    for ligne in description.splitlines():
        ligne = ligne.strip()
        if not ligne or ligne.startswith("("):
            continue
        if RE_LIGNE_GROUPE.match(ligne):
            continue
        if RE_LIGNE_PROF.match(ligne) and ligne not in enseignants:
            enseignants.append(ligne)
    return enseignants


def appliquer_filtres(cours: list, groupes: list = None, types: list = None, matieres: list = None) -> list:
    """Masque les séances qui ne correspondent à aucun filtre actif (groupes, types, matières).

    Chaque dimension est indépendante : une séance qui possède une valeur pour cette dimension
    (ex. un groupe TP) n'est gardée que si cette valeur est cochée. Une séance sans valeur pour
    la dimension (ex. un CM sans groupe) reste visible. Sans filtre actif, tout est affiché.
    """
    grp_actifs = {g.strip().lower() for g in (groupes or []) if g and g.strip()}
    typ_actifs = {t.strip().lower() for t in (types or []) if t and t.strip()}
    mat_actifs = {m.strip().lower() for m in (matieres or []) if m and m.strip()}
    gardes = []
    for c in cours:
        if grp_actifs:
            groupes_c = {g.lower() for g in c.get("groupes", [])}
            if groupes_c and not (groupes_c & grp_actifs):
                continue
        if typ_actifs:
            types_c = {t.lower() for t in c.get("types", [])}
            if types_c and not (types_c & typ_actifs):
                continue
        if mat_actifs:
            matiere_c = (c.get("matiere") or "").lower()
            if matiere_c and matiere_c not in mat_actifs:
                continue
        gardes.append(c)
    return gardes


def charger_cours(user: User = None) -> list:
    """Télécharge et parse tous les cours du flux ADE de l'utilisateur.

    Si aucune URL ADE n'est configurée, renvoie une liste vide au lieu de
    lever une exception — le frontend affichera alors « Aucun cours ».
    """
    url = obtenir_url_ade(user)
    if not url:
        return []
    return parser_cours(telecharger_ics(url))


def charger_cours_du_jour(user: User = None, jour: date = None) -> dict:
    """Cours d'une journée précise (aujourd'hui par défaut) filtrés depuis le flux ADE."""
    tous_les_cours = charger_cours(user)
    jour = jour or date.today()
    cours = [c for c in tous_les_cours if c["date"] == jour.isoformat()]
    return {"date": jour.isoformat(), "nb_cours": len(cours), "cours": cours}


def charger_cours_semaine(user: User = None, ref: date = None) -> dict:
    """Cours du lundi au vendredi de la semaine contenant `ref` (aujourd'hui par défaut)."""
    tous_les_cours = charger_cours(user)
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


# ---------------------------------------------------------------------------
#  Authentification : inscription, connexion, profil
# ---------------------------------------------------------------------------
@app.post("/api/auth/register")
def inscription(data: UtilisateurInscription, db: Session = Depends(get_db)):
    """Inscription d'un nouvel utilisateur : renvoie un token JWT + son profil."""
    email = data.email.strip().lower()
    if not re.match(r"^[^@\s]+@[^@\s]+\.[^@\s]+$", email):
        raise HTTPException(status_code=400, detail="Adresse e-mail invalide")
    if len(data.password) < 8:
        raise HTTPException(status_code=400, detail="Le mot de passe doit contenir au moins 8 caractères")
    if db.query(User).filter(User.email == email).first():
        raise HTTPException(status_code=409, detail="Un compte existe déjà avec cet e-mail")

    user = User(
        email=email,
        password_hash=hacher_mdp(data.password),
        username=data.username.strip(),
    )
    db.add(user)
    db.commit()
    db.refresh(user)
    return {"token": creer_token(user), "user": profil_utilisateur(user)}


@app.post("/api/auth/login")
def connexion(data: UtilisateurConnexion, db: Session = Depends(get_db)):
    """Connexion : vérifie les identifiants et renvoie un token JWT."""
    email = data.email.strip().lower()
    user = db.query(User).filter(User.email == email).first()
    if user is None or not verifier_mdp(data.password, user.password_hash):
        raise HTTPException(status_code=401, detail="E-mail ou mot de passe incorrect")
    return {"token": creer_token(user), "user": profil_utilisateur(user)}


@app.get("/api/auth/me")
def profil_actuel(user: User = Depends(obtenir_utilisateur_actuel)):
    """Renvoie le profil de l'utilisateur connecté."""
    return profil_utilisateur(user)


@app.get("/api/cours-du-jour")
def get_cours_du_jour(
    date: Optional[str] = None,
    groupes: Optional[list[str]] = Query(default=None),
    types: Optional[list[str]] = Query(default=None),
    matieres: Optional[list[str]] = Query(default=None),
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    resultat = charger_cours_du_jour(user, parser_date(date))
    resultat["cours"] = appliquer_filtres(resultat["cours"], groupes, types, matieres)
    resultat["nb_cours"] = len(resultat["cours"])
    return resultat


@app.get("/api/cours-semaine")
def get_cours_semaine(
    date: Optional[str] = None,
    groupes: Optional[list[str]] = Query(default=None),
    types: Optional[list[str]] = Query(default=None),
    matieres: Optional[list[str]] = Query(default=None),
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    resultat = charger_cours_semaine(user, parser_date(date))
    resultat["cours"] = appliquer_filtres(resultat["cours"], groupes, types, matieres)
    return resultat


# ---------------------------------------------------------------------------
#  Devoirs (SQLite via SQLAlchemy) — isolés par utilisateur
# ---------------------------------------------------------------------------
@app.get("/api/devoirs")
def lister_devoirs(db: Session = Depends(get_db), user: User = Depends(obtenir_utilisateur_actuel)):
    return db.query(Devoir).filter(Devoir.user_id == user.id).all()


@app.get("/api/devoirs/export")
def exporter_devoirs(db: Session = Depends(get_db), user: User = Depends(obtenir_utilisateur_actuel)):
    """Exporte tous les devoirs de l'utilisateur au format JSON (sans les identifiants internes)."""
    devoirs = db.query(Devoir).filter(Devoir.user_id == user.id).all()
    return [
        {
            "titre": d.titre,
            "matiere": d.matiere,
            "echeance": d.echeance,
            "type": d.type,
            "statut": d.statut,
        }
        for d in devoirs
    ]


@app.post("/api/devoirs/import")
def importer_devoirs(
    devoirs: list[DevoirCreate],
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    """Importe une liste de devoirs au format JSON et retourne le nombre ajouté."""
    importes = 0
    for item in devoirs:
        db.add(
            Devoir(
                titre=item.titre,
                matiere=item.matiere,
                echeance=item.echeance,
                type=item.type,
                statut=item.statut,
                user_id=user.id,
            )
        )
        importes += 1
    db.commit()
    return {"detail": f"{importes} devoir(s) importé(s)", "nb_importes": importes}


@app.post("/api/devoirs")
def creer_devoir(devoir: DevoirCreate, db: Session = Depends(get_db), user: User = Depends(obtenir_utilisateur_actuel)):
    nouveau = Devoir(
        titre=devoir.titre,
        matiere=devoir.matiere,
        echeance=devoir.echeance,
        type=devoir.type,
        statut=devoir.statut,
        user_id=user.id,
    )
    db.add(nouveau)
    db.commit()
    db.refresh(nouveau)
    return nouveau


@app.patch("/api/devoirs/{devoir_id}")
def maj_devoir(
    devoir_id: int,
    mise_a_jour: DevoirUpdate,
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    devoir = db.query(Devoir).filter(Devoir.id == devoir_id, Devoir.user_id == user.id).first()
    if devoir is None:
        raise HTTPException(status_code=404, detail="Devoir introuvable")
    if mise_a_jour.statut is not None:
        devoir.statut = mise_a_jour.statut
    db.commit()
    db.refresh(devoir)
    return devoir


@app.delete("/api/devoirs/{devoir_id}")
def supprimer_devoir(devoir_id: int, db: Session = Depends(get_db), user: User = Depends(obtenir_utilisateur_actuel)):
    devoir = db.query(Devoir).filter(Devoir.id == devoir_id, Devoir.user_id == user.id).first()
    if devoir is None:
        raise HTTPException(status_code=404, detail="Devoir introuvable")
    db.delete(devoir)
    db.commit()
    return {"detail": "Devoir supprimé"}


# ---------------------------------------------------------------------------
#  Configuration (propre à chaque utilisateur)
# ---------------------------------------------------------------------------
@app.get("/api/config")
def get_config(db: Session = Depends(get_db), user: User = Depends(obtenir_utilisateur_actuel)):
    return {
        "username": user.username or user.email.split("@")[0],
        "ade_url": user.ade_ics_url or "",
        "gemini_key": user.gemini_api_key or "",
    }


@app.post("/api/config")
def save_config(data: ConfigUpdate, db: Session = Depends(get_db), user: User = Depends(obtenir_utilisateur_actuel)):
    user.username = data.username.strip()
    user.ade_ics_url = data.ade_url.strip()
    user.gemini_api_key = data.gemini_key.strip()
    db.commit()
    return {"detail": "Configuration enregistrée"}


@app.get("/api/config/categories")
def analyser_categories(db: Session = Depends(get_db), user: User = Depends(obtenir_utilisateur_actuel)):
    """Télécharge le flux ADE de l'utilisateur et en extrait groupes, types et matières."""
    tous = charger_cours(user)
    groupes, types, matieres = [], [], []
    for c in tous:
        for g in c.get("groupes", []):
            if g and g not in groupes:
                groupes.append(g)
        for t in c.get("types", []):
            if t and t not in types:
                types.append(t)
        m = c.get("matiere", "")
        if m and m not in matieres:
            matieres.append(m)
    return {"groupes": sorted(groupes), "types": sorted(types), "matieres": sorted(matieres)}


# ---------------------------------------------------------------------------
#  Plan de révision IA (Gemini ou simulation)
# ---------------------------------------------------------------------------
def _decrire_cours(c: dict) -> str:
    """Résume un cours pour le prompt IA (titre, horaire, salle, professeur)."""
    infos = f"{c['titre']} ({c['debut']} -> {c['fin']}"
    if c.get("salle"):
        infos += f", salle {c['salle']}"
    if c.get("enseignant"):
        infos += f", professeur : {c['enseignant']}"
    return infos + ")"


def construire_prompt(cours: list, devoirs: list) -> str:
    cours_txt = "\n".join(f"- {_decrire_cours(c)}" for c in cours) or "- Aucun cours aujourd'hui."

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
def generer_plan_revision(
    groupes: Optional[list[str]] = Query(default=None),
    types: Optional[list[str]] = Query(default=None),
    matieres: Optional[list[str]] = Query(default=None),
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    """Génère un plan de révision du soir (Gemini si configuré, sinon simulation)."""
    cours = appliquer_filtres(charger_cours_du_jour(user)["cours"], groupes, types, matieres)
    devoirs = db.query(Devoir).filter(Devoir.user_id == user.id).all()
    prompt = construire_prompt(cours, devoirs)

    api_key = user.gemini_api_key or os.getenv("GEMINI_API_KEY")

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


# ---------------------------------------------------------------------------
#  Frontend statique (servi par FastAPI si le dossier existe)
#  => Permet un accès unique à l'app sur http://localhost:8000 (dont Docker).
#  Les routes /api/* déclarées ci-dessus restent prioritaires.
# ---------------------------------------------------------------------------
_FRONTEND_DIR = Path(__file__).resolve().parent.parent / "frontend"
if _FRONTEND_DIR.is_dir():
    app.mount("/", StaticFiles(directory=_FRONTEND_DIR, html=True), name="frontend")
