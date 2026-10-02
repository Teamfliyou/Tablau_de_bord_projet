import logging
import os
import secrets
from zoneinfo import ZoneInfo

from dotenv import load_dotenv

load_dotenv()

logger = logging.getLogger(__name__)

APP_TIMEZONE = os.getenv("APP_TIMEZONE", "Europe/Paris")
ZONE = ZoneInfo(APP_TIMEZONE)

ALLOWED_ORIGINS = [
    origin.strip()
    for origin in os.getenv("ALLOWED_ORIGINS", "*").split(",")
    if origin.strip()
]

SECRET_KEY = os.getenv("SECRET_KEY")
if not SECRET_KEY:
    SECRET_KEY = secrets.token_urlsafe(48)
    logger.warning(
        "SECRET_KEY n'est pas définie : une clé temporaire aléatoire est utilisée. "
        "Les sessions seront invalidées au prochain redémarrage."
    )

JWT_ALGORITHM = "HS256"
JWT_EXPIRATION_MINUTES = int(os.getenv("JWT_EXPIRATION_MINUTES", str(60 * 24)))
ADE_ALLOWED_HOSTS = [
    host.strip().lower().lstrip(".")
    for host in os.getenv("ADE_ALLOWED_HOSTS", "").split(",")
    if host.strip()
]
GEMINI_MODEL = os.getenv("GEMINI_MODEL", "gemini-3.8-flash")
ICS_CACHE_TTL_SECONDS = int(os.getenv("ICS_CACHE_TTL_SECONDS", "300"))
ICS_CACHE_MAX_ENTRIES = int(os.getenv("ICS_CACHE_MAX_ENTRIES", "64"))
