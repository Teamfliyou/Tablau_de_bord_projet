//! Google Agenda — OAuth local + synchronisation des échéances des devoirs.
//! Les identifiants restent exclusivement dans le coffre Windows.
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::{State, AppHandle};
use tauri_plugin_dialog::DialogExt;
use tdb_core::{Erreur, Resultat, Service};
use std::{sync::Arc, io::{Read, Write}, net::TcpListener, time::{Duration, Instant}};
use keyring::Entry;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};

const SERVICE: &str = "TableauDeBordEtudiant";
const CREDENTIAL: &str = "google_calendar_client";
const REFRESH: &str = "google_calendar_refresh";
const SCOPE: &str = "https://www.googleapis.com/auth/calendar.events";
type Etat<'a> = State<'a, Arc<Service>>;

#[derive(Deserialize)]
struct Installed { client_id: String, client_secret: String, #[serde(default)] auth_uri: String, #[serde(default)] token_uri: String }
#[derive(Deserialize)]
struct ClientFile { installed: Option<Installed> }
#[derive(Deserialize)]
struct Token { access_token: String, refresh_token: Option<String> }
fn erreur(msg: &str) -> Erreur { Erreur::Validation(msg.to_string()) }
fn coffre(nom: &str) -> Resultat<Entry> {
    Entry::new(SERVICE, nom).map_err(|_| erreur("Coffre Windows indisponible."))
}
fn lire(nom: &str) -> Resultat<Option<String>> {
    match coffre(nom)?.get_password() {
        Ok(s) => Ok(Some(s)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err(erreur("Impossible de lire les identifiants Windows.")),
    }
}
fn ecrire(nom: &str, valeur: &str) -> Resultat<()> {
    coffre(nom)?.set_password(valeur).map_err(|_| erreur("Impossible de conserver la connexion Google."))
}
fn client() -> Resultat<Installed> {
    let s = lire(CREDENTIAL)?.ok_or_else(|| erreur("Sélectionne ton fichier client_secret Google."))?;
    let json: ClientFile = serde_json::from_str(&s).map_err(|_| erreur("Configuration Google invalide."))?;
    json.installed.ok_or_else(|| erreur("Il faut un client OAuth de type Application de bureau (installed)."))
}
#[tauri::command]
pub fn statut_google_agenda() -> Resultat<String> {
    if lire(CREDENTIAL)?.is_none() { Ok("Configuration manquante".into()) }
    else if lire(REFRESH)?.is_none() { Ok("Autorisation Google nécessaire".into()) }
    else { Ok("Google Agenda connecté".into()) }
}
#[tauri::command]
pub async fn importer_client_google(app: AppHandle) -> Resultat<String> {
    let Some(file) = app.dialog().file().add_filter("Configuration OAuth Google", &["json"]).blocking_pick_file() else {
        return Ok("Import annulé".into())
    };
    let path = file.into_path().map_err(|_| erreur("Fichier non accessible."))?;
    let metadata = std::fs::metadata(&path).map_err(|_| erreur("Fichier illisible."))?;
    if metadata.len() > 65536 { return Err(erreur("Fichier JSON trop volumineux.")) }
    let data = std::fs::read_to_string(&path).map_err(|_| erreur("Fichier illisible."))?;
    let json: ClientFile = serde_json::from_str(&data).map_err(|_| erreur("Ce fichier n'est pas un JSON OAuth Google valide."))?;
    let installed = json.installed.ok_or_else(|| erreur("Crée dans Google Cloud un client OAuth « Application de bureau » (installed)."))?;
    if installed.client_id.trim().is_empty() || installed.client_secret.trim().is_empty() {
        return Err(erreur("Identifiants OAuth incomplets."))
    }
    // La configuration est écrite au coffre Windows, jamais dans la base SQLite.
    ecrire(CREDENTIAL, &data)?;
    Ok("Configuration Google importée dans le coffre Windows".into())
}
fn nonce() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
#[tauri::command]
pub async fn connecter_google_agenda() -> Resultat<String> {
    let c = client()?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|_| erreur("Impossible d'ouvrir la connexion locale OAuth."))?;
    let port = listener.local_addr().map_err(|_| erreur("Adresse OAuth indisponible."))?.port();
    let redirect = format!("http://127.0.0.1:{port}/");
    let state = nonce();
    let verifier = nonce();
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let mut url = reqwest::Url::parse("https://accounts.google.com/o/oauth2/v2/auth").map_err(|_| erreur("Adresse OAuth invalide."))?;
    url.query_pairs_mut()
        .append_pair("client_id", &c.client_id)
        .append_pair("redirect_uri", &redirect)
        .append_pair("response_type", "code")
        .append_pair("scope", SCOPE)
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("state", &state)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256");
    webbrowser::open(url.as_str()).map_err(|_| erreur("Impossible d'ouvrir le navigateur Google."))?;
    let code = tauri::async_runtime::spawn_blocking(move || -> Resultat<String> {
        listener.set_nonblocking(true).map_err(|_| erreur("Réception OAuth impossible."))?;
        let deadline = Instant::now() + Duration::from_secs(180);
        loop {
            if Instant::now() >= deadline { return Err(erreur("Connexion Google expirée (3 minutes).")) }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                    let mut bytes = [0u8; 8192];
                    let size = stream.read(&mut bytes).map_err(|_| erreur("Réponse OAuth illisible."))?;
                    let request = String::from_utf8_lossy(&bytes[..size]);
                    let target = request.split_whitespace().nth(1).ok_or_else(|| erreur("Retour OAuth invalide."))?;
                    let url = reqwest::Url::parse(&format!("http://localhost{target}")).map_err(|_| erreur("Retour OAuth invalide."))?;
                    let params: std::collections::HashMap<String, String> = url.query_pairs().into_owned().collect();
                    let ok = params.get("state") == Some(&state) && params.contains_key("code");
                    let body = if ok {"Connexion réussie. Tu peux fermer cet onglet et retourner à l'application."} else {"La connexion Google a échoué. Reviens dans l'application."};
                    let reply = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body);
                    let _ = stream.write_all(reply.as_bytes());
                    if !ok { return Err(erreur("Autorisation Google refusée ou état de sécurité invalide.")) }
                    return Ok(params.get("code").cloned().unwrap_or_default())
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(100)),
                Err(_) => return Err(erreur("Erreur du retour OAuth.")),
            }
        }
    }).await.map_err(|_| erreur("Connexion Google interrompue."))??;
    let token = reqwest::Client::new().post("https://oauth2.googleapis.com/token").form(&[
        ("code", code.as_str()), ("client_id", c.client_id.as_str()),
        ("client_secret", c.client_secret.as_str()), ("redirect_uri", redirect.as_str()),
        ("grant_type", "authorization_code"), ("code_verifier", verifier.as_str())
    ]).send().await.map_err(|_| erreur("Connexion au serveur Google impossible."))?;
    if !token.status().is_success() { return Err(erreur("Google a refusé l'autorisation. Vérifie la configuration OAuth.")) }
    let token: Token = token.json().await.map_err(|_| erreur("Réponse Google invalide."))?;
    let refresh = token.refresh_token.ok_or_else(|| erreur("Google n'a pas fourni d'autorisation permanente."))?;
    ecrire(REFRESH, &refresh)?;
    Ok("Google Agenda connecté avec succès.".into())
}
async fn access_token() -> Resultat<String> {
    let c = client()?;
    let refresh = lire(REFRESH)?.ok_or_else(|| erreur("Connecte d'abord ton compte Google."))?;
    let response = reqwest::Client::new().post("https://oauth2.googleapis.com/token").form(&[
        ("refresh_token", refresh.as_str()), ("client_id", c.client_id.as_str()),
        ("client_secret", c.client_secret.as_str()), ("grant_type", "refresh_token")
    ]).send().await.map_err(|_| erreur("Impossible de joindre Google."))?;
    if !response.status().is_success() { return Err(erreur("Autorisation Google expirée ou révoquée : reconnecte ton compte.")) }
    let token: Token = response.json().await.map_err(|_| erreur("Réponse Google invalide."))?;
    Ok(token.access_token)
}
#[tauri::command]
pub async fn synchroniser_google_agenda(s: Etat<'_>) -> Resultat<String> {
    let devoirs = s.db.lister_devoirs()?;
    let access = access_token().await?;
    let http = reqwest::Client::new();
    let mut ok = 0;
    for d in devoirs {
        if !valide_date(&d.echeance) { continue }
        let google_id = format!("tdb{:x}", Sha256::digest(format!("devoir:{}", d.id).as_bytes()));
        // Identifiant stable : PUT idempotent par événement. Ne touche pas aux événements personnels.
        let body = json!({
            "id": google_id,
            "summary": format!("{} — {}", d.matiere, d.titre),
            "description": d.description,
            "start": { "date": d.echeance },
            "end": { "date": lendemain(&d.echeance)? }
        });
        let url = "https://www.googleapis.com/calendar/v3/calendars/primary/events";
        let result = http.post(url).bearer_auth(&access).json(&body).send().await.map_err(|_| erreur("Connexion Google Agenda interrompue."))?;
        let status = result.status().as_u16();
        if status == 409 {
            let updated = http.put(format!("{url}/{google_id}")).bearer_auth(&access).json(&body).send().await.map_err(|_| erreur("Modification Google Agenda impossible."))?;
            if !updated.status().is_success() { return Err(erreur("Impossible de modifier un événement Google Agenda.")) }
        } else if !result.status().is_success() {
            return Err(erreur("Google Agenda a refusé un événement. Vérifie les autorisations d'écriture."))
        }
        ok += 1;
    }
    Ok(format!("{ok} devoir(s) synchronisé(s) dans Google Agenda."))
}
fn valide_date(s: &str) -> bool { chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() }
fn lendemain(s: &str) -> Resultat<String> {
    let date = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| erreur("Date de devoir invalide."))?;
    Ok(date.succ_opt().ok_or_else(|| erreur("Date limite invalide."))?.format("%Y-%m-%d").to_string())
}
