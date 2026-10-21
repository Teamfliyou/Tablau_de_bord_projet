//! Accès aux données : commandes Tauri (application Windows) ou API HTTP (version Web).
//!
//! L'interface est identique dans les deux cas ; seul ce module connaît le transport.

use crate::util;
use gloo_net::http::{Request, RequestBuilder};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tdb_shared::{
    Categories, Config, ConfigUpdate, Connexion, CoursDuJourEtat, CoursSemaineEtat, Devoir,
    DevoirEntree, DevoirModification, Filtres, Inscription, Plan, RapportMigration, ReponseAuth,
    Tag, TagEntree,
};
use wasm_bindgen::prelude::*;

const CLE_JETON: &str = "tdb_jeton";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Application Windows : toute la logique passe par les commandes Tauri (IPC).
    Tauri,
    /// Navigateur : API HTTP du serveur Axum, avec compte utilisateur.
    Web,
}

#[derive(Debug, Clone)]
pub struct ErreurApi {
    pub message: String,
    /// La session est absente ou expirée : l'interface doit afficher l'écran de connexion.
    pub non_auth: bool,
}

impl ErreurApi {
    fn nouvelle(message: impl Into<String>) -> Self {
        ErreurApi {
            message: message.into(),
            non_auth: false,
        }
    }
}

pub type Res<T> = Result<T, ErreurApi>;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"], js_name = invoke, catch)]
    async fn tauri_invoke(commande: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

pub fn mode() -> Mode {
    let dans_tauri = util::fenetre()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__TAURI_INTERNALS__")).ok())
        .is_some_and(|v| !v.is_undefined() && !v.is_null());
    if dans_tauri {
        Mode::Tauri
    } else {
        Mode::Web
    }
}

// ------------------------------------------------------------------- Tauri

async fn invoquer<T: DeserializeOwned>(commande: &str, args: Value) -> Res<T> {
    let js_args = js_sys::JSON::parse(&args.to_string())
        .map_err(|_| ErreurApi::nouvelle("Arguments invalides."))?;
    let reponse = tauri_invoke(commande, js_args).await.map_err(|e| {
        ErreurApi::nouvelle(
            e.as_string()
                .or_else(|| {
                    js_sys::Reflect::get(&e, &JsValue::from_str("message"))
                        .ok()
                        .and_then(|m| m.as_string())
                })
                .unwrap_or_else(|| "Erreur inattendue.".into()),
        )
    })?;
    let texte = js_sys::JSON::stringify(&reponse)
        .ok()
        .and_then(|s| s.as_string())
        .unwrap_or_else(|| "null".into());
    serde_json::from_str(&texte).map_err(|_| ErreurApi::nouvelle("Réponse inattendue."))
}

// -------------------------------------------------------------------- HTTP

pub fn jeton() -> Option<String> {
    util::lire_local(CLE_JETON)
}

pub fn oublier_jeton() {
    util::effacer_local(CLE_JETON);
}

fn constructeur(methode: &str, chemin: &str) -> RequestBuilder {
    let b = match methode {
        "POST" => Request::post(chemin),
        "PATCH" => Request::patch(chemin),
        "PUT" => Request::put(chemin),
        "DELETE" => Request::delete(chemin),
        _ => Request::get(chemin),
    };
    match jeton() {
        Some(j) => b.header("Authorization", &format!("Bearer {j}")),
        None => b,
    }
}

async fn http<T: DeserializeOwned>(methode: &str, chemin: &str, corps: Option<Value>) -> Res<T> {
    let b = constructeur(methode, chemin);
    let requete = match corps {
        Some(c) => b
            .json(&c)
            .map_err(|_| ErreurApi::nouvelle("Requête invalide."))?,
        None => b
            .build()
            .map_err(|_| ErreurApi::nouvelle("Requête invalide."))?,
    };
    let reponse = requete.send().await.map_err(|_| {
        ErreurApi::nouvelle("Impossible de joindre le serveur. Vérifie ta connexion.")
    })?;
    if !reponse.ok() {
        let statut = reponse.status();
        let detail = reponse
            .json::<Value>()
            .await
            .ok()
            .and_then(|v| v.get("detail").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| format!("Le serveur a répondu avec l'erreur {statut}."));
        return Err(ErreurApi {
            message: detail,
            non_auth: statut == 401,
        });
    }
    reponse
        .json::<T>()
        .await
        .map_err(|_| ErreurApi::nouvelle("Réponse inattendue du serveur."))
}

fn encoder(v: &str) -> String {
    js_sys::encode_uri_component(v)
        .as_string()
        .unwrap_or_default()
}

/// Paramètres de requête, avec clés répétées pour les listes (`groupes=1&groupes=2`).
fn requete(date: Option<&str>, f: &Filtres) -> String {
    let mut p: Vec<String> = vec![];
    if let Some(d) = date.filter(|d| !d.is_empty()) {
        p.push(format!("date={}", encoder(d)));
    }
    for (nom, valeurs) in [
        ("groupes", &f.groupes),
        ("types", &f.types),
        ("matieres", &f.matieres),
    ] {
        for v in valeurs {
            p.push(format!("{nom}={}", encoder(v)));
        }
    }
    if p.is_empty() {
        String::new()
    } else {
        format!("?{}", p.join("&"))
    }
}

// ----------------------------------------------------------- Authentification

async fn session(chemin: &str, corps: Value) -> Res<ReponseAuth> {
    let r: ReponseAuth = http("POST", chemin, Some(corps)).await?;
    util::ecrire_local(CLE_JETON, &r.token);
    Ok(r)
}

pub async fn connexion(c: &Connexion) -> Res<ReponseAuth> {
    session("/api/auth/login", json!(c)).await
}

pub async fn inscription(i: &Inscription) -> Res<ReponseAuth> {
    session("/api/auth/register", json!(i)).await
}

pub async fn profil_valide() -> bool {
    jeton().is_some() && http::<Value>("GET", "/api/auth/me", None).await.is_ok()
}

// ------------------------------------------------------------- Opérations

pub async fn obtenir_config() -> Res<Config> {
    match mode() {
        Mode::Tauri => invoquer("obtenir_config", json!({})).await,
        Mode::Web => http("GET", "/api/config", None).await,
    }
}

pub async fn enregistrer_config(maj: &ConfigUpdate) -> Res<Config> {
    match mode() {
        Mode::Tauri => invoquer("enregistrer_config", json!({ "maj": maj })).await,
        Mode::Web => http("POST", "/api/config", Some(json!(maj))).await,
    }
}

pub async fn obtenir_preferences() -> Res<Value> {
    match mode() {
        Mode::Tauri => invoquer("obtenir_preferences", json!({})).await,
        Mode::Web => http("GET", "/api/preferences", None).await,
    }
}

pub async fn enregistrer_preferences(preferences: &Value) -> Res<()> {
    match mode() {
        Mode::Tauri => {
            invoquer::<()>(
                "enregistrer_preferences",
                json!({ "preferences": preferences }),
            )
            .await
        }
        Mode::Web => http::<Value>("PUT", "/api/preferences", Some(preferences.clone()))
            .await
            .map(|_| ()),
    }
}

pub async fn cours_du_jour(date: Option<&str>, f: &Filtres) -> Res<CoursDuJourEtat> {
    match mode() {
        Mode::Tauri => invoquer("cours_du_jour", json!({ "date": date, "filtres": f })).await,
        Mode::Web => {
            http(
                "GET",
                &format!("/api/cours-du-jour{}", requete(date, f)),
                None,
            )
            .await
        }
    }
}

pub async fn cours_semaine(date: Option<&str>, f: &Filtres) -> Res<CoursSemaineEtat> {
    match mode() {
        Mode::Tauri => invoquer("cours_semaine", json!({ "date": date, "filtres": f })).await,
        Mode::Web => {
            http(
                "GET",
                &format!("/api/cours-semaine{}", requete(date, f)),
                None,
            )
            .await
        }
    }
}

pub async fn analyser_categories() -> Res<Categories> {
    match mode() {
        Mode::Tauri => invoquer("analyser_categories", json!({})).await,
        Mode::Web => http("GET", "/api/config/categories", None).await,
    }
}

pub async fn lister_devoirs() -> Res<Vec<Devoir>> {
    match mode() {
        Mode::Tauri => invoquer("lister_devoirs", json!({})).await,
        Mode::Web => http("GET", "/api/devoirs", None).await,
    }
}

pub async fn creer_devoir(d: &DevoirEntree) -> Res<Devoir> {
    match mode() {
        Mode::Tauri => invoquer("creer_devoir", json!({ "devoir": d })).await,
        Mode::Web => http("POST", "/api/devoirs", Some(json!(d))).await,
    }
}

pub async fn modifier_devoir(id: i64, m: &DevoirModification) -> Res<Devoir> {
    match mode() {
        Mode::Tauri => invoquer("modifier_devoir", json!({ "id": id, "modification": m })).await,
        Mode::Web => http("PATCH", &format!("/api/devoirs/{id}"), Some(json!(m))).await,
    }
}

pub async fn supprimer_devoir(id: i64) -> Res<()> {
    match mode() {
        Mode::Tauri => invoquer::<()>("supprimer_devoir", json!({ "id": id })).await,
        Mode::Web => http::<Value>("DELETE", &format!("/api/devoirs/{id}"), None)
            .await
            .map(|_| ()),
    }
}

pub async fn lister_tags() -> Res<Vec<Tag>> {
    match mode() {
        Mode::Tauri => invoquer("lister_tags", json!({})).await,
        Mode::Web => http("GET", "/api/tags", None).await,
    }
}

pub async fn creer_tag(t: &TagEntree) -> Res<Tag> {
    match mode() {
        Mode::Tauri => invoquer("creer_tag", json!({ "tag": t })).await,
        Mode::Web => http("POST", "/api/tags", Some(json!(t))).await,
    }
}

pub async fn modifier_tag(id: i64, t: &TagEntree) -> Res<Tag> {
    match mode() {
        Mode::Tauri => invoquer("modifier_tag", json!({ "id": id, "tag": t })).await,
        Mode::Web => http("PUT", &format!("/api/tags/{id}"), Some(json!(t))).await,
    }
}

pub async fn supprimer_tag(id: i64) -> Res<()> {
    match mode() {
        Mode::Tauri => invoquer::<()>("supprimer_tag", json!({ "id": id })).await,
        Mode::Web => http::<Value>("DELETE", &format!("/api/tags/{id}"), None)
            .await
            .map(|_| ()),
    }
}

/// Remplace les tags d'un devoir (ajout et retrait).
pub async fn definir_tags_devoir(id: i64, tags: &[i64]) -> Res<Devoir> {
    match mode() {
        Mode::Tauri => invoquer("definir_tags_devoir", json!({ "id": id, "tags": tags })).await,
        Mode::Web => http("PUT", &format!("/api/devoirs/{id}/tags"), Some(json!(tags))).await,
    }
}

pub async fn generer_plan(f: &Filtres) -> Res<Plan> {
    match mode() {
        Mode::Tauri => invoquer("generer_plan", json!({ "filtres": f })).await,
        Mode::Web => {
            http(
                "POST",
                &format!("/api/plan-revision{}", requete(None, f)),
                None,
            )
            .await
        }
    }
}

pub async fn version() -> Option<String> {
    match mode() {
        Mode::Tauri => invoquer::<String>("version_application", json!({}))
            .await
            .ok(),
        Mode::Web => http::<Value>("GET", "/api/health", None)
            .await
            .ok()
            .and_then(|v| v.get("version").and_then(Value::as_str).map(str::to_string)),
    }
}

// -------------------------------------------------- Fichiers (export / import)

/// Export JSON des devoirs. Windows : boîte « Enregistrer sous » ouverte côté Rust (retourne le chemin).
/// Web : téléchargement d'un fichier par le navigateur (retourne le nom du fichier).
pub async fn exporter_devoirs() -> Res<Option<String>> {
    match mode() {
        Mode::Tauri => invoquer("exporter_devoirs", json!({})).await,
        Mode::Web => {
            let liste: Vec<DevoirEntree> = http("GET", "/api/devoirs/export", None).await?;
            let texte = serde_json::to_string_pretty(&liste)
                .map_err(|_| ErreurApi::nouvelle("Export impossible."))?;
            let nom = format!("devoirs_{}.json", util::aujourdhui_iso());
            telecharger(&nom, &texte)?;
            Ok(Some(nom))
        }
    }
}

fn telecharger(nom: &str, contenu: &str) -> Res<()> {
    let erreur = || ErreurApi::nouvelle("Le téléchargement a échoué.");
    let parties = js_sys::Array::of1(&JsValue::from_str(contenu));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/json");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parties, &options)
        .map_err(|_| erreur())?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(|_| erreur())?;
    let document = util::fenetre()
        .and_then(|w| w.document())
        .ok_or_else(erreur)?;
    let lien: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .map_err(|_| erreur())?
        .dyn_into()
        .map_err(|_| erreur())?;
    lien.set_href(&url);
    lien.set_download(nom);
    lien.click();
    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(())
}

/// Import JSON. Windows : sélection du fichier côté Rust. Web : sélecteur de fichier du navigateur.
pub async fn importer_devoirs() -> Res<Option<usize>> {
    match mode() {
        Mode::Tauri => invoquer("importer_devoirs", json!({})).await,
        Mode::Web => {
            let Some(contenu) = choisir_fichier_json().await? else {
                return Ok(None);
            };
            let valeur: Value = serde_json::from_str(&contenu).map_err(|_| {
                ErreurApi::nouvelle("Fichier JSON invalide : une liste de devoirs est attendue.")
            })?;
            let r: Value = http("POST", "/api/devoirs/import", Some(valeur)).await?;
            Ok(Some(
                r.get("nb_importes").and_then(Value::as_u64).unwrap_or(0) as usize,
            ))
        }
    }
}

async fn choisir_fichier_json() -> Res<Option<String>> {
    let erreur = || ErreurApi::nouvelle("Impossible d'ouvrir le sélecteur de fichier.");
    let document = util::fenetre()
        .and_then(|w| w.document())
        .ok_or_else(erreur)?;
    let champ: web_sys::HtmlInputElement = document
        .create_element("input")
        .map_err(|_| erreur())?
        .dyn_into()
        .map_err(|_| erreur())?;
    champ.set_type("file");
    champ.set_accept(".json,application/json");
    let (emetteur, recepteur) = futures_channel::oneshot::channel::<()>();
    let emetteur = std::cell::RefCell::new(Some(emetteur));
    let rappel = Closure::<dyn Fn()>::new(move || {
        if let Some(e) = emetteur.borrow_mut().take() {
            let _ = e.send(());
        }
    });
    let _ = champ.add_event_listener_with_callback("change", rappel.as_ref().unchecked_ref());
    let _ = champ.add_event_listener_with_callback("cancel", rappel.as_ref().unchecked_ref());
    champ.click();
    let _ = recepteur.await;
    let Some(fichier) = champ.files().and_then(|f| f.get(0)) else {
        return Ok(None);
    };
    if fichier.size() > tdb_shared::IMPORT_MAX_OCTETS as f64 {
        return Err(ErreurApi::nouvelle(
            "Fichier invalide ou trop volumineux (2 Mo maximum).",
        ));
    }
    let texte = wasm_bindgen_futures::JsFuture::from(fichier.text())
        .await
        .map_err(|_| ErreurApi::nouvelle("Le fichier n'est pas un texte UTF-8 valide."))?;
    Ok(texte.as_string())
}

/// Import d'une ancienne base SQLite : réservé à l'application Windows.
pub async fn migrer_ancienne_base() -> Res<Option<RapportMigration>> {
    invoquer("migrer_ancienne_base", json!({})).await
}
