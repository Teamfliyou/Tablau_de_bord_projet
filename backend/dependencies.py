from datetime import datetime, timedelta, timezone

import jwt
from fastapi import Depends, HTTPException
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
from passlib.context import CryptContext
from sqlalchemy.orm import Session

from backend.database import SessionLocal, User
from backend.settings import JWT_ALGORITHM, JWT_EXPIRATION_MINUTES, SECRET_KEY

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")
bearer_scheme = HTTPBearer(auto_error=False)


def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()


def hacher_mdp(mot_de_passe: str) -> str:
    return pwd_context.hash(mot_de_passe)


def verifier_mdp(mot_de_passe: str, hash_: str) -> bool:
    try:
        return pwd_context.verify(mot_de_passe, hash_)
    except Exception:
        return False


def creer_token(user: User) -> str:
    payload = {
        "sub": str(user.id),
        "exp": datetime.now(timezone.utc) + timedelta(minutes=JWT_EXPIRATION_MINUTES),
    }
    return jwt.encode(payload, SECRET_KEY, algorithm=JWT_ALGORITHM)


def obtenir_utilisateur_actuel(
    credentials: HTTPAuthorizationCredentials = Depends(bearer_scheme),
    db: Session = Depends(get_db),
) -> User:
    if credentials is None:
        raise HTTPException(status_code=401, detail="Authentification requise")
    try:
        payload = jwt.decode(credentials.credentials, SECRET_KEY, algorithms=[JWT_ALGORITHM])
        user_id = int(payload.get("sub"))
    except (jwt.PyJWTError, ValueError, TypeError):
        raise HTTPException(status_code=401, detail="Token invalide ou expiré")

    user = db.query(User).filter(User.id == user_id).first()
    if user is None:
        raise HTTPException(status_code=401, detail="Utilisateur introuvable")
    return user


def profil_utilisateur(user: User) -> dict:
    return {
        "id": user.id,
        "email": user.email,
        "username": user.username or user.email.split("@")[0],
        "ade_url": user.ade_ics_url or "",
        "gemini_configured": bool(user.gemini_api_key),
        "created_at": user.created_at.isoformat() if user.created_at else None,
    }
