import re

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session

from backend.database import User
from backend.dependencies import (
    creer_token,
    get_db,
    hacher_mdp,
    obtenir_utilisateur_actuel,
    profil_utilisateur,
    verifier_mdp,
)
from backend.schemas import UtilisateurConnexion, UtilisateurInscription

router = APIRouter(prefix="/api/auth", tags=["auth"])


@router.post("/register")
def inscription(data: UtilisateurInscription, db: Session = Depends(get_db)):
    email = data.email.strip().lower()
    if not re.match(r"^[^@\s]+@[^@\s]+\.[^@\s]+$", email):
        raise HTTPException(status_code=400, detail="Adresse e-mail invalide")
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


@router.post("/login")
def connexion(data: UtilisateurConnexion, db: Session = Depends(get_db)):
    email = data.email.strip().lower()
    user = db.query(User).filter(User.email == email).first()
    if user is None or not verifier_mdp(data.password, user.password_hash):
        raise HTTPException(status_code=401, detail="E-mail ou mot de passe incorrect")
    return {"token": creer_token(user), "user": profil_utilisateur(user)}


@router.get("/me")
def profil_actuel(user: User = Depends(obtenir_utilisateur_actuel)):
    return profil_utilisateur(user)
