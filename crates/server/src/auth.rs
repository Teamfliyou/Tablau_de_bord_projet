//! Authentification : mots de passe bcrypt, jetons JWT (HS256) et limitation des tentatives.

use crate::error::ApiError;
use crate::state::AppState;
use crate::store::Utilisateur;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_ECHECS: u32 = 10;
const FENETRE: Duration = Duration::from_secs(15 * 60);
const MAX_ENTREES: usize = 10_000;

#[derive(Serialize, Deserialize)]
struct Revendications {
    sub: String,
    exp: usize,
}

pub async fn hacher(mot_de_passe: String, cout: u32) -> Result<String, ApiError> {
    tokio::task::spawn_blocking(move || bcrypt::hash(mot_de_passe, cout))
        .await
        .map_err(|_| {
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Erreur interne du serveur.",
            )
        })?
        .map_err(|_| {
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Erreur interne du serveur.",
            )
        })
}

pub async fn verifier(mot_de_passe: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || bcrypt::verify(mot_de_passe, &hash).unwrap_or(false))
        .await
        .unwrap_or(false)
}

pub fn creer_jeton(secret: &[u8], minutes: i64, user_id: i64) -> Result<String, ApiError> {
    let expiration = chrono::Utc::now() + chrono::Duration::minutes(minutes);
    let revendications = Revendications {
        sub: user_id.to_string(),
        exp: usize::try_from(expiration.timestamp()).unwrap_or(0),
    };
    encode(
        &Header::default(),
        &revendications,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|_| {
        ApiError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Erreur interne du serveur.",
        )
    })
}

fn lire_jeton(secret: &[u8], jeton: &str) -> Option<i64> {
    let donnees = decode::<Revendications>(
        jeton,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .ok()?;
    donnees.claims.sub.parse().ok()
}

/// Utilisateur authentifié par l'en-tête `Authorization: Bearer <jeton>`.
pub struct UtilisateurAuth(pub Utilisateur);

impl FromRequestParts<AppState> for UtilisateurAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let entete = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::non_authentifie("Authentification requise"))?;
        let jeton = entete
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::non_authentifie("Authentification requise"))?;
        let id = lire_jeton(&state.settings.secret, jeton.trim())
            .ok_or_else(|| ApiError::non_authentifie("Token invalide ou expiré"))?;
        let utilisateur = state
            .store
            .utilisateur_par_id(id)?
            .ok_or_else(|| ApiError::non_authentifie("Utilisateur introuvable"))?;
        Ok(UtilisateurAuth(utilisateur))
    }
}

/// Limite les échecs de connexion par e-mail (fenêtre glissante simple, en mémoire).
#[derive(Default)]
pub struct Limiteur {
    echecs: Mutex<HashMap<String, (u32, Instant)>>,
}

impl Limiteur {
    pub fn bloque(&self, cle: &str) -> bool {
        let Ok(mut m) = self.echecs.lock() else {
            return false;
        };
        match m.get(cle) {
            Some((n, debut)) if debut.elapsed() < FENETRE => *n >= MAX_ECHECS,
            Some(_) => {
                m.remove(cle);
                false
            }
            None => false,
        }
    }

    pub fn echec(&self, cle: &str) {
        let Ok(mut m) = self.echecs.lock() else {
            return;
        };
        if m.len() >= MAX_ENTREES {
            m.retain(|_, (_, debut)| debut.elapsed() < FENETRE);
        }
        let e = m.entry(cle.to_string()).or_insert((0, Instant::now()));
        if e.1.elapsed() >= FENETRE {
            *e = (0, Instant::now());
        }
        e.0 += 1;
    }

    pub fn succes(&self, cle: &str) {
        if let Ok(mut m) = self.echecs.lock() {
            m.remove(cle);
        }
    }
}
