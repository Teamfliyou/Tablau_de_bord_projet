import ipaddress
import socket
import time
from datetime import date, datetime, timedelta
from typing import Optional
from urllib.parse import urlparse

import httpx
from fastapi import HTTPException
from ics import Calendar

from backend.database import User
from backend.settings import (
    ADE_ALLOWED_HOSTS,
    ICS_CACHE_MAX_ENTRIES,
    ICS_CACHE_TTL_SECONDS,
    ZONE,
)

_CACHE_ICS: dict[str, tuple[str, float]] = {}


def obtenir_url_ade(user: User | None = None) -> Optional[str]:
    import os

    if user is not None and user.ade_ics_url:
        return user.ade_ics_url
    return os.getenv("ADE_ICS_URL") or None


def _adresse_interdite(adresse: str) -> bool:
    try:
        ip = ipaddress.ip_address(adresse)
    except ValueError:
        return True
    return (
        ip.is_private
        or ip.is_loopback
        or ip.is_link_local
        or ip.is_multicast
        or ip.is_reserved
        or ip.is_unspecified
    )


def valider_url_ade(url: str) -> str:
    url = (url or "").strip()
    if not url:
        raise HTTPException(status_code=400, detail="Lien ADE vide.")

    parsed = urlparse(url)
    if parsed.scheme.lower() != "https" or not parsed.hostname:
        raise HTTPException(status_code=400, detail="Le lien ADE doit être une URL HTTPS valide.")

    hostname = parsed.hostname.lower().rstrip(".")
    if hostname == "localhost" or hostname.endswith(".localhost"):
        raise HTTPException(status_code=400, detail="Hôte ADE non autorisé.")

    if ADE_ALLOWED_HOSTS and not any(
        hostname == host or hostname.endswith("." + host)
        for host in ADE_ALLOWED_HOSTS
    ):
        raise HTTPException(status_code=400, detail="Ce domaine ADE n'est pas autorisé.")

    try:
        infos = socket.getaddrinfo(hostname, parsed.port or 443, type=socket.SOCK_STREAM)
    except socket.gaierror as exc:
        raise HTTPException(status_code=400, detail="Le domaine ADE est introuvable.") from exc

    adresses = {info[4][0] for info in infos}
    if not adresses or any(_adresse_interdite(adresse) for adresse in adresses):
        raise HTTPException(status_code=400, detail="Adresse réseau ADE non autorisée.")

    return url


def _nettoyer_cache(now: float) -> None:
    expires_before = now - ICS_CACHE_TTL_SECONDS
    perimees = [url for url, (_, stored_at) in _CACHE_ICS.items() if stored_at < expires_before]
    for url in perimees:
        _CACHE_ICS.pop(url, None)

    if len(_CACHE_ICS) > ICS_CACHE_MAX_ENTRIES:
        plus_anciennes = sorted(_CACHE_ICS.items(), key=lambda item: item[1][1])
        for url, _ in plus_anciennes[: len(_CACHE_ICS) - ICS_CACHE_MAX_ENTRIES]:
            _CACHE_ICS.pop(url, None)


def telecharger_ics(url: str) -> str:
    url = valider_url_ade(url)
    now = time.monotonic()
    _nettoyer_cache(now)

    cached = _CACHE_ICS.get(url)
    if cached and now - cached[1] < ICS_CACHE_TTL_SECONDS:
        return cached[0]

    try:
        with httpx.Client(timeout=20, follow_redirects=False) as client:
            response = client.get(url)
            if 300 <= response.status_code < 400:
                raise HTTPException(
                    status_code=502,
                    detail="Les redirections du flux ADE ne sont pas autorisées.",
                )
            response.raise_for_status()
            contenu = response.text
    except HTTPException:
        raise
    except httpx.HTTPError as exc:
        if cached:
            return cached[0]
        raise HTTPException(
            status_code=502,
            detail="Impossible de télécharger le flux ADE et aucun cache n'est disponible.",
        ) from exc

    _CACHE_ICS[url] = (contenu, now)
    _nettoyer_cache(now)
    return contenu


import re

RE_GROUPES = [
    re.compile(r"\b(?:TP|TD)[ ]*(\d+|[A-Z])\b", re.IGNORECASE),
    re.compile(r"\bGr(?:oupe)?s?[ ]+(?:\d+|[A-Z])\b", re.IGNORECASE),
    re.compile(r"\bParcours[ ]+([A-Z0-9]+)\b", re.IGNORECASE),
    re.compile(r"\bPromo[ ]+([A-Z]+)\b", re.IGNORECASE),
    re.compile(r"\b(CP[12]|ING[12])\b", re.IGNORECASE),
]
RE_TYPES = [
    re.compile(r"\b(Kh[oô]lle)\b", re.IGNORECASE),
    re.compile(r"\b(DS)\b"),
    re.compile(r"\b(CM)\b"),
    re.compile(r"\b(TD)\b"),
    re.compile(r"\b(TP)\b"),
    re.compile(r"\b(IE)\b"),
    re.compile(r"\b(CC)\b"),
    re.compile(r"\b(TRAVAUX[ ]+PRATIQUES)\b", re.IGNORECASE),
    re.compile(r"\b(TRAVAUX[ ]+DIRIGES)\b", re.IGNORECASE),
    re.compile(r"\b(COURS[ ]+MAGISTRAL)\b", re.IGNORECASE),
    re.compile(r"\b(PROJET|ATELIER|AMPHI)\b", re.IGNORECASE),
    re.compile(r"\b(Examen|Exam)\b", re.IGNORECASE),
    re.compile(r"\b(Contr[oô]le)\b", re.IGNORECASE),
    re.compile(r"\b(Oral)\b", re.IGNORECASE),
    re.compile(r"\b(PRESENTATION)\b", re.IGNORECASE),
    re.compile(r"\b(REUNION)\b", re.IGNORECASE),
    re.compile(r"\b(RENTREE)\b", re.IGNORECASE),
    re.compile(r"\b(OLYMPIADES)\b", re.IGNORECASE),
    re.compile(r"\b(TEDS)\b", re.IGNORECASE),
    re.compile(r"\b(WEC)\b", re.IGNORECASE),
]
RE_MARQUEURS_GROUPE = re.compile(
    r"\((?:TP|TD|CM|DS)[^)]*\)|Gr(?:oupe)?s?[ ]+(?:\d+|[A-Z])|"
    r"Parcours[ ]+[A-Z0-9]+|Promo[ ]+[A-Z]+|\b(?:CP[12]|ING[12])\b",
    re.IGNORECASE,
)
RE_NUMERO_TRAILING = re.compile(r"\s+\d+(?:\.\d+)*\s*$")
RE_LIGNE_GROUPE = re.compile(
    r"^(?:[A-Z]{2,3}\s*\d|ING\d|CP[12]|TD|TP|CM|PROMO|PARCOURS|FISE|GEE|GI|DK)\b",
    re.IGNORECASE,
)
RE_LIGNE_PROF = re.compile(
    r"^[A-ZÀ-ÖØ-Þ][A-ZÀ-ÖØ-Þ'’\- ]{1,} +[A-ZÀ-ÖØ-Þ][A-ZÀ-ÖØ-Þa-zà-öø-ÿ'’\- ]+$"
)


def extraire_matches(texte: str, regexes: list) -> list:
    resultats = []
    for rx in regexes:
        for match in rx.finditer(texte):
            valeur = (match.group(1) if match.lastindex else match.group(0)).strip().upper()
            if valeur and valeur not in resultats:
                resultats.append(valeur)
    return resultats


def detecter_groupes(titre: str, description: str) -> list:
    return extraire_matches(f"{titre} {description}", RE_GROUPES)


def detecter_types(titre: str, description: str) -> list:
    return extraire_matches(f"{titre} {description}", RE_TYPES)


def extraire_matiere(titre: str) -> str:
    texte = re.sub(r"\([^)]*\)", " ", titre)
    texte = texte.split(" - ")[0]
    texte = RE_MARQUEURS_GROUPE.sub(" ", texte)
    texte = RE_NUMERO_TRAILING.sub("", texte)
    return re.sub(r"\s+", " ", texte).strip().upper()


def extraire_enseignants(description: str) -> list:
    enseignants = []
    for ligne in description.splitlines():
        ligne = ligne.strip()
        if not ligne or ligne.startswith("(") or RE_LIGNE_GROUPE.match(ligne):
            continue
        if RE_LIGNE_PROF.match(ligne) and ligne not in enseignants:
            enseignants.append(ligne)
    return enseignants


def parser_cours(contenu: str) -> list:
    try:
        calendrier = Calendar(contenu)
    except Exception as exc:
        raise HTTPException(status_code=502, detail=f"Impossible de lire le fichier ICS : {exc}") from exc

    cours = []
    for evenement in calendrier.events:
        try:
            titre = str(evenement.name or "Cours sans titre").strip()
            description = str(evenement.description or "").strip()
            debut = evenement.begin.datetime
            fin = evenement.end.datetime

            if not isinstance(debut, datetime):
                debut = datetime.combine(debut, datetime.min.time())
            if not isinstance(fin, datetime):
                fin = datetime.combine(fin, datetime.min.time())

            debut = debut.astimezone(ZONE) if debut.tzinfo else debut.replace(tzinfo=ZONE)
            fin = fin.astimezone(ZONE) if fin.tzinfo else fin.replace(tzinfo=ZONE)
            enseignants = extraire_enseignants(description)

            cours.append(
                {
                    "titre": titre,
                    "salle": str(evenement.location or "").strip(),
                    "enseignant": ", ".join(enseignants),
                    "enseignants": enseignants,
                    "debut": debut.strftime("%H:%M"),
                    "fin": fin.strftime("%H:%M"),
                    "debut_iso": debut.isoformat(),
                    "fin_iso": fin.isoformat(),
                    "date": debut.date().isoformat(),
                    "groupes": detecter_groupes(titre, description),
                    "types": detecter_types(titre, description),
                    "matiere": extraire_matiere(titre),
                }
            )
        except Exception:
            continue

    cours.sort(key=lambda item: item["debut_iso"])
    return cours


def appliquer_filtres(
    cours: list,
    groupes: list | None = None,
    types: list | None = None,
    matieres: list | None = None,
) -> list:
    grp_actifs = {g.strip().lower() for g in (groupes or []) if g and g.strip()}
    typ_actifs = {t.strip().lower() for t in (types or []) if t and t.strip()}
    mat_actifs = {m.strip().lower() for m in (matieres or []) if m and m.strip()}

    gardes = []
    for cours_item in cours:
        if grp_actifs:
            groupes_c = {g.lower() for g in cours_item.get("groupes", [])}
            if groupes_c and not (groupes_c & grp_actifs):
                continue
        if typ_actifs:
            types_c = {t.lower() for t in cours_item.get("types", [])}
            if types_c and not (types_c & typ_actifs):
                continue
        if mat_actifs:
            matiere_c = (cours_item.get("matiere") or "").lower()
            if matiere_c and matiere_c not in mat_actifs:
                continue
        gardes.append(cours_item)
    return gardes


def charger_cours(user: User | None = None) -> list:
    url = obtenir_url_ade(user)
    if not url:
        return []
    return parser_cours(telecharger_ics(url))


def charger_cours_du_jour(user: User | None = None, jour: date | None = None) -> dict:
    jour = jour or datetime.now(ZONE).date()
    cours = [item for item in charger_cours(user) if item["date"] == jour.isoformat()]
    return {"date": jour.isoformat(), "nb_cours": len(cours), "cours": cours}


def charger_cours_semaine(user: User | None = None, ref: date | None = None) -> dict:
    ref = ref or datetime.now(ZONE).date()
    lundi = ref - timedelta(days=ref.weekday())
    vendredi = lundi + timedelta(days=4)
    cours = [
        item
        for item in charger_cours(user)
        if lundi <= date.fromisoformat(item["date"]) <= vendredi
    ]
    return {"date_debut": lundi.isoformat(), "date_fin": vendredi.isoformat(), "cours": cours}


def parser_date(valeur: Optional[str]) -> Optional[date]:
    if not valeur:
        return None
    try:
        return date.fromisoformat(valeur)
    except ValueError as exc:
        raise HTTPException(
            status_code=400,
            detail=f"Date invalide : {valeur} (format attendu YYYY-MM-DD)",
        ) from exc
