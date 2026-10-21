//! Onglet Devoirs : formulaire d'ajout, tableau Kanban, import / export JSON.

use crate::icons::Icone;
use crate::state::{
    couleur_echeance, libelle_echeance, type_badge, type_libelle, Etat, COLONNES, TYPES_DEVOIR,
};
use crate::ui::*;
use leptos::prelude::*;
use tdb_shared::{Devoir, DevoirEntree};

#[component]
pub fn Kanban() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let type_ = RwSignal::new("devoir".to_string());
    let titre = RwSignal::new(String::new());
    let matiere = RwSignal::new(String::new());
    let echeance = RwSignal::new(String::new());
    let statut = RwSignal::new("a_faire".to_string());

    let ajouter = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let t = titre.get_untracked().trim().to_string();
        if t.is_empty() {
            return;
        }
        etat.ajouter_devoir(
            DevoirEntree {
                titre: t,
                matiere: matiere.get_untracked().trim().to_string(),
                echeance: echeance.get_untracked(),
                type_: type_.get_untracked(),
                statut: statut.get_untracked(),
            },
            move || {
                titre.set(String::new());
                matiere.set(String::new());
                echeance.set(String::new());
                type_.set("devoir".into());
                statut.set("a_faire".into());
            },
        );
    };

    view! {
        <section class="max-w-7xl mx-auto">
            <header class="flex flex-wrap items-center justify-between gap-4">
                <div>
                    <h2 class="text-2xl font-bold text-slate-900">"Devoirs"</h2>
                    <p class="text-sm text-slate-500">"Tableau Kanban : glisse tes devoirs entre les statuts."</p>
                </div>
                <div class="flex flex-wrap items-center gap-2">
                    <button type="button" class=BOUTON_PILULE title="Sauvegarder tous les devoirs au format JSON" on:click=move |_| etat.exporter_devoirs()>
                        <Icone nom="download" class="w-4 h-4" />
                        "Exporter"
                    </button>
                    <button type="button" class=BOUTON_PILULE title="Restaurer des devoirs depuis un fichier JSON" on:click=move |_| etat.importer_devoirs()>
                        <Icone nom="upload" class="w-4 h-4" />
                        "Importer"
                    </button>
                </div>
            </header>

            <form class=format!("mt-6 {CARTE} p-5 space-y-4") on:submit=ajouter>
                <div class="flex flex-wrap items-center gap-2">
                    <span class="text-sm font-medium text-slate-500 mr-1">"Type :"</span>
                    {TYPES_DEVOIR.iter().map(|(id, libelle)| {
                        let id = *id;
                        view! {
                            <button
                                type="button"
                                class=move || format!(
                                    "inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl text-sm font-medium border transition {}",
                                    if type_.get() == id { "bg-indigo-600 text-white border-indigo-600 shadow-sm" } else { "bg-white/60 backdrop-blur text-slate-600 border-white/50 hover:border-indigo-300" }
                                )
                                on:click=move |_| type_.set(id.to_string())
                            >
                                <Icone nom=id class="w-4 h-4 shrink-0" />
                                {*libelle}
                            </button>
                        }
                    }).collect_view()}
                </div>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
                    <input class=CHAMP type="text" required maxlength="200" placeholder="Intitulé…"
                        prop:value=move || titre.get() on:input=move |ev| titre.set(event_target_value(&ev)) />
                    <input class=CHAMP type="text" maxlength="120" placeholder="Matière"
                        prop:value=move || matiere.get() on:input=move |ev| matiere.set(event_target_value(&ev)) />
                    <input class=format!("{CHAMP} text-slate-600") type="date"
                        prop:value=move || echeance.get() on:input=move |ev| echeance.set(event_target_value(&ev)) />
                    <select class=format!("{CHAMP} text-slate-700")
                        prop:value=move || statut.get() on:change=move |ev| statut.set(event_target_value(&ev))>
                        {COLONNES.iter().map(|(id, libelle, _)| view! { <option value=*id>{*libelle}</option> }).collect_view()}
                    </select>
                </div>
                <button type="submit" class="inline-flex items-center justify-center gap-2 px-5 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium shadow-sm transition">
                    <Icone nom="plus" class="w-4 h-4" />
                    "Ajouter"
                </button>
            </form>

            {move || etat.erreur_devoirs.get().map(|m| view! { <p class=format!("mt-4 {ALERTE_ERREUR}")>{m}</p> })}

            <div class="mt-6 grid grid-cols-1 md:grid-cols-3 gap-5">
                {COLONNES.iter().map(|(id, libelle, badge)| {
                    let id = *id;
                    let dans_colonne = move || {
                        etat.devoirs.get().into_iter().filter(|d| d.statut == id).collect::<Vec<Devoir>>()
                    };
                    view! {
                        <div class="rounded-3xl border border-white/40 bg-white/60 backdrop-blur-2xl shadow-lg shadow-indigo-500/5 p-4 flex flex-col">
                            <div class="flex items-center justify-between px-1">
                                <span class=format!("inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-sm font-semibold {badge}")>
                                    <Icone nom="dot" class="w-4 h-4" />
                                    {*libelle}
                                </span>
                                <span class="text-sm font-bold text-slate-400 tabular-nums">{move || dans_colonne().len()}</span>
                            </div>
                            <div class="mt-4 space-y-3 flex-1">
                                {move || {
                                    let liste = dans_colonne();
                                    if liste.is_empty() {
                                        return view! {
                                            <div class="rounded-xl border border-dashed border-white/60 p-6 text-center">
                                                <p class="text-xs text-slate-400">"Aucun devoir"</p>
                                            </div>
                                        }.into_any();
                                    }
                                    liste.into_iter().map(|d| carte_devoir(etat, d)).collect_view().into_any()
                                }}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </section>
    }
}

fn carte_devoir(etat: Etat, d: Devoir) -> impl IntoView {
    let (d_edit, d_arriere, d_avant) = (d.clone(), d.clone(), d.clone());
    let id = d.id;
    let premier = d.statut == "a_faire";
    let dernier = d.statut == "termine";
    view! {
        <div class="rounded-xl border border-slate-100 bg-slate-50/60 p-4">
            <div class="flex items-start justify-between gap-2">
                <div class="min-w-0">
                    <span class=format!("inline-flex items-center px-2 py-0.5 rounded-full text-xs {}", type_badge(&d.type_))>{type_libelle(&d.type_).to_string()}</span>
                    <p class="font-medium text-slate-800 leading-snug mt-1">{d.titre.clone()}</p>
                    <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
                        {(!d.matiere.is_empty()).then(|| view! { <span class="text-xs text-slate-500">{d.matiere.clone()}</span> })}
                        {(!d.echeance.is_empty()).then(|| view! {
                            <span class=format!("inline-flex items-center px-2 py-0.5 rounded-full text-xs border {}", couleur_echeance(&d.echeance))>
                                {libelle_echeance(&d.echeance)}
                            </span>
                        })}
                    </div>
                </div>
                <div class="flex items-center gap-1.5 shrink-0">
                    <button type="button" title="Modifier" class="text-slate-300 hover:text-indigo-500 transition" on:click=move |_| etat.edition.set(Some(d_edit.clone()))>
                        <Icone nom="pencil" class="w-4 h-4" />
                    </button>
                    <button type="button" title="Supprimer" class="text-slate-300 hover:text-rose-500 transition shrink-0" on:click=move |_| etat.supprimer_devoir(id)>
                        <Icone nom="trash" class="w-4 h-4" />
                    </button>
                </div>
            </div>
            <div class="mt-3 flex items-center justify-between gap-2">
                <button type="button" title="Revenir en arrière" prop:disabled=premier
                    class="inline-flex items-center justify-center w-8 h-8 rounded-lg bg-white/60 backdrop-blur border border-white/50 text-slate-500 hover:text-indigo-600 hover:border-indigo-200 disabled:opacity-30 disabled:cursor-not-allowed transition"
                    on:click=move |_| etat.deplacer(&d_arriere, -1)>
                    <Icone nom="chevron-left" class="w-4 h-4" />
                </button>
                <button type="button" title="Faire avancer" prop:disabled=dernier
                    class="inline-flex items-center justify-center w-8 h-8 rounded-lg bg-white/60 backdrop-blur border border-white/50 text-slate-500 hover:text-indigo-600 hover:border-indigo-200 disabled:opacity-30 disabled:cursor-not-allowed transition"
                    on:click=move |_| etat.deplacer(&d_avant, 1)>
                    <Icone nom="chevron-right" class="w-4 h-4" />
                </button>
            </div>
        </div>
    }
}
