//! Onglet Semaine : emploi du temps du lundi au vendredi avec navigation de semaine en semaine.

use crate::icons::Icone;
use crate::pages::dashboard::maj_libelle;
use crate::state::Etat;
use crate::ui::*;
use crate::util;
use leptos::prelude::*;
use std::collections::BTreeMap;
use tdb_shared::Cours;

#[component]
pub fn Semaine() -> impl IntoView {
    let etat = expect_context::<Etat>();

    let jours = move || {
        let mut par_date: BTreeMap<String, Vec<Cours>> = BTreeMap::new();
        for c in etat.cours_semaine.get() {
            par_date.entry(c.date.clone()).or_default().push(c);
        }
        let aujourdhui = util::aujourdhui_iso();
        par_date
            .into_iter()
            .map(|(date, mut cours)| {
                cours.sort_by(|a, b| a.debut.cmp(&b.debut));
                (
                    date.clone(),
                    util::libelle_jour(&date),
                    date == aujourdhui,
                    cours,
                )
            })
            .collect::<Vec<_>>()
    };

    let sous_titre = move || {
        let (debut, fin) = etat.semaine_info.get();
        if debut.is_empty() {
            "Choisis une semaine pour naviguer dans ton calendrier".to_string()
        } else {
            format!(
                "Semaine du {} au {}",
                util::date_courte(&debut),
                util::date_courte(&fin)
            )
        }
    };

    view! {
        <section class="max-w-7xl mx-auto">
            <header class="flex flex-wrap items-center justify-between gap-4">
                <div>
                    <h2 class="text-2xl font-bold text-slate-900">"Emploi du temps de la semaine"</h2>
                    <p class="text-sm text-slate-500">{sous_titre}</p>
                </div>
                <div class="flex items-center gap-2 flex-wrap">
                    <button class=BOUTON_ICONE title="Semaine précédente" on:click=move |_| etat.changer_semaine(-7)>
                        <Icone nom="chevron-left" class="w-4 h-4" />
                    </button>
                    <input
                        type="date"
                        class="rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3 py-2 text-sm text-slate-700 shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        prop:value=move || etat.semaine_date.get()
                        on:change=move |ev| {
                            etat.semaine_date.set(event_target_value(&ev));
                            etat.charger_semaine();
                        }
                    />
                    <button class=BOUTON_ICONE title="Semaine suivante" on:click=move |_| etat.changer_semaine(7)>
                        <Icone nom="chevron-right" class="w-4 h-4" />
                    </button>
                    <button class=BOUTON_PRIMAIRE prop:disabled=move || etat.chargement_semaine.get() on:click=move |_| etat.charger_semaine()>
                        <span class="inline-flex" class:animate-spin=move || etat.chargement_semaine.get()>
                            <Icone nom="refresh" class="w-4 h-4" />
                        </span>
                        "Rafraîchir"
                    </button>
                </div>
            </header>

            {move || etat.erreur_semaine.get().map(|m| view! { <p class=format!("mt-6 {ALERTE_ERREUR}")>{m}</p> })}
            {move || etat.hors_ligne.get().then(|| view! {
                <p class=format!("mt-6 {ALERTE_HORS_LIGNE}")>
                    "Mode hors ligne : emploi du temps du dernier chargement réussi"
                    {maj_libelle(etat)}
                    "."
                </p>
            })}

            {move || {
                if etat.chargement_semaine.get() {
                    return view! {
                        <div class="mt-6 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
                            {(0..5).map(|_| view! { <div class=format!("rounded-3xl border border-white/40 {CARTE} p-5 h-64 animate-pulse")></div> }).collect_view()}
                        </div>
                    }.into_any();
                }
                let liste = jours();
                if liste.is_empty() {
                    return view! {
                        <div class=format!("mt-6 {VIDE} p-10")>
                            <p class="font-medium text-slate-600">"Aucune donnée pour cette semaine"</p>
                        </div>
                    }.into_any();
                }
                view! {
                    <div class="mt-6 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
                        {liste.into_iter().map(|(_, libelle, aujourdhui, cours)| view! {
                            <div class=format!(
                                "rounded-3xl border p-5 bg-white/60 backdrop-blur-2xl shadow-lg shadow-indigo-500/5 flex flex-col {}",
                                if aujourdhui { "border-indigo-300 ring-1 ring-indigo-200" } else { "border-white/40" }
                            )>
                                <div class="flex items-center justify-between gap-2">
                                    <h3 class="font-semibold text-slate-900">{libelle}</h3>
                                    {aujourdhui.then(|| view! {
                                        <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-bold text-white bg-indigo-600">"AUJOURD'HUI"</span>
                                    })}
                                </div>
                                <div class="mt-3 space-y-2 flex-1">
                                    {cours.into_iter().map(|c| view! {
                                        <div class=format!(
                                            "rounded-xl border p-3 {}",
                                            if aujourdhui { "border-indigo-100 bg-indigo-50/60" } else { "border-slate-100 bg-slate-50/60" }
                                        )>
                                            <p class="text-xs font-semibold text-indigo-600 tabular-nums">{format!("{} – {}", c.debut, c.fin)}</p>
                                            <p class="font-medium text-slate-800 text-sm mt-1 leading-snug">{c.titre.clone()}</p>
                                            <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-0.5 text-xs text-slate-500">
                                                <span class="inline-flex items-center gap-1">
                                                    <Icone nom="pin" class="w-3 h-3 text-slate-400" />
                                                    {if c.salle.is_empty() { "Salle non renseignée".to_string() } else { c.salle.clone() }}
                                                </span>
                                                {(!c.enseignant.is_empty()).then(|| view! {
                                                    <span class="inline-flex items-center gap-1">
                                                        <Icone nom="user" class="w-3 h-3 text-slate-400" />
                                                        {c.enseignant.clone()}
                                                    </span>
                                                })}
                                            </div>
                                        </div>
                                    }).collect_view()}
                                </div>
                            </div>
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </section>
    }
}
