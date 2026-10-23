//! Tests d'intégration de l'API : requêtes HTTP réelles contre le routeur, base SQLite en mémoire.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tdb_server::{router, AppState, Settings, Store};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tower::ServiceExt;

fn application_avec(settings: Settings, store: Store) -> Router {
    router(AppState::nouveau(settings, store).expect("état"))
}

fn application() -> Router {
    application_avec(
        Settings::pour_tests(),
        Store::en_memoire().expect("base en mémoire"),
    )
}

async fn appel(
    app: &Router,
    methode: Method,
    chemin: &str,
    jeton: Option<&str>,
    corps: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(methode).uri(chemin);
    if let Some(j) = jeton {
        req = req.header(header::AUTHORIZATION, format!("Bearer {j}"));
    }
    let body = match corps {
        Some(v) => {
            req = req.header(header::CONTENT_TYPE, "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let reponse = app
        .clone()
        .oneshot(req.body(body).expect("requête"))
        .await
        .expect("réponse");
    let statut = reponse.status();
    let octets = reponse
        .into_body()
        .collect()
        .await
        .expect("corps")
        .to_bytes();
    let json = serde_json::from_slice(&octets).unwrap_or(Value::Null);
    (statut, json)
}

async fn inscrire(app: &Router, email: &str) -> String {
    let (s, v) = appel(
        app,
        Method::POST,
        "/api/auth/register",
        None,
        Some(json!({"email": email, "password": "motdepasse123", "username": "Test"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{v}");
    v["token"].as_str().expect("jeton").to_string()
}

#[tokio::test]
async fn sante() {
    let (s, v) = appel(&application(), Method::GET, "/api/health", None, None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["status"], "ok");
}

#[tokio::test]
async fn inscription_connexion_et_profil() {
    let app = application();
    let (s, v) = appel(
        &app,
        Method::POST,
        "/api/auth/register",
        None,
        Some(json!({"email": "Etudiant@Example.com", "password": "motdepasse123", "username": "Zayd"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["user"]["email"], "etudiant@example.com");
    assert!(v["user"].get("password_hash").is_none());
    assert!(v["user"].get("gemini_key").is_none());

    let (s, v) = appel(
        &app,
        Method::POST,
        "/api/auth/login",
        None,
        Some(json!({"email": "etudiant@example.com", "password": "motdepasse123"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let jeton = v["token"].as_str().expect("jeton").to_string();
    let (s, v) = appel(&app, Method::GET, "/api/auth/me", Some(&jeton), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["username"], "Zayd");
}

#[tokio::test]
async fn inscription_invalide_et_doublon() {
    let app = application();
    inscrire(&app, "meme@example.com").await;
    let (s, _) = appel(
        &app,
        Method::POST,
        "/api/auth/register",
        None,
        Some(json!({"email": "MEME@example.com", "password": "motdepasse123"})),
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT);
    for (email, mdp) in [("pas-un-email", "motdepasse123"), ("a@b.fr", "court")] {
        let (s, v) = appel(
            &app,
            Method::POST,
            "/api/auth/register",
            None,
            Some(json!({"email": email, "password": mdp})),
        )
        .await;
        assert_eq!(s, StatusCode::BAD_REQUEST, "{v}");
        assert!(v["detail"].is_string());
    }
}

#[tokio::test]
async fn connexion_refusee_et_limitation() {
    let app = application();
    inscrire(&app, "bloque@example.com").await;
    for _ in 0..10 {
        let (s, _) = appel(
            &app,
            Method::POST,
            "/api/auth/login",
            None,
            Some(json!({"email": "bloque@example.com", "password": "mauvais-mdp"})),
        )
        .await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);
    }
    // Même le bon mot de passe est refusé une fois le seuil d'échecs atteint.
    let (s, _) = appel(
        &app,
        Method::POST,
        "/api/auth/login",
        None,
        Some(json!({"email": "bloque@example.com", "password": "motdepasse123"})),
    )
    .await;
    assert_eq!(s, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn routes_protegees_exigent_un_jeton_valide() {
    let app = application();
    let (s, _) = appel(&app, Method::GET, "/api/devoirs", None, None).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let (s, _) = appel(
        &app,
        Method::GET,
        "/api/devoirs",
        Some("jeton.invalide.x"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn devoirs_crud_validation_et_isolation() {
    let app = application();
    let a = inscrire(&app, "a@example.com").await;
    let b = inscrire(&app, "b@example.com").await;

    let (s, d) = appel(
        &app,
        Method::POST,
        "/api/devoirs",
        Some(&a),
        Some(json!({"titre": "  Rapport  ", "matiere": "Physique", "echeance": "2026-11-02", "type": "ds"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{d}");
    assert_eq!(d["titre"], "Rapport");
    assert_eq!(d["statut"], "a_faire");
    let id = d["id"].as_i64().expect("id");

    for mauvais in [
        json!({"titre": ""}),
        json!({"titre": "x", "type": "inconnu"}),
        json!({"titre": "x", "echeance": "02/11/2026"}),
    ] {
        let (s, _) = appel(&app, Method::POST, "/api/devoirs", Some(&a), Some(mauvais)).await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    let chemin = format!("/api/devoirs/{id}");
    let (s, d) = appel(
        &app,
        Method::PATCH,
        &chemin,
        Some(&a),
        Some(json!({"statut": "en_cours"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(d["statut"], "en_cours");

    // L'utilisateur B ne voit, ne modifie et ne supprime pas les devoirs de A.
    let (_, liste_b) = appel(&app, Method::GET, "/api/devoirs", Some(&b), None).await;
    assert_eq!(liste_b, json!([]));
    let (s, _) = appel(
        &app,
        Method::PATCH,
        &chemin,
        Some(&b),
        Some(json!({"statut": "termine"})),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (s, _) = appel(&app, Method::DELETE, &chemin, Some(&b), None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    let (s, _) = appel(
        &app,
        Method::PATCH,
        "/api/devoirs/abc",
        Some(&a),
        Some(json!({})),
    )
    .await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);

    let (s, _) = appel(&app, Method::DELETE, &chemin, Some(&a), None).await;
    assert_eq!(s, StatusCode::OK);
    let (_, liste_a) = appel(&app, Method::GET, "/api/devoirs", Some(&a), None).await;
    assert_eq!(liste_a, json!([]));
}

#[tokio::test]
async fn import_export_atomique() {
    let app = application();
    let t = inscrire(&app, "imp@example.com").await;
    let (s, v) = appel(
        &app,
        Method::POST,
        "/api/devoirs/import",
        Some(&t),
        Some(json!([{"titre": "A"}, {"titre": "B", "type": "exam", "statut": "termine"}])),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(v["nb_importes"], 2);

    // Un seul élément invalide : rien n'est importé.
    let (s, _) = appel(
        &app,
        Method::POST,
        "/api/devoirs/import",
        Some(&t),
        Some(json!([{"titre": "C"}, {"titre": "D", "statut": "nimporte"}])),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    let trop: Vec<Value> = (0..501)
        .map(|i| json!({"titre": format!("T{i}")}))
        .collect();
    let (s, _) = appel(
        &app,
        Method::POST,
        "/api/devoirs/import",
        Some(&t),
        Some(json!(trop)),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    let (s, export) = appel(&app, Method::GET, "/api/devoirs/export", Some(&t), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(export.as_array().map(Vec::len), Some(2));
    assert!(export[0].get("id").is_none());
}

#[tokio::test]
async fn configuration_secrets_et_ssrf() {
    let app = application();
    let t = inscrire(&app, "cfg@example.com").await;

    let (s, v) = appel(
        &app,
        Method::POST,
        "/api/config",
        Some(&t),
        Some(json!({"username": "Zayd", "ade_url": "", "gemini_key": "AIzaSyCleSecrete123"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(v["gemini_configured"], true);

    // La clé n'est jamais renvoyée.
    let (_, v) = appel(&app, Method::GET, "/api/config", Some(&t), None).await;
    assert_eq!(v["gemini_configured"], true);
    assert_eq!(v["gemini_key"], "");
    assert!(!v.to_string().contains("CleSecrete"));

    let (_, v) = appel(
        &app,
        Method::POST,
        "/api/config",
        Some(&t),
        Some(json!({"remove_gemini_key": true})),
    )
    .await;
    assert_eq!(v["gemini_configured"], false);

    // Liens ADE refusés : non HTTPS, adresse locale ou privée.
    for url in [
        "http://ade.example.com/ics",
        "https://127.0.0.1/ics",
        "https://192.168.1.10/ics",
        "https://localhost/ics",
        "ftp://ade.example.com/ics",
    ] {
        let (s, v) = appel(
            &app,
            Method::POST,
            "/api/config",
            Some(&t),
            Some(json!({"ade_url": url})),
        )
        .await;
        assert_eq!(s, StatusCode::BAD_REQUEST, "{url} : {v}");
    }
}

#[tokio::test]
async fn liste_blanche_ade() {
    let mut settings = Settings::pour_tests();
    settings.ade_allowed_hosts = vec!["univ.example.fr".into()];
    let app = application_avec(settings, Store::en_memoire().expect("base"));
    let t = inscrire(&app, "lb@example.com").await;
    let (s, _) = appel(
        &app,
        Method::POST,
        "/api/config",
        Some(&t),
        Some(json!({"ade_url": "https://autre.example.com/ics"})),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, v) = appel(
        &app,
        Method::POST,
        "/api/config",
        Some(&t),
        Some(json!({"ade_url": "https://ade.univ.example.fr/ics?data=x"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{v}");
}

#[tokio::test]
async fn sans_lien_ade_emploi_du_temps_vide() {
    let app = application();
    let t = inscrire(&app, "vide@example.com").await;
    let (s, v) = appel(
        &app,
        Method::GET,
        "/api/cours-du-jour?groupes=1&groupes=2",
        Some(&t),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(v["nb_cours"], 0);
    assert_eq!(v["hors_ligne"], false);
    let (s, v) = appel(
        &app,
        Method::GET,
        "/api/cours-semaine?date=2026-10-21",
        Some(&t),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["date_debut"], "2026-10-19");
    assert_eq!(v["date_fin"], "2026-10-23");
    let (s, _) = appel(
        &app,
        Method::GET,
        "/api/cours-du-jour?date=pas-une-date",
        Some(&t),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, v) = appel(&app, Method::GET, "/api/config/categories", Some(&t), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v, json!({"groupes": [], "types": [], "matieres": []}));
}

#[tokio::test]
async fn preferences_aller_retour() {
    let app = application();
    let t = inscrire(&app, "pref@example.com").await;
    let (_, v) = appel(&app, Method::GET, "/api/preferences", Some(&t), None).await;
    assert_eq!(v, json!({}));
    let prefs = json!({"theme": "dark", "interface": "classic"});
    let (s, _) = appel(
        &app,
        Method::PUT,
        "/api/preferences",
        Some(&t),
        Some(prefs.clone()),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (_, v) = appel(&app, Method::GET, "/api/preferences", Some(&t), None).await;
    assert_eq!(v, prefs);
    let (s, _) = appel(
        &app,
        Method::PUT,
        "/api/preferences",
        Some(&t),
        Some(json!([1, 2])),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

/// Faux serveur Gemini : répond une fois et renvoie la requête reçue.
async fn faux_gemini(statut: u16, corps: String) -> (String, tokio::task::JoinHandle<String>) {
    let ecouteur = TcpListener::bind("127.0.0.1:0").await.expect("port");
    let base = format!("http://{}", ecouteur.local_addr().expect("adresse"));
    let tache = tokio::spawn(async move {
        let (mut flux, _) = ecouteur.accept().await.expect("connexion");
        let mut recu = vec![0u8; 16 * 1024];
        let n = flux.read(&mut recu).await.expect("lecture");
        let reponse = format!(
            "HTTP/1.1 {statut} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{corps}",
            corps.len()
        );
        flux.write_all(reponse.as_bytes()).await.expect("écriture");
        String::from_utf8_lossy(&recu[..n]).to_lowercase()
    });
    (base, tache)
}

#[tokio::test]
async fn plan_simulation_sans_cle() {
    let app = application();
    let t = inscrire(&app, "plan@example.com").await;
    appel(
        &app,
        Method::POST,
        "/api/devoirs",
        Some(&t),
        Some(json!({"titre": "DS maths", "type": "ds", "echeance": "2026-11-05"})),
    )
    .await;
    let (s, v) = appel(&app, Method::POST, "/api/plan-revision", Some(&t), None).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(v["mode"], "simulation");
    assert_eq!(v["planning"][0]["matiere"], "DS maths");
}

#[tokio::test]
async fn plan_gemini_simule_utilise_la_cle_dechiffree() {
    let reponse = json!({"candidates":[{"content":{"parts":[{"text":
        "{\"conseil\":\"Courage\",\"planning\":[{\"heure\":\"18:00 - 18:45\",\"matiere\":\"Maths\",\"action\":\"Exos\"}]}"}]}}]});
    let (base, tache) = faux_gemini(200, reponse.to_string()).await;
    let mut settings = Settings::pour_tests();
    settings.gemini_base = base;
    let app = application_avec(settings, Store::en_memoire().expect("base"));
    let t = inscrire(&app, "ia@example.com").await;
    appel(
        &app,
        Method::POST,
        "/api/config",
        Some(&t),
        Some(json!({"gemini_key": "AIzaCleDeTest123456"})),
    )
    .await;
    let (s, v) = appel(&app, Method::POST, "/api/plan-revision", Some(&t), None).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(v["mode"], "gemini");
    assert_eq!(v["conseil"], "Courage");
    let requete = tache.await.expect("tâche");
    assert!(requete.contains("x-goog-api-key: aizacledetest123456"));
    assert!(!requete
        .lines()
        .next()
        .unwrap_or_default()
        .contains("aizacle"));
}

#[tokio::test]
async fn plan_gemini_en_erreur_repli_en_simulation() {
    let (base, _t) = faux_gemini(429, "{}".into()).await;
    let mut settings = Settings::pour_tests();
    settings.gemini_base = base;
    let app = application_avec(settings, Store::en_memoire().expect("base"));
    let t = inscrire(&app, "quota@example.com").await;
    appel(
        &app,
        Method::POST,
        "/api/config",
        Some(&t),
        Some(json!({"gemini_key": "AIzaCleDeTest123456"})),
    )
    .await;
    let (s, v) = appel(&app, Method::POST, "/api/plan-revision", Some(&t), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["mode"], "simulation");
    assert!(v["avertissement"]
        .as_str()
        .unwrap_or_default()
        .contains("Quota"));
    assert!(!v.to_string().contains("AIzaCle"));
}

#[tokio::test]
async fn en_tetes_de_securite_et_json_invalide() {
    let app = application();
    let rep = app
        .clone()
        .oneshot(
            Request::get("/api/health")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("rep");
    let h = rep.headers();
    assert_eq!(h["x-content-type-options"], "nosniff");
    assert_eq!(h["x-frame-options"], "DENY");
    assert!(h["content-security-policy"]
        .to_str()
        .unwrap_or_default()
        .contains("frame-ancestors 'none'"));
    assert_eq!(h["cache-control"], "no-store");

    let req = Request::post("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{pas du json"))
        .expect("req");
    let rep = app.oneshot(req).await.expect("rep");
    assert_eq!(rep.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn migration_d_une_base_python_existante() {
    let dossier = tempfile::tempdir().expect("dossier");
    let chemin = dossier.path().join("app.db");
    // Base telle que créée par l'ancienne version Python (SQLAlchemy), avec un mot de passe bcrypt.
    let hash = bcrypt::hash("ancien-mot-de-passe", 4).expect("hash");
    {
        let c = rusqlite::Connection::open(&chemin).expect("base");
        c.execute_batch(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, email VARCHAR NOT NULL, password_hash VARCHAR NOT NULL,
                username VARCHAR, ade_ics_url VARCHAR, gemini_api_key VARCHAR, created_at DATETIME);
             CREATE UNIQUE INDEX ix_users_email ON users (email);
             CREATE TABLE devoirs (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users (id), titre VARCHAR NOT NULL,
                matiere VARCHAR, echeance VARCHAR, type VARCHAR, statut VARCHAR);
             CREATE TABLE config (id INTEGER PRIMARY KEY, user_id INTEGER, username VARCHAR, ade_url VARCHAR, gemini_key VARCHAR);",
        )
        .expect("schéma");
        c.execute(
            "INSERT INTO users (id, email, password_hash, username, ade_ics_url, gemini_api_key, created_at)
             VALUES (1, 'ancien@example.com', ?1, 'Ancien', 'https://ade.example.com/ics?data=1', 'AIzaAncienneCleEnClair', '2025-09-01 10:00:00.000000')",
            [&hash],
        )
        .expect("utilisateur");
        c.execute("INSERT INTO devoirs (user_id, titre, matiere, echeance, type, statut) VALUES (1, 'Vieux devoir', 'Maths', '2025-10-01', 'ds', 'en_cours')", [])
            .expect("devoir");
    }

    let store = Store::ouvrir(&chemin).expect("migration");
    assert_eq!(store.version().expect("version"), 4);
    // Une sauvegarde de l'ancienne base a été créée avant toute modification.
    let sauvegardes: Vec<_> = std::fs::read_dir(dossier.path().join("sauvegardes"))
        .expect("dossier de sauvegardes")
        .collect();
    assert_eq!(sauvegardes.len(), 1);

    let app = application_avec(Settings::pour_tests(), store);
    let (s, v) = appel(
        &app,
        Method::POST,
        "/api/auth/login",
        None,
        Some(json!({"email": "ancien@example.com", "password": "ancien-mot-de-passe"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{v}");
    let t = v["token"].as_str().expect("jeton").to_string();
    let (_, liste) = appel(&app, Method::GET, "/api/devoirs", Some(&t), None).await;
    assert_eq!(liste[0]["titre"], "Vieux devoir");
    assert_eq!(liste[0]["statut"], "en_cours");
    let (_, cfg) = appel(&app, Method::GET, "/api/config", Some(&t), None).await;
    assert_eq!(cfg["ade_url"], "https://ade.example.com/ics?data=1");
    assert_eq!(cfg["gemini_configured"], true);

    // Rouvrir la base ne refait pas de migration ni de nouvelle sauvegarde.
    drop(app);
    let store = Store::ouvrir(&chemin).expect("réouverture");
    assert_eq!(store.version().expect("version"), 4);
    assert_eq!(
        std::fs::read_dir(dossier.path().join("sauvegardes"))
            .expect("dossier")
            .count(),
        1
    );
}

#[tokio::test]
async fn todos_import_export_et_edition() {
    let app = application();
    let t = inscrire(&app, "todo@example.com").await;
    let autre = inscrire(&app, "todo2@example.com").await;
    let (s, _) = appel(
        &app,
        Method::POST,
        "/api/devoirs/import",
        Some(&t),
        Some(json!([
            {"titre": "Ancien"},
            {"titre": "TD Maths", "todos": [{"texte": "Exercice 1", "fait": true}, {"texte": "Exercice 2"}]}
        ])),
    )
    .await;
    assert_eq!(s, StatusCode::OK);

    let (_, liste) = appel(&app, Method::GET, "/api/devoirs", Some(&t), None).await;
    assert_eq!(liste[0]["todos"], json!([]));
    assert_eq!(liste[1]["todos"][0]["texte"], "Exercice 1");
    assert_eq!(liste[1]["todos"][0]["fait"], true);
    assert_eq!(liste[1]["todos"][1]["fait"], false);
    let id = liste[1]["id"].as_i64().expect("id");

    // Cocher / réordonner / ajouter via le remplacement de la liste.
    let (s, d) = appel(
        &app,
        Method::PUT,
        &format!("/api/devoirs/{id}/todos"),
        Some(&t),
        Some(json!([{"texte": "Exercice 2", "fait": true}, {"texte": "Exercice 1", "fait": true}, {"texte": "Exercice 3", "fait": false}])),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{d}");
    assert_eq!(d["todos"][0]["texte"], "Exercice 2");
    assert_eq!(d["todos"].as_array().map(Vec::len), Some(3));

    // Validation et isolation entre utilisateurs.
    let (s, _) = appel(
        &app,
        Method::PUT,
        &format!("/api/devoirs/{id}/todos"),
        Some(&t),
        Some(json!([{"texte": " "}])),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _) = appel(
        &app,
        Method::PUT,
        &format!("/api/devoirs/{id}/todos"),
        Some(&autre),
        Some(json!([])),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // Export : `todos` seulement quand la liste n'est pas vide.
    let (_, export) = appel(&app, Method::GET, "/api/devoirs/export", Some(&t), None).await;
    assert!(export[0].get("todos").is_none());
    assert_eq!(
        export[1]["todos"][2],
        json!({"texte": "Exercice 3", "fait": false})
    );
}
