//! Gestion des tags : feuille de création, renommage, couleurs et suppression.

use crate::api;
use crate::icons::Icone;
use crate::state::Etat;
use leptos::prelude::*;
use leptos::task::spawn_local;
use tdb_shared::{Tag, TagEntree};

/// Palette de tags : teintes moyennes, rendues en pastel par le CSS (mélange avec la surface).
pub const COULEURS_TAG: [&str; 8] = [
    "#ff5d5d", "#ff9f1c", "#e6b800", "#30b455", "#19b5a5", "#0a84ff", "#7c6bff", "#ee4f9b",
];

/// Sélecteur de couleur : palette + couleur libre.
#[component]
pub fn CouleursTag(valeur: RwSignal<String>) -> impl IntoView {
    view! {
        <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap" role="radiogroup" aria-label="Couleur du tag">
            {COULEURS_TAG.iter().map(|c| {
                let c = *c;
                view! {
                    <button type="button" role="radio" aria-label=format!("Couleur {c}")
                        aria-checked=move || if valeur.get().eq_ignore_ascii_case(c) { "true" } else { "false" }
                        style=move || format!("width:20px;height:20px;border-radius:50%;border:0;background:{c};cursor:default;box-shadow:{}",
                            if valeur.get().eq_ignore_ascii_case(c) { "0 0 0 2px var(--surface),0 0 0 4px var(--accent)" } else { "inset 0 0 0 .5px rgba(0,0,0,.25)" })
                        on:click=move |_| valeur.set(c.to_string())></button>
                }
            }).collect_view()}
            <input type="color" aria-label="Couleur personnalisée" style="width:24px;height:24px;border:0;padding:0;background:none"
                prop:value=move || valeur.get() on:input=move |ev| valeur.set(event_target_value(&ev)) />
        </div>
    }
}

#[component]
pub fn GestionTags() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let fermer = move || etat.gestion_tags.set(false);
    let nom = RwSignal::new(String::new());
    let couleur = RwSignal::new(COULEURS_TAG[5].to_string());
    let erreur = RwSignal::new(String::new());

    let creer = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let entree = TagEntree { nom: nom.get_untracked(), couleur: couleur.get_untracked() };
        spawn_local(async move {
            match api::creer_tag(&entree).await {
                Ok(_) => {
                    nom.set(String::new());
                    erreur.set(String::new());
                    etat.charger_devoirs_async().await;
                }
                Err(e) => erreur.set(etat.message(e)),
            }
        });
    };

    view! {
        <div class="w-voile centre" on:click=move |_| fermer()>
            <div class="w-feuille w-glass" role="dialog" aria-modal="true" aria-labelledby="tags-titre" on:click=|ev| ev.stop_propagation()>
                <h2 id="tags-titre">"Tags"</h2>
                <p class="w-aide">"Crée des tags colorés pour classer tes devoirs, puis filtre-les depuis la barre latérale."</p>
                <form on:submit=creer style="display:flex;flex-direction:column;gap:10px">
                    <div style="display:flex;gap:8px">
                        <input class="w-champ" type="text" required maxlength="30" placeholder="Nouveau tag (ex. CM)" aria-label="Nom du tag" autofocus
                            prop:value=move || nom.get() on:input=move |ev| nom.set(event_target_value(&ev)) />
                        <button type="submit" class="w-btn w-btn-primaire"><Icone nom="plus" />"Ajouter"</button>
                    </div>
                    <CouleursTag valeur=couleur />
                </form>
                {move || { let m = erreur.get(); (!m.is_empty()).then(|| view! { <p role="alert" style="color:var(--danger);font-size:12px">{m}</p> }) }}
                <ul style="list-style:none;margin:16px 0 0;padding:0;display:flex;flex-direction:column;gap:6px">
                    {move || {
                        let liste = etat.tags.get();
                        if liste.is_empty() {
                            return view! { <li class="w-vide" style="padding:18px"><p>"Aucun tag pour l'instant."</p></li> }.into_any();
                        }
                        liste.into_iter().map(|t| ligne_tag(etat, t, erreur)).collect_view().into_any()
                    }}
                </ul>
                <div class="w-feuille-pied"><button type="button" class="w-btn" on:click=move |_| fermer()>"Terminé"</button></div>
            </div>
        </div>
    }
}

fn ligne_tag(etat: Etat, t: Tag, erreur: RwSignal<String>) -> impl IntoView {
    let id = t.id;
    let nom = RwSignal::new(t.nom);
    let couleur = RwSignal::new(t.couleur);
    let enregistrer = move || {
        let entree = TagEntree { nom: nom.get_untracked(), couleur: couleur.get_untracked() };
        spawn_local(async move {
            match api::modifier_tag(id, &entree).await {
                Ok(_) => { erreur.set(String::new()); etat.charger_devoirs_async().await; }
                Err(e) => erreur.set(etat.message(e)),
            }
        });
    };
    let supprimer = move |_| {
        spawn_local(async move {
            match api::supprimer_tag(id).await {
                Ok(()) => {
                    if etat.filtre_tag.get_untracked() == Some(id) { etat.filtre_tag.set(None); }
                    erreur.set(String::new());
                    etat.charger_devoirs_async().await;
                }
                Err(e) => erreur.set(etat.message(e)),
            }
        });
    };
    view! {
        <li style="display:flex;align-items:center;gap:8px;padding:6px;border-radius:var(--r-sm);background:var(--surface-2)">
            <input type="color" aria-label="Couleur" style="width:26px;height:26px;border:0;padding:0;background:none;flex:none"
                prop:value=move || couleur.get()
                on:input=move |ev| couleur.set(event_target_value(&ev))
                on:change=move |_| enregistrer() />
            <input class="w-champ" type="text" required maxlength="30" aria-label="Nom du tag"
                prop:value=move || nom.get() on:input=move |ev| nom.set(event_target_value(&ev))
                on:blur=move |_| enregistrer()
                on:keydown=move |ev| if ev.key() == "Enter" { enregistrer(); } />
            <button type="button" class="w-btn w-btn-icone" aria-label="Supprimer le tag" title="Supprimer le tag (les devoirs sont conservés)" on:click=supprimer>
                <Icone nom="trash" />
            </button>
        </li>
    }
}
