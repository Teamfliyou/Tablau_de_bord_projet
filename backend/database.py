import os
from datetime import datetime

from sqlalchemy import Column, DateTime, ForeignKey, Integer, String, create_engine, text
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import relationship, sessionmaker

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


class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True, index=True)
    email = Column(String, unique=True, index=True, nullable=False)
    password_hash = Column(String, nullable=False)
    username = Column(String, default="")
    ade_ics_url = Column(String, default="")
    gemini_api_key = Column(String, default="")
    created_at = Column(DateTime, default=datetime.utcnow)

    devoirs = relationship("Devoir", back_populates="user")


class Devoir(Base):
    __tablename__ = "devoirs"

    id = Column(Integer, primary_key=True, index=True)
    user_id = Column(Integer, ForeignKey("users.id"), index=True)
    titre = Column(String, nullable=False)
    matiere = Column(String, default="")
    echeance = Column(String, default="")  # format YYYY-MM-DD
    type = Column(String, default="devoir")  # devoir | ie | ds | exam
    # a_faire | en_cours | termine
    statut = Column(String, default="a_faire")

    user = relationship("User", back_populates="devoirs")


class Config(Base):
    """Table historique conservée pour la rétrocompatibilité des anciennes bases.

    Depuis le passage au multi-utilisateurs, la configuration (lien ADE, clé Gemini,
    pseudo) est stockée directement sur le modèle User. Cette table n'est plus utilisée
    par l'API mais reste présente pour ne pas casser les anciennes installations.
    """

    __tablename__ = "config"

    id = Column(Integer, primary_key=True)
    user_id = Column(Integer, ForeignKey("users.id"), index=True)
    username = Column(String, default="")
    ade_url = Column(String, default="")
    gemini_key = Column(String, default="")


Base.metadata.create_all(bind=engine)


def _migrer():
    """Met à niveau une base SQLite existante sans perdre les données.

    PRAGMA table_info est spécifique à SQLite : la migration ne concerne que SQLite.
    Pour PostgreSQL, le schéma est entièrement géré par Base.metadata.create_all().
    """
    if not DATABASE_URL.startswith("sqlite"):
        return
    with engine.begin() as conn:
        # Devoirs : ajout de la clé étrangère user_id (multi-utilisateurs).
        colonnes = {row[1] for row in conn.execute(text("PRAGMA table_info(devoirs)"))}
        ajouts = {
            "matiere": "VARCHAR DEFAULT ''",
            "echeance": "VARCHAR DEFAULT ''",
            "type": "VARCHAR DEFAULT 'devoir'",
            "user_id": "INTEGER REFERENCES users (id)",
        }
        for nom, ddl in ajouts.items():
            if nom not in colonnes:
                conn.execute(text(f"ALTER TABLE devoirs ADD COLUMN {nom} {ddl}"))
        # Config historique : ajout de la clé étrangère user_id.
        try:
            colonnes_config = {row[1] for row in conn.execute(text("PRAGMA table_info(config)"))}
            if "user_id" not in colonnes_config:
                conn.execute(text("ALTER TABLE config ADD COLUMN user_id INTEGER REFERENCES users (id)"))
        except Exception:
            # Table config absente sur une base très ancienne : sans gravité.
            pass


_migrer()
