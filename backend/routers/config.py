from fastapi import APIRouter, Depends
from sqlalchemy.orm import Session

from backend.database import User
from backend.dependencies import get_db, obtenir_utilisateur_actuel
from backend.schemas import ConfigUpdate
from backend.services.ade import charger_cours, valider_url_ade

router = APIRouter(prefix="/api/config", tags=["configuration"])


@router.get("")
def get_config(
    user: User = Depends(obtenir_utilisateur_actuel),
):
    return {
        "username": user.username or user.email.split("@")[0],
        "ade_url": user.ade_ics_url or "",
        "gemini_key": "",
        "gemini_configured": bool(user.gemini_api_key),
    }


@router.post("")
def save_config(
    data: ConfigUpdate,
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    user.username = data.username.strip()
    ade_url = data.ade_url.strip()
    if ade_url:
        valider_url_ade(ade_url)
    user.ade_ics_url = ade_url

    if data.remove_gemini_key:
        user.gemini_api_key = ""
    elif data.gemini_key is not None and data.gemini_key.strip():
        user.gemini_api_key = data.gemini_key.strip()

    db.commit()
    return {
        "detail": "Configuration enregistrée",
        "gemini_configured": bool(user.gemini_api_key),
    }


@router.get("/categories")
def analyser_categories(
    user: User = Depends(obtenir_utilisateur_actuel),
):
    tous = charger_cours(user)
    groupes: set[str] = set()
    types: set[str] = set()
    matieres: set[str] = set()

    for cours in tous:
        groupes.update(g for g in cours.get("groupes", []) if g)
        types.update(t for t in cours.get("types", []) if t)
        if cours.get("matiere"):
            matieres.add(cours["matiere"])

    return {
        "groupes": sorted(groupes),
        "types": sorted(types),
        "matieres": sorted(matieres),
    }
