def test_register_login_and_profile(client):
    inscription = client.post(
        "/api/auth/register",
        json={
            "email": "Etudiant@Example.com",
            "password": "motdepasse123",
            "username": "Zayd",
        },
    )
    assert inscription.status_code == 200
    payload = inscription.json()
    assert payload["user"]["email"] == "etudiant@example.com"
    assert "password_hash" not in payload["user"]
    assert "gemini_key" not in payload["user"]

    login = client.post(
        "/api/auth/login",
        json={"email": "etudiant@example.com", "password": "motdepasse123"},
    )
    assert login.status_code == 200

    token = login.json()["token"]
    me = client.get("/api/auth/me", headers={"Authorization": f"Bearer {token}"})
    assert me.status_code == 200
    assert me.json()["username"] == "Zayd"


def test_duplicate_email_is_rejected(client, register_user):
    register_user(email="same@example.com")
    duplicate = client.post(
        "/api/auth/register",
        json={"email": "SAME@example.com", "password": "motdepasse123", "username": ""},
    )
    assert duplicate.status_code == 409


def test_protected_route_requires_token(client):
    response = client.get("/api/devoirs")
    assert response.status_code == 401
