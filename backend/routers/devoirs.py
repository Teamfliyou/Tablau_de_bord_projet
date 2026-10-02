from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session

from backend.database import Devoir, User
from backend.dependencies import get_db, obtenir_utilisateur_actuel
from backend.schemas import DevoirCreate, DevoirUpdate

router = APIRouter(prefix="/api/devoirs", tags=["devoirs"])


@router.get("")
def lister_devoirs(
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    return db.query(Devoir).filter(Devoir.user_id == user.id).all()


@router.get("/export")
def exporter_devoirs(
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    devoirs = db.query(Devoir).filter(Devoir.user_id == user.id).all()
    return [
        {
            "titre": devoir.titre,
            "matiere": devoir.matiere,
            "echeance": devoir.echeance,
            "type": devoir.type,
            "statut": devoir.statut,
        }
        for devoir in devoirs
    ]


@router.post("/import")
def importer_devoirs(
    devoirs: list[DevoirCreate],
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    if len(devoirs) > 500:
        raise HTTPException(status_code=400, detail="Import limité à 500 devoirs par fichier.")

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
    db.commit()
    return {"detail": f"{len(devoirs)} devoir(s) importé(s)", "nb_importes": len(devoirs)}


@router.post("")
def creer_devoir(
    devoir: DevoirCreate,
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
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


@router.patch("/{devoir_id}")
def maj_devoir(
    devoir_id: int,
    mise_a_jour: DevoirUpdate,
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    devoir = db.query(Devoir).filter(
        Devoir.id == devoir_id,
        Devoir.user_id == user.id,
    ).first()
    if devoir is None:
        raise HTTPException(status_code=404, detail="Devoir introuvable")

    if mise_a_jour.statut is not None:
        devoir.statut = mise_a_jour.statut
    db.commit()
    db.refresh(devoir)
    return devoir


@router.delete("/{devoir_id}")
def supprimer_devoir(
    devoir_id: int,
    db: Session = Depends(get_db),
    user: User = Depends(obtenir_utilisateur_actuel),
):
    devoir = db.query(Devoir).filter(
        Devoir.id == devoir_id,
        Devoir.user_id == user.id,
    ).first()
    if devoir is None:
        raise HTTPException(status_code=404, detail="Devoir introuvable")

    db.delete(devoir)
    db.commit()
    return {"detail": "Devoir supprimé"}
