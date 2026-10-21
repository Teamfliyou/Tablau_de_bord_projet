//! Utilitaires navigateur : dates en français, stockage local, presse-papiers, thème.

use js_sys::{Date, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};

pub fn fenetre() -> Option<web_sys::Window> {
    web_sys::window()
}

fn stockage() -> Option<web_sys::Storage> {
    fenetre()?.local_storage().ok().flatten()
}

pub fn lire_local(cle: &str) -> Option<String> {
    stockage()?.get_item(cle).ok().flatten()
}

pub fn ecrire_local(cle: &str, valeur: &str) {
    if let Some(s) = stockage() {
        let _ = s.set_item(cle, valeur);
    }
}

pub fn effacer_local(cle: &str) {
    if let Some(s) = stockage() {
        let _ = s.remove_item(cle);
    }
}

/// Date locale au format AAAA-MM-JJ.
pub fn iso_local(d: &Date) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        d.get_full_year(),
        d.get_month() + 1,
        d.get_date()
    )
}

pub fn aujourdhui_iso() -> String {
    iso_local(&Date::new_0())
}

/// Heure locale au format HH:MM.
pub fn heure_actuelle() -> String {
    let d = Date::new_0();
    format!("{:02}:{:02}", d.get_hours(), d.get_minutes())
}

/// Date AAAA-MM-JJ à minuit heure locale.
fn date_locale(iso: &str) -> Date {
    Date::new(&JsValue::from_str(&format!("{iso}T00:00:00")))
}

pub fn decaler_jours(iso: &str, jours: i32) -> String {
    let d = date_locale(iso);
    d.set_date((d.get_date() as i32 + jours) as u32);
    iso_local(&d)
}

fn options(champs: &[(&str, &str)]) -> Object {
    let o = Object::new();
    for (k, v) in champs {
        let _ = Reflect::set(&o, &JsValue::from_str(k), &JsValue::from_str(v));
    }
    o
}

fn formater(d: &Date, champs: &[(&str, &str)]) -> String {
    d.to_locale_date_string("fr-FR", &options(champs))
        .as_string()
        .unwrap_or_default()
}

/// « mercredi 21 octobre 2026 »
pub fn date_longue(d: &Date) -> String {
    formater(
        d,
        &[
            ("weekday", "long"),
            ("day", "numeric"),
            ("month", "long"),
            ("year", "numeric"),
        ],
    )
}

/// « 21 octobre »
pub fn date_courte(iso: &str) -> String {
    formater(&date_locale(iso), &[("day", "numeric"), ("month", "long")])
}

/// « 21 oct. »
pub fn date_tres_courte(iso: &str) -> String {
    formater(&date_locale(iso), &[("day", "numeric"), ("month", "short")])
}

/// « Mercredi 21 oct. »
pub fn libelle_jour(iso: &str) -> String {
    let s = formater(
        &date_locale(iso),
        &[("weekday", "long"), ("day", "numeric"), ("month", "short")],
    );
    let mut c = s.chars();
    match c.next() {
        Some(p) => p.to_uppercase().collect::<String>() + c.as_str(),
        None => s,
    }
}

/// « 21 octobre, 14:05 » à partir d'une date RFC 3339.
pub fn date_heure(rfc3339: &str) -> String {
    let d = Date::new(&JsValue::from_str(rfc3339));
    if d.get_time().is_nan() {
        return String::new();
    }
    d.to_locale_string(
        "fr-FR",
        &options(&[
            ("day", "numeric"),
            ("month", "long"),
            ("hour", "2-digit"),
            ("minute", "2-digit"),
        ]),
    )
    .as_string()
    .unwrap_or_default()
}

/// Jours entre aujourd'hui et l'échéance (négatif si en retard).
pub fn jours_restants(echeance: &str) -> i32 {
    let e = date_locale(echeance).get_time();
    let a = date_locale(&aujourdhui_iso()).get_time();
    ((e - a) / 86_400_000.0).round() as i32
}

pub fn prefere_sombre() -> bool {
    fenetre()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
        .is_some_and(|m| m.matches())
}

pub fn appliquer_classe_sombre(sombre: bool) {
    if let Some(racine) = fenetre()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = racine.class_list().toggle_with_force("dark", sombre);
    }
}

/// Appelle `rappel` quand le thème du système change.
pub fn surveiller_theme_systeme(rappel: impl Fn() + 'static) {
    let Some(m) =
        fenetre().and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
    else {
        return;
    };
    let cb = wasm_bindgen::closure::Closure::<dyn Fn()>::new(rappel);
    let _ = m.add_event_listener_with_callback("change", cb.as_ref().unchecked_ref());
    cb.forget();
}

/// Copie du texte dans le presse-papiers (API `navigator.clipboard`, sans binding instable).
pub async fn copier(texte: &str) -> bool {
    let Some(nav) = fenetre().map(|w| w.navigator()) else {
        return false;
    };
    let Ok(presse_papiers) = Reflect::get(&nav, &JsValue::from_str("clipboard")) else {
        return false;
    };
    if presse_papiers.is_undefined() || presse_papiers.is_null() {
        return false;
    }
    let Ok(ecrire) = Reflect::get(&presse_papiers, &JsValue::from_str("writeText"))
        .and_then(|f| f.dyn_into::<js_sys::Function>())
    else {
        return false;
    };
    let Ok(promesse) = ecrire.call1(&presse_papiers, &JsValue::from_str(texte)) else {
        return false;
    };
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promesse))
        .await
        .is_ok()
}

/// Ajoute ou retire une valeur d'une liste (bascule d'un filtre).
pub fn basculer(liste: &mut Vec<String>, valeur: &str) {
    match liste.iter().position(|v| v == valeur) {
        Some(i) => {
            liste.remove(i);
        }
        None => liste.push(valeur.to_string()),
    }
}
