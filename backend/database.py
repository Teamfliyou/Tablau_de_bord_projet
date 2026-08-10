import os

from sqlalchemy import Column, Integer, String, create_engine, text
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

# Surchargeable via l'environnement (ex. Docker ou PaaS Render) ; sinon base SQLite locale.
DATABASE_URL = os.getenv("DATABASE_URL", "sqlite:///./app.db")

# Render fournit souvent une URL au format historique postgres://
# (non reconnu par psycopg2) : on la normalise en postgresql://.
if DATABASE_URL.startswith("postgres://"):
    DATABASE_URL = DATABASE_URL.replace("postgres://", "postgresql://", 1)

# connect_args={"check_same_thread": False} est spécifique à SQLite :
# on ne l'ajoute que pour une base SQLite, jamais pour PostgreSQL.
_engine_kwargs = {}
if DATABASE_URL.startswith("sqlite"):
    _engine_kwargs["connect_args"] = {"check_same_thread": False}

engine = create_engine(DATABASE_URL, **_engine_kwargs)
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
    """Ajoute les nouvelles colonnes à une base SQLite existante sans perdre les données."""
    # PRAGMA table_info est spécifique à SQLite : la migration ne concerne que SQLite.
    # Pour PostgreSQL, le schéma est géré par Base.metadata.create_all() ci-dessus.
    if not DATABASE_URL.startswith("sqlite"):
        return
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
