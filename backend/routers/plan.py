import os
from typing import Optional

from fastapi import APIRouter, Depends, Query
from sqlalchemy.orm import Session

from backend.database import Devoir, User
from backend.dependencies import get_db, obtenir_utilisateur_actuel
from backend.services.ade import appliquer_filtres, charger_cours_du_jour
from backend.services.planner import generer_plan

router = APIRouter(prefix="/api", tags=["planification"])


@router.post("/plan-revision")
def generer_plan_revision(
    groupes: Optional[list[str]] = Query(default=None),
    types: Optional[list[str]] = Query(default=None),
    matieres: Optional[list[str]] = Query(default=None),
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    cours = appliquer_filtres(
        charger_cours_du_jour(user)["cours"],
        groupes,
        types,
        matieres,
    )
    devoirs = db.query(Devoir).filter(Devoir.user_id == user.id).all()
    api_key = user.gemini_api_key or os.getenv("GEMINI_API_KEY")
    return generer_plan(cours, devoirs, api_key)
