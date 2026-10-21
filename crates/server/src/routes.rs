//! Routes de l'API : mêmes chemins que l'ancienne version FastAPI, complétées par les préférences.

use crate::auth::{self, UtilisateurAuth};
use crate::crypto;
use crate::error::{ApiError, ApiResult};
use crate::state::{verifier_hote, AppState};
use crate::store::{MajConfig, Utilisateur};
use axum::extract::{FromRequest, FromRequestParts, State};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tdb_core::ade::{self, Categories};
use tdb_core::planner;
use tdb_shared::{
    Config, ConfigUpdate, Connexion, DevoirEntree, DevoirModification, Filtres, Inscription,
    Profil, ReponseAuth, IMPORT_MAX,
};

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);

#[derive(FromRequestParts)]
#[from_request(via(axum_extra::extract::Query), rejection(ApiError))]
pub struct ApiQuery<T>(pub T);

#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct ApiPath<T>(pub T);

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(sante))
        .route("/api/auth/register", post(inscription))
        .route("/api/auth/login", post(connexion))
        .route("/api/auth/me", get(profil_actuel))
        .route("/api/config", get(lire_config).post(enregistrer_config))
        .route("/api/config/categories", get(categories))
        .route(
            "/api/preferences",
            get(lire_preferences).put(ecrire_preferences),
        )
        .route("/api/cours-du-jour", get(cours_du_jour))
        .route("/api/cours-semaine", get(cours_semaine))
        .route("/api/devoirs", get(lister_devoirs).post(creer_devoir))
        .route("/api/devoirs/export", get(exporter_devoirs))
        .route("/api/devoirs/import", post(importer_devoirs))
        .route(
            "/api/devoirs/{id}",
            patch(modifier_devoir).delete(supprimer_devoir),
        )
        .route("/api/plan-revision", post(plan_revision))
}

// ------------------------------------------------------------------- Système

async fn sante() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

// ------------------------------------------------------------ Authentification

fn email_valide(email: &str) -> bool {
    let Some((local, domaine)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !email.chars().any(char::is_whitespace)
        && !domaine.contains('@')
        && domaine
            .rsplit_once('.')
            .is_some_and(|(h, tld)| !h.is_empty() && !tld.is_empty())
}

fn profil(u: &Utilisateur) -> Profil {
    Profil {
        id: u.id,
        email: u.email.clone(),
        username: if u.username.is_empty() {
            u.email.split('@').next().unwrap_or_default().to_string()
        } else {
            u.username.clone()
        },
        ade_url: u.ade_url.clone(),
        gemini_configured: !u.gemini_key.is_empty(),
        created_at: u.created_at.clone(),
    }
}

async fn inscription(
    State(s): State<AppState>,
    ApiJson(d): ApiJson<Inscription>,
) -> ApiResult<Json<ReponseAuth>> {
    let email = d.email.trim().to_lowercase();
    if !(3..=254).contains(&email.chars().count()) {
        return Err(ApiError::requete("Adresse e-mail invalide"));
    }
    if !email_valide(&email) {
        return Err(ApiError::requete("Adresse e-mail invalide"));
    }
    if d.password.chars().count() < 8 || d.password.chars().count() > 128 {
        return Err(ApiError::requete(
            "Le mot de passe doit contenir entre 8 et 128 caractères.",
        ));
    }
    if d.password.len() > 72 {
        return Err(ApiError::requete(
            "Le mot de passe est limité à 72 octets (limite de bcrypt).",
        ));
    }
    let username = d.username.trim().to_string();
    if username.chars().count() > 80 {
        return Err(ApiError::requete("Le pseudo est limité à 80 caractères."));
    }
    let hash = auth::hacher(d.password, s.settings.bcrypt_cost).await?;
    let user = s
        .store
        .creer_utilisateur(&email, &hash, &username)?
        .ok_or_else(|| {
            ApiError::new(
                axum::http::StatusCode::CONFLICT,
                "Un compte existe déjà avec cet e-mail",
            )
        })?;
    let token = auth::creer_jeton(&s.settings.secret, s.settings.jwt_minutes, user.id)?;
    Ok(Json(ReponseAuth {
        token,
        user: profil(&user),
    }))
}

async fn connexion(
    State(s): State<AppState>,
    ApiJson(d): ApiJson<Connexion>,
) -> ApiResult<Json<ReponseAuth>> {
    let email = d.email.trim().to_lowercase();
    if s.limiteur.bloque(&email) {
        return Err(ApiError::new(
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "Trop de tentatives de connexion : réessaie dans quelques minutes.",
        ));
    }
    let user = s.store.utilisateur_par_email(&email)?;
    let hash = user
        .as_ref()
        .map_or_else(|| s.hash_factice.clone(), |u| u.password_hash.clone());
    let ok = auth::verifier(d.password, hash).await;
    match user {
        Some(u) if ok => {
            s.limiteur.succes(&email);
            let token = auth::creer_jeton(&s.settings.secret, s.settings.jwt_minutes, u.id)?;
            Ok(Json(ReponseAuth {
                token,
                user: profil(&u),
            }))
        }
        _ => {
            s.limiteur.echec(&email);
            Err(ApiError::non_authentifie(
                "E-mail ou mot de passe incorrect",
            ))
        }
    }
}

async fn profil_actuel(UtilisateurAuth(u): UtilisateurAuth) -> Json<Profil> {
    Json(profil(&u))
}

// -------------------------------------------------------------- Configuration

fn config_de(s: &AppState, u: &Utilisateur) -> Config {
    Config {
        username: if u.username.is_empty() {
            u.email.split('@').next().unwrap_or_default().to_string()
        } else {
            u.username.clone()
        },
        ade_url: u.ade_url.clone(),
        gemini_model: if u.gemini_model.is_empty() {
            planner::MODELE_PAR_DEFAUT.to_string()
        } else {
            u.gemini_model.clone()
        },
        gemini_configured: cle_gemini(s, u).is_some(),
    }
}

fn cle_gemini(s: &AppState, u: &Utilisateur) -> Option<String> {
    crypto::dechiffrer(&s.settings.secret, &u.gemini_key)
        .filter(|c| !c.trim().is_empty())
        .or_else(|| s.settings.gemini_api_key.clone())
}

async fn lire_config(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
) -> Json<Value> {
    let c = config_de(&s, &u);
    // `gemini_key` est conservé (toujours vide) pour la compatibilité du contrat historique.
    Json(json!({
        "username": c.username,
        "ade_url": c.ade_url,
        "gemini_key": "",
        "gemini_model": c.gemini_model,
        "gemini_configured": c.gemini_configured,
    }))
}

async fn enregistrer_config(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
    ApiJson(maj): ApiJson<ConfigUpdate>,
) -> ApiResult<Json<Value>> {
    let username = maj.username.trim();
    if username.chars().count() > 80 {
        return Err(ApiError::requete("Le pseudo est limité à 80 caractères."));
    }
    let ade_url = maj.ade_url.trim();
    if ade_url.len() > 2048 {
        return Err(ApiError::requete("Le lien ADE est trop long."));
    }
    if !ade_url.is_empty() {
        verifier_hote(&s.settings, ade_url)?;
    }
    let modele = match maj
        .gemini_model
        .as_deref()
        .map(str::trim)
        .filter(|m| !m.is_empty())
    {
        Some(m) => Some(planner::valider_modele(m)?.to_string()),
        None => None,
    };
    let nouvelle_cle = maj
        .gemini_key
        .as_deref()
        .map(str::trim)
        .filter(|k| !k.is_empty());
    if nouvelle_cle.is_some_and(|k| k.len() > 512) {
        return Err(ApiError::requete("La clé API est trop longue."));
    }
    let cle_chiffree = if maj.remove_gemini_key {
        Some(String::new())
    } else {
        nouvelle_cle
            .map(|k| crypto::chiffrer(&s.settings.secret, k))
            .transpose()?
    };
    s.store.enregistrer_config(
        u.id,
        &MajConfig {
            username,
            ade_url,
            gemini_model: modele.as_deref(),
            gemini_key: cle_chiffree.as_deref(),
        },
    )?;
    let a_jour = s
        .store
        .utilisateur_par_id(u.id)?
        .ok_or_else(|| ApiError::non_authentifie("Utilisateur introuvable"))?;
    let c = config_de(&s, &a_jour);
    Ok(Json(json!({
        "detail": "Configuration enregistrée",
        "username": c.username,
        "ade_url": c.ade_url,
        "gemini_model": c.gemini_model,
        "gemini_configured": c.gemini_configured,
    })))
}

async fn categories(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
) -> ApiResult<Json<Categories>> {
    let (tous, _) = s.ics.cours(&s.settings, &u.ade_url, true).await?;
    Ok(Json(ade::analyser_categories(&tous)))
}

async fn lire_preferences(UtilisateurAuth(u): UtilisateurAuth) -> Json<Value> {
    let v: Value = serde_json::from_str(&u.preferences).unwrap_or_else(|_| json!({}));
    Json(if v.is_object() { v } else { json!({}) })
}

async fn ecrire_preferences(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
    ApiJson(prefs): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    if !prefs.is_object() {
        return Err(ApiError::requete("Préférences invalides."));
    }
    let texte = prefs.to_string();
    if texte.len() > 16 * 1024 {
        return Err(ApiError::requete("Préférences trop volumineuses."));
    }
    s.store.enregistrer_preferences(u.id, &texte)?;
    Ok(Json(json!({ "detail": "Préférences enregistrées" })))
}

// ------------------------------------------------------------ Emploi du temps

#[derive(Deserialize, Default)]
pub struct RequeteCours {
    date: Option<String>,
    #[serde(default)]
    groupes: Vec<String>,
    #[serde(default)]
    types: Vec<String>,
    #[serde(default)]
    matieres: Vec<String>,
}

impl RequeteCours {
    fn filtres(&self) -> Filtres {
        Filtres {
            groupes: self.groupes.clone(),
            types: self.types.clone(),
            matieres: self.matieres.clone(),
        }
    }
}

async fn cours_du_jour(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
    ApiQuery(q): ApiQuery<RequeteCours>,
) -> ApiResult<Json<Value>> {
    let jour = ade::parser_jour(q.date.as_deref())?;
    let (tous, etat) = s.ics.cours(&s.settings, &u.ade_url, false).await?;
    let r = ade::cours_du_jour(tous, jour, &q.filtres());
    Ok(Json(fusion(serde_json::to_value(r), etat)))
}

async fn cours_semaine(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
    ApiQuery(q): ApiQuery<RequeteCours>,
) -> ApiResult<Json<Value>> {
    let jour = ade::parser_jour(q.date.as_deref())?;
    let (tous, etat) = s.ics.cours(&s.settings, &u.ade_url, false).await?;
    let r = ade::cours_semaine(tous, jour, &q.filtres());
    Ok(Json(fusion(serde_json::to_value(r), etat)))
}

/// Ajoute `hors_ligne`, `maj` et `avertissement` à la réponse (même forme que l'application Windows).
fn fusion(base: Result<Value, serde_json::Error>, etat: tdb_shared::EtatFlux) -> Value {
    let mut v = base.unwrap_or_else(|_| json!({}));
    if let (Some(o), Ok(Value::Object(e))) = (v.as_object_mut(), serde_json::to_value(etat)) {
        o.extend(e);
    }
    v
}

// --------------------------------------------------------------------- Devoirs

async fn lister_devoirs(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
) -> ApiResult<Json<Vec<tdb_shared::Devoir>>> {
    Ok(Json(s.store.lister_devoirs(u.id)?))
}

async fn creer_devoir(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
    ApiJson(d): ApiJson<DevoirEntree>,
) -> ApiResult<Json<tdb_shared::Devoir>> {
    Ok(Json(s.store.creer_devoir(u.id, d)?))
}

async fn modifier_devoir(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
    ApiPath(id): ApiPath<i64>,
    ApiJson(m): ApiJson<DevoirModification>,
) -> ApiResult<Json<tdb_shared::Devoir>> {
    Ok(Json(s.store.modifier_devoir(u.id, id, m)?))
}

async fn supprimer_devoir(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<Value>> {
    s.store.supprimer_devoir(u.id, id)?;
    Ok(Json(json!({ "detail": "Devoir supprimé" })))
}

async fn exporter_devoirs(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
) -> ApiResult<Json<Vec<DevoirEntree>>> {
    let liste = s
        .store
        .lister_devoirs(u.id)?
        .into_iter()
        .map(|d| DevoirEntree {
            titre: d.titre,
            matiere: d.matiere,
            echeance: d.echeance,
            type_: d.type_,
            statut: d.statut,
        })
        .collect();
    Ok(Json(liste))
}

async fn importer_devoirs(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
    ApiJson(devoirs): ApiJson<Vec<DevoirEntree>>,
) -> ApiResult<Json<Value>> {
    if devoirs.len() > IMPORT_MAX {
        return Err(ApiError::requete(
            "Import limité à 500 devoirs par fichier.",
        ));
    }
    let n = s.store.importer_devoirs(u.id, devoirs)?;
    Ok(Json(json!({
        "detail": format!("{n} devoir(s) importé(s)"),
        "nb_importes": n,
    })))
}

// --------------------------------------------------------------------- Plan IA

async fn plan_revision(
    State(s): State<AppState>,
    UtilisateurAuth(u): UtilisateurAuth,
    ApiQuery(q): ApiQuery<RequeteCours>,
) -> ApiResult<Json<tdb_shared::Plan>> {
    // Sans emploi du temps disponible, le plan reste utile à partir des devoirs seuls.
    let cours = match s.ics.cours(&s.settings, &u.ade_url, false).await {
        Ok((tous, _)) => ade::cours_du_jour(tous, None, &q.filtres()).cours,
        Err(e) => {
            tracing::warn!("Plan généré sans emploi du temps : {e}");
            vec![]
        }
    };
    let devoirs = s.store.lister_devoirs(u.id)?;
    let cle = cle_gemini(&s, &u);
    let modele = if u.gemini_model.is_empty() {
        planner::MODELE_PAR_DEFAUT.to_string()
    } else {
        u.gemini_model.clone()
    };
    let plan = planner::generer_plan(
        &s.settings.gemini_base,
        cle.as_deref(),
        &modele,
        &cours,
        &devoirs,
    )
    .await;
    Ok(Json(plan))
}
