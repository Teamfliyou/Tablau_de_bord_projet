//! Interface de l'application Windows : coquille type app Mac (barre latérale, barre d'outils,
//! contenu, inspecteur), Kanban, import guidé et palette de commandes.
//! La version Web conserve l'interface historique (`app.rs`).

pub mod import;
pub mod inspecteur;
pub mod kanban;
pub mod menu;
pub mod palette;
pub mod shell;
pub mod tags;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// État d'interface partagé par les composants de la coquille.
#[derive(Clone, Copy)]
pub struct Ui {
    /// Colonne dont le champ de création rapide est ouvert.
    pub ajout_col: RwSignal<Option<&'static str>>,
    pub recherche_ref: NodeRef<leptos::html::Input>,
}

/// Libellé d'un raccourci : ⌘ sur Mac, Ctrl sur Windows.
pub fn raccourci(touche: &str) -> String {
    let mac = web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .is_some_and(|ua| ua.contains("Mac"));
    if mac {
        format!("⌘{touche}")
    } else {
        format!("Ctrl+{touche}")
    }
}

/// Vrai si l'événement vient d'un champ de saisie (les raccourcis à une touche sont alors ignorés).
pub fn dans_saisie(ev: &web_sys::KeyboardEvent) -> bool {
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        .is_some_and(|e| {
            matches!(e.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT")
                || e.get_attribute("contenteditable").is_some()
        })
}

/// Donne le focus à l'élément correspondant au sélecteur CSS.
pub fn focaliser(selecteur: &str) {
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector(selecteur).ok().flatten())
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = el.focus();
    }
}
