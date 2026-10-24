//! Onglet Devoirs : formulaire d'ajout, tableau Kanban, import / export JSON.

use crate::api;
use crate::icons::Icone;
use crate::state::{
    couleur_echeance, libelle_echeance, type_badge, type_libelle, Etat, COLONNES, TYPES_DEVOIR,
};
use crate::ui::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use tdb_shared::{Devoir, DevoirEntree, Tag, TagEntree};

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
                tags: Vec::new(),
                todos: Vec::new(),
                description: String::new(),
                priorite: 0,
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
                    <button type="button" class=BOUTON_PILULE title="Créer, modifier ou supprimer les tags" on:click=move |_| etat.gestion_tags.set(true)>
                        <Icone nom="tag" class="w-4 h-4" />
                        "Gérer les tags"
                    </button>
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

            {move || {
                let liste = etat.tags.get();
                (!liste.is_empty()).then(|| {
                    let actif = etat.filtre_tag.get();
                    view! {
                        <div class="mt-4 flex flex-wrap items-center gap-2">
                            <span class="text-sm font-medium text-slate-500 mr-1">"Filtrer :"</span>
                            <button type="button" class=move || format!("px-3 py-1.5 rounded-full text-xs font-medium border transition {}", if actif.is_none() { "bg-indigo-600 text-white border-indigo-600" } else { "bg-white/60 text-slate-600 border-white/50 hover:border-indigo-300" })
                                on:click=move |_| etat.filtre_tag.set(None)>"Tous"</button>
                            {liste.into_iter().map(|t| {
                                let id = t.id;
                                let couleur = t.couleur.clone();
                                view! {
                                    <button type="button" class=move || format!("px-3 py-1.5 rounded-full text-xs font-medium border transition {}", if etat.filtre_tag.get() == Some(id) { "text-white border-transparent" } else { "bg-white/60 text-slate-600 border-white/50 hover:border-indigo-300" })
                                        style=move || if etat.filtre_tag.get() == Some(id) { format!("background-color:{couleur}") } else { String::new() }
                                        on:click=move |_| etat.filtre_tag.set(Some(id))>{t.nom}</button>
                                }
                            }).collect_view()}
                        </div>
                    }
                })
            }}
            {move || etat.erreur_devoirs.get().map(|m| view! { <p class=format!("mt-4 {ALERTE_ERREUR}")>{m}</p> })}

            <div class="mt-6 grid grid-cols-1 md:grid-cols-3 gap-5">
                {COLONNES.iter().map(|(id, libelle, badge)| {
                    let id = *id;
                    let dans_colonne = move || {
                        let filtre = etat.filtre_tag.get();
                        etat.devoirs.get().into_iter()
                            .filter(|d| d.statut == id && filtre.is_none_or(|t| d.tags.iter().any(|g| g.id == t)))
                            .collect::<Vec<Devoir>>()
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
            {move || etat.gestion_tags.get().then(|| view! { <GestionTags /> })}
        </section>
    }
}

/// Fenêtre de gestion des tags : création, renommage, changement de couleur et suppression.
#[component]
fn GestionTags() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let fermer = move || etat.gestion_tags.set(false);
    let nouveau_nom = RwSignal::new(String::new());
    let nouvelle_couleur = RwSignal::new("#6366f1".to_string());
    let erreur = RwSignal::new(String::new());

    let creer = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let entree = TagEntree {
            nom: nouveau_nom.get_untracked(),
            couleur: nouvelle_couleur.get_untracked(),
        };
        spawn_local(async move {
            match api::creer_tag(&entree).await {
                Ok(_) => {
                    nouveau_nom.set(String::new());
                    erreur.set(String::new());
                    etat.charger_devoirs_async().await;
                }
                Err(e) => erreur.set(etat.message(e)),
            }
        });
    };

    view! {
        <div class="fixed inset-0 z-[60] flex items-center justify-center p-4">
            <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-sm" on:click=move |_| fermer()></div>
            <div class="relative w-full max-w-lg max-h-[90vh] overflow-y-auto bg-white/90 backdrop-blur-2xl rounded-3xl border border-white/50 shadow-2xl p-6 sm:p-8">
                <h3 class="text-lg font-bold text-slate-900">"Gérer les tags"</h3>
                <form class="mt-5 flex flex-wrap items-center gap-2" on:submit=creer>
                    <input class=format!("{CHAMP} flex-1 min-w-0") type="text" required maxlength="30" placeholder="Nouveau tag (ex. CM)"
                        prop:value=move || nouveau_nom.get() on:input=move |ev| nouveau_nom.set(event_target_value(&ev)) />
                    <input class="h-10 w-12 rounded-lg border border-white/50 bg-white/60 cursor-pointer" type="color"
                        prop:value=move || nouvelle_couleur.get() on:input=move |ev| nouvelle_couleur.set(event_target_value(&ev)) />
                    <button type="submit" class=BOUTON_PRIMAIRE>
                        <Icone nom="plus" class="w-4 h-4" />
                        "Ajouter"
                    </button>
                </form>
                {move || { let m = erreur.get(); (!m.is_empty()).then(|| view! { <p class=format!("mt-4 {ALERTE_ERREUR}")>{m}</p> }) }}
                <ul class="mt-5 space-y-2">
                    {move || {
                        let liste = etat.tags.get();
                        if liste.is_empty() {
                            return view! { <li class=format!("{VIDE} p-4 text-xs text-slate-400")>"Aucun tag pour l'instant."</li> }.into_any();
                        }
                        liste.into_iter().map(|t| ligne_tag(etat, t, erreur)).collect_view().into_any()
                    }}
                </ul>
                <div class="mt-6 text-right">
                    <button type="button" class=BOUTON_PILULE on:click=move |_| fermer()>"Fermer"</button>
                </div>
            </div>
        </div>
    }
}

/// Une ligne de la fenêtre de gestion : renommer, changer la couleur ou supprimer un tag.
fn ligne_tag(etat: Etat, t: Tag, erreur: RwSignal<String>) -> impl IntoView {
    let id = t.id;
    let nom = RwSignal::new(t.nom);
    let couleur = RwSignal::new(t.couleur);
    let enregistrer = move |_| {
        let entree = TagEntree {
            nom: nom.get_untracked(),
            couleur: couleur.get_untracked(),
        };
        spawn_local(async move {
            match api::modifier_tag(id, &entree).await {
                Ok(_) => {
                    erreur.set(String::new());
                    etat.charger_devoirs_async().await;
                }
                Err(e) => erreur.set(etat.message(e)),
            }
        });
    };
    let supprimer = move |_| {
        spawn_local(async move {
            match api::supprimer_tag(id).await {
                Ok(()) => {
                    if etat.filtre_tag.get_untracked() == Some(id) {
                        etat.filtre_tag.set(None);
                    }
                    erreur.set(String::new());
                    etat.charger_devoirs_async().await;
                }
                Err(e) => erreur.set(etat.message(e)),
            }
        });
    };
    view! {
        <li class="flex flex-wrap items-center gap-2 rounded-xl bg-slate-50/60 border border-slate-100 p-2">
            <input class="h-9 w-12 rounded-lg border border-white/50 bg-white/60 cursor-pointer" type="color"
                prop:value=move || couleur.get() on:input=move |ev| couleur.set(event_target_value(&ev)) />
            <input class=format!("{CHAMP} flex-1 min-w-0") type="text" required maxlength="30"
                prop:value=move || nom.get() on:input=move |ev| nom.set(event_target_value(&ev)) />
            <button type="button" class=BOUTON_PILULE on:click=enregistrer>"Enregistrer"</button>
            <button type="button" title="Supprimer le tag (les devoirs sont conservés)" class="text-slate-300 hover:text-rose-500 transition" on:click=supprimer>
                <Icone nom="trash" class="w-4 h-4" />
            </button>
        </li>
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
            {(!d.todos.is_empty()).then(|| {
                let total = d.todos.len();
                let faits = d.todos.iter().filter(|t| t.fait).count();
                let pourcent = faits * 100 / total;
                let complet = faits == total;
                view! {
                    <div class="mt-2.5" title="Sous-tâches terminées">
                        <div class="flex items-center justify-between text-xs text-slate-500">
                            <span class="inline-flex items-center gap-1">
                                <Icone nom="check" class="w-3.5 h-3.5" />
                                "Sous-tâches"
                            </span>
                            <span class=format!("font-medium tabular-nums {}", if complet { "text-emerald-600" } else { "" })>{format!("{faits}/{total}")}</span>
                        </div>
                        <div class="mt-1 h-1.5 rounded-full bg-slate-200/70 overflow-hidden">
                            <div class=format!("h-full rounded-full transition-all {}", if complet { "bg-emerald-500" } else { "bg-indigo-500" }) style=format!("width:{pourcent}%")></div>
                        </div>
                    </div>
                }
            })}
            {(!d.tags.is_empty()).then(|| view! {
                <div class="mt-2 flex flex-wrap gap-1.5">
                    {d.tags.iter().map(|t| view! {
                        <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium text-white" style=format!("background-color:{}", t.couleur)>{t.nom.clone()}</span>
                    }).collect_view()}
                </div>
            })}
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
