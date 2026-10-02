from datetime import date
from typing import Literal, Optional

from pydantic import BaseModel, Field, field_validator


class UtilisateurInscription(BaseModel):
    email: str = Field(min_length=3, max_length=254)
    password: str = Field(min_length=8, max_length=128)
    username: str = Field(default="", max_length=80)


class UtilisateurConnexion(BaseModel):
    email: str = Field(min_length=3, max_length=254)
    password: str = Field(min_length=1, max_length=128)


class DevoirCreate(BaseModel):
    titre: str = Field(min_length=1, max_length=200)
    matiere: str = Field(default="", max_length=120)
    echeance: str = ""
    type: Literal["devoir", "ie", "ds", "exam"] = "devoir"
    statut: Literal["a_faire", "en_cours", "termine"] = "a_faire"

    @field_validator("titre", "matiere")
    @classmethod
    def nettoyer_texte(cls, valeur: str) -> str:
        return valeur.strip()

    @field_validator("echeance")
    @classmethod
    def valider_echeance(cls, valeur: str) -> str:
        valeur = valeur.strip()
        if valeur:
            try:
                date.fromisoformat(valeur)
            except ValueError as exc:
                raise ValueError("L'échéance doit être au format YYYY-MM-DD") from exc
        return valeur


class DevoirUpdate(BaseModel):
    statut: Optional[Literal["a_faire", "en_cours", "termine"]] = None


class ConfigUpdate(BaseModel):
    username: str = Field(default="", max_length=80)
    ade_url: str = Field(default="", max_length=2048)
    gemini_key: Optional[str] = Field(default=None, max_length=512)
    remove_gemini_key: bool = False
