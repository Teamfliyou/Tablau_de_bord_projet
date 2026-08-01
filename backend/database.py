"""Couche d'accès aux données (SQLite via SQLModel).

Ce module centralise :
  - le moteur SQLAlchemy / SQLModel pointant vers la base SQLite ;
  - la définition du modèle `Devoir` (table `devoir`) ;
  - la fonction de création des tables au démarrage ;
  - la dépendance FastAPI `get_session` pour injecter une session dans les routes.

La chaîne de connexion est lue depuis la variable d'environnement
`DATABASE_URL` (voir `.env.example`).
"""

import os
from typing import Iterator, Optional

from dotenv import load_dotenv
from sqlmodel import Field, Session, SQLModel, create_engine

load_dotenv()

DATABASE_URL = os.getenv("DATABASE_URL", "sqlite:///./tableau_bord.db")

# `check_same_thread=False` : indispensable pour SQLite en mode web
# (les sessions peuvent être utilisées depuis des threads différents).
engine = create_engine(
    DATABASE_URL,
    echo=False,
    connect_args={"check_same_thread": False}
    if DATABASE_URL.startswith("sqlite")
    else {},
)


def create_db_and_tables() -> None:
    """Crée les tables manquantes dans la base (idempotent)."""
    SQLModel.metadata.create_all(engine)


def get_session() -> Iterator[Session]:
    """Dépendance FastAPI : fournit une session SQLModel par requête."""
    with Session(engine) as session:
        yield session


class Devoir(SQLModel, table=True):
    """Un devoir à rendre, stocké en base.

    `table=True` indique que ce modèle correspond à une vraie table SQLite.
    """

    id: Optional[int] = Field(default=None, primary_key=True)
    titre: str = Field(min_length=1, description="Intitulé du devoir")
    matiere: str = Field(default="", description="Matière associée")
    echeance: Optional[str] = Field(
        default=None, description="Échéance au format YYYY-MM-DD"
    )
    fait: bool = Field(default=False, description="Devoir terminé ou non")


class DevoirUpdate(SQLModel):
    """Schéma de mise à jour partielle d'un devoir (PATCH).

    Tous les champs sont optionnels : seuls ceux fournis sont modifiés.
    """

    titre: Optional[str] = None
    matiere: Optional[str] = None
    echeance: Optional[str] = None
    fait: Optional[bool] = None
