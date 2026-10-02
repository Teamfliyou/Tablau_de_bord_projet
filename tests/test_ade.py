import pytest
from fastapi import HTTPException

from backend.services.ade import (
    appliquer_filtres,
    detecter_groupes,
    detecter_types,
    extraire_matiere,
    parser_cours,
    valider_url_ade,
)


def test_local_ade_urls_are_blocked():
    with pytest.raises(HTTPException):
        valider_url_ade("http://127.0.0.1/calendar.ics")
    with pytest.raises(HTTPException):
        valider_url_ade("https://localhost/calendar.ics")


def test_ade_detection_helpers():
    titre = "MECANIQUE DU SOLIDE - CINETIQUE (TP2)"
    description = "CP2 PROMO DK\nDUPONT Jean\nTD2"
    assert "2" in detecter_groupes(titre, description)
    assert "TP" in detecter_types(titre, description)
    assert extraire_matiere(titre) == "MECANIQUE DU SOLIDE"


def test_parse_and_filter_ics():
    contenu = """BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Tableau de bord//Tests//FR
BEGIN:VEVENT
UID:test-1
DTSTAMP:20261001T080000Z
DTSTART:20261005T080000Z
DTEND:20261005T100000Z
SUMMARY:ALGEBRE (TD2)
LOCATION:Salle A
DESCRIPTION:DUPONT Jean
END:VEVENT
END:VCALENDAR
"""
    cours = parser_cours(contenu)
    assert len(cours) == 1
    assert cours[0]["matiere"] == "ALGEBRE"
    assert "TD" in cours[0]["types"]
    assert appliquer_filtres(cours, types=["TD"]) == cours
    assert appliquer_filtres(cours, types=["TP"]) == []
