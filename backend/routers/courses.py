from typing import Optional

from fastapi import APIRouter, Depends, Query

from backend.database import User
from backend.dependencies import obtenir_utilisateur_actuel
from backend.services.ade import (
    appliquer_filtres,
    charger_cours_du_jour,
    charger_cours_semaine,
    parser_date,
)

router = APIRouter(prefix="/api", tags=["cours"])


@router.get("/cours-du-jour")
def get_cours_du_jour(
    date: Optional[str] = None,
    groupes: Optional[list[str]] = Query(default=None),
    types: Optional[list[str]] = Query(default=None),
    matieres: Optional[list[str]] = Query(default=None),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    resultat = charger_cours_du_jour(user, parser_date(date))
    resultat["cours"] = appliquer_filtres(resultat["cours"], groupes, types, matieres)
    resultat["nb_cours"] = len(resultat["cours"])
    return resultat


@router.get("/cours-semaine")
def get_cours_semaine(
    date: Optional[str] = None,
    groupes: Optional[list[str]] = Query(default=None),
    types: Optional[list[str]] = Query(default=None),
    matieres: Optional[list[str]] = Query(default=None),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    resultat = charger_cours_semaine(user, parser_date(date))
    resultat["cours"] = appliquer_filtres(resultat["cours"], groupes, types, matieres)
    return resultat
