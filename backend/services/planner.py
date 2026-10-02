import json
import os

from backend.settings import GEMINI_MODEL

POIDS_TYPE = {"exam": 0, "ds": 1, "ie": 2, "devoir": 3}


def _decrire_cours(cours: dict) -> str:
    infos = f"{cours['titre']} ({cours['debut']} -> {cours['fin']}"
    if cours.get("salle"):
        infos += f", salle {cours['salle']}"
    if cours.get("enseignant"):
        infos += f", professeur : {cours['enseignant']}"
    return infos + ")"


def construire_prompt(cours: list, devoirs: list) -> str:
    cours_txt = "\n".join(f"- {_decrire_cours(c)}" for c in cours) or "- Aucun cours aujourd'hui."
    devoirs_txt = "\n".join(
        f"- {d.titre} | matière : {d.matiere or 'non précisée'} | type : {d.type} | "
        f"échéance : {d.echeance or 'non précisée'} | statut : {d.statut}"
        for d in devoirs
    ) or "- Aucun devoir enregistré."

    return f"""Tu es un coach de révision personnel pour un étudiant en école d'ingénieur.

Cours de la journée :
{cours_txt}

Devoirs enregistrés :
{devoirs_txt}

Génère un plan de révision du soir au format JSON strict (sans texte autour), avec cette structure EXACTE :
{{
  "conseil": "un conseil général sur la gestion de la soirée, en 2 ou 3 phrases",
  "planning": [
    {{
      "heure": "18:00 - 18:45",
      "matiere": "nom de la matière",
      "action": "tâche précise et réalisable"
    }}
  ]
}}

Consignes :
- Priorise les devoirs non terminés : les examens (exam) et DS d'abord, puis les IE, puis les devoirs.
- Parmi ceux-là, priorise ceux dont l'échéance est la plus proche.
- Ne révise pas ce qui a déjà été vu aujourd'hui en cours si possible.
- Propose 3 à 5 créneaux le soir, avec des pauses de 5 à 10 minutes.
- Réponds UNIQUEMENT avec du JSON valide, aucune autre sortie."""


def simuler_plan(cours: list, devoirs: list) -> dict:
    non_termines = [d for d in devoirs if d.statut != "termine"]
    non_termines.sort(key=lambda d: (POIDS_TYPE.get(d.type, 3), d.echeance or "9999-12-31"))
    sujets = [d.titre for d in non_termines]

    for cours_item in cours:
        if len(sujets) >= 4:
            break
        if cours_item["titre"] not in sujets:
            sujets.append(cours_item["titre"])

    heures = [("18:00", "18:45"), ("18:55", "19:40"), ("19:50", "20:35"), ("20:45", "21:30")]
    planning = [
        {
            "heure": f"{debut} - {fin}",
            "matiere": sujet,
            "action": f"Réviser {sujet} et faire des exercices ciblés",
        }
        for sujet, (debut, fin) in zip(sujets[:4], heures)
    ]

    if not planning:
        planning.append(
            {
                "heure": "18:00 - 18:45",
                "matiere": "Revue de la journée",
                "action": "Relire tes notes de cours et anticiper la prochaine séance",
            }
        )

    return {
        "conseil": (
            "Commence par une session de 45 minutes pour te mettre dans le bain, "
            "puis alterne révisions et courtes pauses. Termine en douceur en relisant "
            "tes notes plutôt qu'en attaquant un nouveau chapitre."
        ),
        "planning": planning,
    }


def generer_plan(cours: list, devoirs: list, api_key: str | None = None) -> dict:
    if not api_key:
        return simuler_plan(cours, devoirs)

    try:
        from google import genai
        from google.genai import types

        client = genai.Client(api_key=api_key)
        reponse = client.models.generate_content(
            model=os.getenv("GEMINI_MODEL", GEMINI_MODEL),
            contents=construire_prompt(cours, devoirs),
            config=types.GenerateContentConfig(
                response_mime_type="application/json",
                temperature=0.7,
            ),
        )
        texte = (reponse.text or "").strip()
        texte = texte.removeprefix("```json").removeprefix("```").removesuffix("```").strip()
        resultat = json.loads(texte)
        if not isinstance(resultat, dict) or not isinstance(resultat.get("planning"), list):
            raise ValueError("Réponse IA invalide")
        return resultat
    except Exception:
        return simuler_plan(cours, devoirs)
