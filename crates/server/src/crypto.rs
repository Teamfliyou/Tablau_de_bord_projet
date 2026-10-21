//! Chiffrement des clés Gemini au repos (AES-256-GCM) et génération de secrets.

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use sha2::{Digest, Sha256};

const PREFIXE: &str = "enc:v1:";

fn chiffre(secret: &[u8]) -> Aes256Gcm {
    let mut h = Sha256::new();
    h.update(b"tdb-gemini-v1");
    h.update(secret);
    Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&h.finalize()))
}

/// Chiffre une valeur ; la sortie est un texte `enc:v1:<base64(nonce || chiffré)>`.
pub fn chiffrer(secret: &[u8], clair: &str) -> Result<String, String> {
    if clair.is_empty() {
        return Ok(String::new());
    }
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let chiffre_ = chiffre(secret)
        .encrypt(&nonce, clair.as_bytes())
        .map_err(|_| "Chiffrement impossible.".to_string())?;
    let mut brut = nonce.to_vec();
    brut.extend_from_slice(&chiffre_);
    Ok(format!("{PREFIXE}{}", STANDARD.encode(brut)))
}

/// Déchiffre une valeur. Une valeur sans préfixe vient d'une ancienne base (clair) et est rendue telle quelle.
pub fn dechiffrer(secret: &[u8], stocke: &str) -> Option<String> {
    let Some(encode) = stocke.strip_prefix(PREFIXE) else {
        return Some(stocke.to_string());
    };
    let brut = STANDARD.decode(encode).ok()?;
    if brut.len() < 13 {
        return None;
    }
    let (nonce, contenu) = brut.split_at(12);
    let clair = chiffre(secret)
        .decrypt(Nonce::from_slice(nonce), contenu)
        .ok()?;
    String::from_utf8(clair).ok()
}

/// Secret aléatoire de 48 octets encodé en base64.
pub fn secret_aleatoire() -> String {
    let a = Aes256Gcm::generate_key(&mut OsRng);
    let b = Aes256Gcm::generate_key(&mut OsRng);
    let mut brut = a.to_vec();
    brut.extend_from_slice(&b[..16]);
    STANDARD.encode(brut)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aller_retour_et_ancien_format_en_clair() {
        let chiffre_ = chiffrer(b"secret", "AIzaCleSecrete").expect("chiffrement");
        assert!(chiffre_.starts_with("enc:v1:"));
        assert!(!chiffre_.contains("AIza"));
        assert_eq!(
            dechiffrer(b"secret", &chiffre_).as_deref(),
            Some("AIzaCleSecrete")
        );
        assert_eq!(dechiffrer(b"autre", &chiffre_), None);
        assert_eq!(
            dechiffrer(b"secret", "cle-en-clair").as_deref(),
            Some("cle-en-clair")
        );
    }
}
