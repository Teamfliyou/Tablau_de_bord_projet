use crate::error::{Erreur, Resultat};
use std::collections::HashMap;
use std::sync::Mutex;

/// Stockage sécurisé des secrets (gestionnaire d'identifiants Windows en production).
pub trait StockageSecrets: Send + Sync {
    fn lire(&self, nom: &str) -> Resultat<Option<String>>;
    fn ecrire(&self, nom: &str, valeur: &str) -> Resultat<()>;
    fn supprimer(&self, nom: &str) -> Resultat<()>;
}

pub const SECRET_GEMINI: &str = "gemini_api_key";

/// Implémentation en mémoire, utilisée par les tests.
#[derive(Default)]
pub struct SecretsMemoire(Mutex<HashMap<String, String>>);

fn verrou_err<T>(_: T) -> Erreur {
    Erreur::Interne("Verrou interne indisponible.".into())
}

impl StockageSecrets for SecretsMemoire {
    fn lire(&self, nom: &str) -> Resultat<Option<String>> {
        Ok(self.0.lock().map_err(verrou_err)?.get(nom).cloned())
    }
    fn ecrire(&self, nom: &str, valeur: &str) -> Resultat<()> {
        self.0
            .lock()
            .map_err(verrou_err)?
            .insert(nom.into(), valeur.into());
        Ok(())
    }
    fn supprimer(&self, nom: &str) -> Resultat<()> {
        self.0.lock().map_err(verrou_err)?.remove(nom);
        Ok(())
    }
}
