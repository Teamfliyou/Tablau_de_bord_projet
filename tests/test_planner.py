from types import SimpleNamespace

from backend.services.planner import simuler_plan


def devoir(titre, type_, echeance, statut="a_faire"):
    return SimpleNamespace(
        titre=titre,
        matiere="",
        type=type_,
        echeance=echeance,
        statut=statut,
    )


def test_local_planner_prioritizes_exam_then_ds():
    devoirs = [
        devoir("Devoir maison", "devoir", "2026-10-03"),
        devoir("Examen physique", "exam", "2026-10-20"),
        devoir("DS maths", "ds", "2026-10-05"),
    ]
    plan = simuler_plan([], devoirs)
    assert plan["planning"][0]["matiere"] == "Examen physique"
    assert plan["planning"][1]["matiere"] == "DS maths"
    assert plan["planning"][2]["matiere"] == "Devoir maison"
