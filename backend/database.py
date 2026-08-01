import os

from sqlalchemy import Column, Integer, String, create_engine, text
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

# Surchargeable via l'environnement (ex. Docker) ; sinon base SQLite locale.
DATABASE_URL = os.getenv("DATABASE_URL", "sqlite:///./app.db")

engine = create_engine(
    DATABASE_URL,
    connect_args={"check_same_thread": False},
)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
Base = declarative_base()


class Devoir(Base):
    __tablename__ = "devoirs"

    id = Column(Integer, primary_key=True, index=True)
    titre = Column(String, nullable=False)
    matiere = Column(String, default="")
    echeance = Column(String, default="")  # format YYYY-MM-DD
    type = Column(String, default="devoir")  # devoir | ie | ds | exam
    # a_faire | en_cours | termine
    statut = Column(String, default="a_faire")


class Config(Base):
    __tablename__ = "config"

    id = Column(Integer, primary_key=True)
    username = Column(String, default="")
    ade_url = Column(String, default="")
    gemini_key = Column(String, default="")


Base.metadata.create_all(bind=engine)


def _migrer():
    """Ajoute les nouvelles colonnes à une base existante sans perdre les données."""
    with engine.begin() as conn:
        colonnes = {row[1] for row in conn.execute(text("PRAGMA table_info(devoirs)"))}
        ajouts = {
            "matiere": "VARCHAR DEFAULT ''",
            "echeance": "VARCHAR DEFAULT ''",
            "type": "VARCHAR DEFAULT 'devoir'",
        }
        for nom, ddl in ajouts.items():
            if nom not in colonnes:
                conn.execute(text(f"ALTER TABLE devoirs ADD COLUMN {nom} {ddl}"))


_migrer()
