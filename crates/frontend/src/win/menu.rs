//! Menu contextuel (clic droit) d'une carte.

use crate::icons::Icone;
use crate::state::{statut_libelle, Etat};
use leptos::prelude::*;

#[component]
pub fn MenuContextuel(x: f64, y: f64, id: i64) -> impl IntoView {
    let etat = expect_context::<Etat>();
    let fermer = move || etat.menu_ctx.set(None);
    // Évite de sortir de la fenêtre.
    let (l, h) = crate::util::fenetre()
        .map(|w| {
            (
                w.inner_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1280.0),
                w.inner_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(800.0),
            )
        })
        .unwrap_or((1280.0, 800.0));
    let gauche = x.min(l - 230.0).max(8.0);
    let haut = y.min(h - 200.0).max(8.0);
    let courant = etat.devoir_par_id(id).map(|d| d.statut).unwrap_or_default();

    view! {
        <div style="position:fixed;inset:0;z-index:79" on:click=move |_| fermer()
            on:contextmenu=move |ev| { ev.prevent_default(); fermer(); }></div>
        <div class="w-menu w-glass" role="menu" style=format!("left:{gauche}px;top:{haut}px")>
            <button type="button" role="menuitem" on:click=move |_| {
                if let Some(d) = etat.devoir_par_id(id) { etat.edition.set(Some(d)); }
                fermer();
            }>
                <Icone nom="open" />
                "Ouvrir"
                <span class="w-raccourci">"Entrée"</span>
            </button>
            <hr />
            {[("a_faire", "inbox"), ("en_cours", "clock"), ("termine", "check")].into_iter()
                .filter(|(s, _)| *s != courant)
                .map(|(s, icone)| view! {
                    <button type="button" role="menuitem" on:click=move |_| { etat.deplacer_vers(id, s); fermer(); }>
                        <Icone nom=icone />
                        {format!("Déplacer vers « {} »", statut_libelle(s))}
                    </button>
                }).collect_view()}
            <hr />
            <button type="button" role="menuitem" class="danger" on:click=move |_| { etat.supprimer_annulable(id); fermer(); }>
                <Icone nom="trash" />
                "Supprimer"
                <span class="w-raccourci">"Suppr"</span>
            </button>
        </div>
    }
}
