//! État partagé de l'application et cache des flux ICS.

use crate::auth::Limiteur;
use crate::config::Settings;
use crate::store::Store;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tdb_core::ade::{self, Cours};
use tdb_core::{Erreur, Resultat};
use tdb_shared::EtatFlux;

pub struct Inner {
    pub settings: Settings,
    pub store: Store,
    pub ics: CacheIcs,
    pub limiteur: Limiteur,
    /// Hachage factice : une connexion sur un e-mail inconnu coûte autant qu'une connexion valide.
    pub hash_factice: String,
}

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

impl AppState {
    pub fn nouveau(settings: Settings, store: Store) -> anyhow::Result<AppState> {
        let hash_factice = bcrypt::hash("mot-de-passe-factice", settings.bcrypt_cost)?;
        Ok(AppState(Arc::new(Inner {
            settings,
            store,
            ics: CacheIcs::default(),
            limiteur: Limiteur::default(),
            hash_factice,
        })))
    }
}

impl Deref for AppState {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}

struct Entree {
    contenu: String,
    instant: Instant,
    maj: String,
}

/// Cache mémoire des flux ICS, borné en nombre d'entrées, partagé entre les utilisateurs d'un même lien.
#[derive(Default)]
pub struct CacheIcs {
    entrees: Mutex<HashMap<String, Entree>>,
}

/// Refuse les liens hors de la liste blanche éventuelle `ADE_ALLOWED_HOSTS`.
pub fn verifier_hote(settings: &Settings, url: &str) -> Resultat<()> {
    let analyse = ade::valider_syntaxe_url(url)?;
    if settings.ade_allowed_hosts.is_empty() {
        return Ok(());
    }
    let hote = analyse.host_str().unwrap_or_default().to_lowercase();
    let autorise = settings
        .ade_allowed_hosts
        .iter()
        .any(|a| hote == *a || hote.ends_with(&format!(".{a}")));
    if autorise {
        Ok(())
    } else {
        Err(Erreur::Validation("Ce domaine ADE n'est pas autorisé.".into()))
    }
}

fn verrou_err<T>(_: T) -> Erreur {
    Erreur::Interne("Verrou interne indisponible.".into())
}

impl CacheIcs {
    /// Contenu ICS : cache (TTL) → réseau → dernier contenu connu en cas de panne réseau.
    pub async fn contenu(
        &self,
        settings: &Settings,
        url: &str,
        forcer: bool,
    ) -> Resultat<(String, EtatFlux)> {
        verifier_hote(settings, url)?;
        if !forcer {
            let entrees = self.entrees.lock().map_err(verrou_err)?;
            if let Some(e) = entrees.get(url) {
                if e.instant.elapsed() < settings.ics_ttl {
                    return Ok((
                        e.contenu.clone(),
                        EtatFlux {
                            hors_ligne: false,
                            maj: Some(e.maj.clone()),
                            avertissement: None,
                        },
                    ));
                }
            }
        }
        match ade::telecharger_ics(url)
            .await
            .and_then(|c| ade::parser_cours(&c).map(|_| c))
        {
            Ok(contenu) => {
                let maj = chrono::Utc::now().to_rfc3339();
                let mut entrees = self.entrees.lock().map_err(verrou_err)?;
                if entrees.len() >= settings.ics_max_entries && !entrees.contains_key(url) {
                    if let Some(ancienne) = entrees
                        .iter()
                        .min_by_key(|(_, e)| e.instant)
                        .map(|(k, _)| k.clone())
                    {
                        entrees.remove(&ancienne);
                    }
                }
                entrees.insert(
                    url.to_string(),
                    Entree {
                        contenu: contenu.clone(),
                        instant: Instant::now(),
                        maj: maj.clone(),
                    },
                );
                Ok((
                    contenu,
                    EtatFlux {
                        hors_ligne: false,
                        maj: Some(maj),
                        avertissement: None,
                    },
                ))
            }
            Err(Erreur::Reseau(msg)) => {
                let entrees = self.entrees.lock().map_err(verrou_err)?;
                match entrees.get(url) {
                    Some(e) => Ok((
                        e.contenu.clone(),
                        EtatFlux {
                            hors_ligne: true,
                            maj: Some(e.maj.clone()),
                            avertissement: Some(msg),
                        },
                    )),
                    None => Err(Erreur::Reseau(msg)),
                }
            }
            Err(autre) => Err(autre),
        }
    }

    pub async fn cours(
        &self,
        settings: &Settings,
        url: &str,
        forcer: bool,
    ) -> Resultat<(Vec<Cours>, EtatFlux)> {
        if url.trim().is_empty() {
            return Ok((vec![], EtatFlux::default()));
        }
        let (contenu, etat) = self.contenu(settings, url, forcer).await?;
        Ok((ade::parser_cours(&contenu)?, etat))
    }
}
