//! Serveur Web du Tableau de bord étudiant : API REST (Axum) + frontend Leptos statique.

pub mod auth;
pub mod config;
pub mod crypto;
pub mod error;
pub mod routes;
pub mod state;
pub mod store;

use axum::extract::Request;
use axum::http::{header, HeaderName, HeaderValue, Method};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::Router;
use sha2::{Digest, Sha256};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

pub use config::Settings;
pub use state::AppState;
pub use store::Store;

/// Empreintes `'sha256-…'` des scripts en ligne de `index.html` (injectés par Trunk pour charger le WASM).
fn empreintes_scripts(html: &str) -> Vec<String> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    let mut resultat = vec![];
    let mut reste = html;
    while let Some(debut) = reste.find("<script") {
        let apres = &reste[debut..];
        let Some(fin_balise) = apres.find('>') else {
            break;
        };
        let balise = &apres[..fin_balise];
        let corps_depuis = &apres[fin_balise + 1..];
        let Some(fin) = corps_depuis.find("</script>") else {
            break;
        };
        let corps = &corps_depuis[..fin];
        if !balise.contains("src=") && !corps.trim().is_empty() {
            let empreinte = STANDARD.encode(Sha256::digest(corps.as_bytes()));
            resultat.push(format!("'sha256-{empreinte}'"));
        }
        reste = &corps_depuis[fin..];
    }
    resultat
}

fn politique_csp(scripts_en_ligne: &[String]) -> String {
    let mut script_src = String::from("'self' 'wasm-unsafe-eval'");
    for e in scripts_en_ligne {
        script_src.push(' ');
        script_src.push_str(e);
    }
    format!(
        "default-src 'self'; script-src {script_src}; style-src 'self' 'unsafe-inline'; \
         img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; \
         base-uri 'self'; form-action 'self'; frame-ancestors 'none'"
    )
}

async fn pas_de_cache_api(req: Request, next: Next) -> Response {
    let api = req.uri().path().starts_with("/api");
    let mut reponse = next.run(req).await;
    if api {
        reponse
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    reponse
}

pub fn router(state: AppState) -> Router {
    let settings = &state.settings;
    let mut app: Router = Router::new().merge(routes::routes().with_state(state.clone()));

    let mut scripts = vec![];
    if let Some(dossier) = &settings.frontend_dir {
        let index = dossier.join("index.html");
        if let Ok(html) = std::fs::read_to_string(&index) {
            scripts = empreintes_scripts(&html);
        }
        let statique = ServeDir::new(dossier).not_found_service(ServeFile::new(index));
        app = app.fallback_service(statique);
    }

    let entetes: [(HeaderName, String); 5] = [
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff".into()),
        (header::X_FRAME_OPTIONS, "DENY".into()),
        (
            header::REFERRER_POLICY,
            "strict-origin-when-cross-origin".into(),
        ),
        (
            HeaderName::from_static("permissions-policy"),
            "camera=(), microphone=(), geolocation=()".into(),
        ),
        (header::CONTENT_SECURITY_POLICY, politique_csp(&scripts)),
    ];
    for (nom, valeur) in entetes {
        if let Ok(v) = HeaderValue::from_str(&valeur) {
            app = app.layer(SetResponseHeaderLayer::if_not_present(nom, v));
        }
    }

    if !settings.allowed_origins.is_empty() {
        let origines = if settings.allowed_origins.iter().any(|o| o == "*") {
            tracing::warn!("CORS ouvert à toutes les origines (ALLOWED_ORIGINS=*).");
            AllowOrigin::any()
        } else {
            AllowOrigin::list(
                settings
                    .allowed_origins
                    .iter()
                    .filter_map(|o| HeaderValue::from_str(o).ok()),
            )
        };
        app = app.layer(
            CorsLayer::new()
                .allow_origin(origines)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PATCH,
                    Method::PUT,
                    Method::DELETE,
                ])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        );
    }

    app.layer(middleware::from_fn(pas_de_cache_api))
        .layer(TraceLayer::new_for_http())
}
