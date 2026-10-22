//! Vérifie la signature des mises à jour avec la clé publique réellement intégrée à l'application.
//!
//! - Toujours : la clé publique de `tauri.conf.json` est une vraie clé minisign valide.
//! - En CI (tag de release) : `TDB_VERIF_FICHIER` et `TDB_VERIF_SIGNATURE` désignent l'installateur
//!   et sa signature `.sig` ; la signature doit correspondre à la clé publique embarquée.

use base64::Engine;
use minisign_verify::{PublicKey, Signature};

fn decoder_base64(texte: &str) -> String {
    let octets = base64::engine::general_purpose::STANDARD
        .decode(texte.trim())
        .expect("base64 invalide");
    String::from_utf8(octets).expect("UTF-8 invalide")
}

fn cle_publique() -> PublicKey {
    let conf: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).expect("tauri.conf.json invalide");
    let pubkey = conf["plugins"]["updater"]["pubkey"]
        .as_str()
        .expect("plugins.updater.pubkey absent de tauri.conf.json");
    PublicKey::decode(&decoder_base64(pubkey)).expect("clé publique minisign invalide")
}

#[test]
fn cle_publique_integree_valide() {
    let _ = cle_publique();
}

#[test]
fn endpoint_github_releases_en_https() {
    let conf: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    let endpoints = conf["plugins"]["updater"]["endpoints"].as_array().unwrap();
    assert!(!endpoints.is_empty());
    for e in endpoints {
        let url = e.as_str().unwrap();
        assert!(url.starts_with("https://github.com/Teamfliyou/"), "{url}");
        assert!(
            url.ends_with("/releases/latest/download/latest.json"),
            "{url}"
        );
    }
}

#[test]
fn identifiant_de_donnees_inchange() {
    let conf: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    assert_eq!(conf["identifier"], "fr.teamfliyou.tableaudebord");
}

#[test]
fn signature_de_l_installateur_publie() {
    let (Ok(fichier), Ok(sig)) = (
        std::env::var("TDB_VERIF_FICHIER"),
        std::env::var("TDB_VERIF_SIGNATURE"),
    ) else {
        eprintln!("Variables TDB_VERIF_* absentes : vérification de fichier ignorée.");
        return;
    };
    let contenu = std::fs::read(fichier).expect("installateur illisible");
    let signature = Signature::decode(&decoder_base64(
        &std::fs::read_to_string(sig).expect("signature illisible"),
    ))
    .expect("signature minisign invalide");
    cle_publique()
        .verify(&contenu, &signature, false)
        .expect("la signature ne correspond pas à la clé publique intégrée");
}
