def test_config_never_returns_gemini_secret(client, auth_headers):
    response = client.get("/api/config", headers=auth_headers)
    assert response.status_code == 200
    data = response.json()
    assert data["gemini_key"] == ""
    assert data["gemini_configured"] is False


def test_config_rejects_local_ade_url(client, auth_headers):
    response = client.post(
        "/api/config",
        headers=auth_headers,
        json={
            "username": "Test",
            "ade_url": "https://localhost/private.ics",
            "gemini_key": "",
        },
    )
    assert response.status_code == 400
