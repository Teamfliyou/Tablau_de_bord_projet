//! Secrets stockés dans le gestionnaire d'identifiants Windows (Credential Manager).

use keyring::Entry;
use tdb_core::secrets::StockageSecrets;
use tdb_core::{Erreur, Resultat};

const SERVICE: &str = "TableauDeBordEtudiant";

pub struct CoffreWindows;

fn entree(nom: &str) -> Resultat<Entry> {
    Entry::new(SERVICE, nom).map_err(|_| indisponible())
}

fn indisponible() -> Erreur {
    // Message volontairement générique : ni nom de secret ni valeur dans les erreurs.
    Erreur::Secret("Impossible d'accéder au gestionnaire d'identifiants Windows.".into())
}

impl StockageSecrets for CoffreWindows {
    fn lire(&self, nom: &str) -> Resultat<Option<String>> {
        match entree(nom)?.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(indisponible()),
        }
    }

    fn ecrire(&self, nom: &str, valeur: &str) -> Resultat<()> {
        entree(nom)?
            .set_password(valeur)
            .map_err(|_| indisponible())
    }

    fn supprimer(&self, nom: &str) -> Resultat<()> {
        match entree(nom)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(indisponible()),
        }
    }
}
