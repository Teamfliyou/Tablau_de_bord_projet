use serde::{Serialize, Serializer};

/// Erreur applicative : le message est toujours en français et ne contient jamais de secret.
#[derive(Debug, thiserror::Error)]
pub enum Erreur {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Introuvable(String),
    #[error("{0}")]
    Reseau(String),
    #[error("{0}")]
    Secret(String),
    #[error("Erreur de base de données : {0}")]
    Db(#[from] rusqlite::Error),
    #[error("{0}")]
    Interne(String),
}

impl Serialize for Erreur {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Resultat<T> = Result<T, Erreur>;

pub fn validation<T>(msg: &str) -> Resultat<T> {
    Err(Erreur::Validation(msg.to_string()))
}

/// Les règles de validation partagées renvoient un message : il devient une erreur de validation.
impl From<String> for Erreur {
    fn from(message: String) -> Self {
        Erreur::Validation(message)
    }
}
