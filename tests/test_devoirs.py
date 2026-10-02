def test_devoir_crud_and_validation(client, auth_headers):
    created = client.post(
        "/api/devoirs",
        headers=auth_headers,
        json={
            "titre": "Réviser algèbre",
            "matiere": "Maths",
            "echeance": "2026-10-12",
            "type": "ds",
            "statut": "a_faire",
        },
    )
    assert created.status_code == 200
    devoir = created.json()

    listing = client.get("/api/devoirs", headers=auth_headers)
    assert listing.status_code == 200
    assert len(listing.json()) == 1

    updated = client.patch(
        f"/api/devoirs/{devoir['id']}",
        headers=auth_headers,
        json={"statut": "termine"},
    )
    assert updated.status_code == 200
    assert updated.json()["statut"] == "termine"

    invalid = client.post(
        "/api/devoirs",
        headers=auth_headers,
        json={"titre": "Test", "type": "inconnu", "statut": "banane"},
    )
    assert invalid.status_code == 422

    invalid_date = client.post(
        "/api/devoirs",
        headers=auth_headers,
        json={"titre": "Test", "echeance": "demain"},
    )
    assert invalid_date.status_code == 422

    deleted = client.delete(f"/api/devoirs/{devoir['id']}", headers=auth_headers)
    assert deleted.status_code == 200
    assert client.get("/api/devoirs", headers=auth_headers).json() == []


def test_users_cannot_access_each_others_devoirs(client, register_user):
    user1 = register_user(email="u1@example.com", username="U1")
    user2 = register_user(email="u2@example.com", username="U2")
    h1 = {"Authorization": f"Bearer {user1['token']}"}
    h2 = {"Authorization": f"Bearer {user2['token']}"}

    created = client.post("/api/devoirs", headers=h1, json={"titre": "Privé"})
    assert created.status_code == 200
    devoir_id = created.json()["id"]

    assert client.get("/api/devoirs", headers=h2).json() == []
    assert client.patch(
        f"/api/devoirs/{devoir_id}",
        headers=h2,
        json={"statut": "termine"},
    ).status_code == 404
    assert client.delete(f"/api/devoirs/{devoir_id}", headers=h2).status_code == 404


def test_import_is_limited(client, auth_headers):
    payload = [{"titre": f"Devoir {i}"} for i in range(501)]
    response = client.post("/api/devoirs/import", headers=auth_headers, json=payload)
    assert response.status_code == 400
