//! Onglet Dashboard : statistiques, emploi du temps du jour, devoirs rapides, plan de révision IA.

use crate::icons::{Icone, Spinner};
use crate::state::{
    statut_badge, statut_libelle, statut_point, type_badge, type_libelle, Etat, Onglet,
};
use crate::ui::*;
use crate::util;
use leptos::prelude::*;
use leptos::task::spawn_local;
use tdb_shared::DevoirEntree;

#[component]
pub fn Dashboard() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let date_jour = util::date_longue(&js_sys::Date::new_0());
    let nouveau = RwSignal::new(String::new());
    let plan_copie = RwSignal::new(false);

    let nb_statut =
        move |s: &'static str| etat.devoirs.get().iter().filter(|d| d.statut == s).count();

    let ajouter_rapide = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let titre = nouveau.get_untracked().trim().to_string();
        if titre.is_empty() {
            return;
        }
        etat.ajouter_devoir(
            DevoirEntree {
                titre,
                matiere: String::new(),
                echeance: String::new(),
                type_: "devoir".into(),
                statut: "a_faire".into(),
                tags: Vec::new(),
                todos: Vec::new(),
            },
            move || nouveau.set(String::new()),
        );
    };

    let copier_plan = move |_| {
        let Some(p) = etat.plan.get_untracked() else {
            return;
        };
        let lignes = p
            .planning
            .iter()
            .map(|s| format!("- {} · {} : {}", s.heure, s.matiere, s.action))
            .collect::<Vec<_>>()
            .join("\n");
        let texte = format!("Plan de révision du soir\n\n{}\n\n{lignes}", p.conseil);
        spawn_local(async move {
            if util::copier(&texte).await {
                plan_copie.set(true);
                gloo_timers::callback::Timeout::new(2000, move || plan_copie.set(false)).forget();
            }
        });
    };

    view! {
        <section class="max-w-7xl mx-auto">
            <header class="flex flex-wrap items-center justify-between gap-4">
                <div>
                    <h2 class="text-2xl font-bold text-slate-900">"Dashboard"</h2>
                    <p class="text-sm text-slate-500">{date_jour.clone()}</p>
                </div>
                <span class="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-white/60 backdrop-blur-xl border border-white/40 shadow-sm text-sm text-slate-600">
                    <span class="relative flex h-2 w-2">
                        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                        <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                    </span>
                    {move || etat.heure.get()}
                </span>
            </header>

            <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mt-8">
                <div class=format!("{CARTE} p-5")>
                    <p class="text-xs font-medium text-slate-400 uppercase tracking-wide">"Cours aujourd'hui"</p>
                    <p class="text-2xl font-bold text-slate-900 mt-1">{move || etat.cours.get().len()}</p>
                </div>
                <div class=format!("{CARTE} p-5")>
                    <p class="text-xs font-medium text-slate-400 uppercase tracking-wide">"À faire"</p>
                    <p class="text-2xl font-bold text-rose-600 mt-1">{move || nb_statut("a_faire")}</p>
                </div>
                <div class=format!("{CARTE} p-5")>
                    <p class="text-xs font-medium text-slate-400 uppercase tracking-wide">"En cours"</p>
                    <p class="text-2xl font-bold text-amber-500 mt-1">{move || nb_statut("en_cours")}</p>
                </div>
                <div class=format!("{CARTE} p-5")>
                    <p class="text-xs font-medium text-slate-400 uppercase tracking-wide">"Terminés"</p>
                    <p class="text-2xl font-bold text-emerald-600 mt-1">{move || nb_statut("termine")}</p>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-5 gap-6 mt-6">
                // ---- Emploi du temps du jour
                <section class=format!("lg:col-span-3 {CARTE} p-6")>
                    <div class="flex items-center justify-between gap-3 flex-wrap">
                        <div class="flex items-center gap-3">
                            <div class="w-10 h-10 rounded-xl bg-indigo-50 flex items-center justify-center">
                                <Icone nom="clock" class="w-5 h-5 text-indigo-600" />
                            </div>
                            <div>
                                <h3 class="font-semibold text-slate-900">"Emploi du temps du jour"</h3>
                                <p class="text-xs text-slate-400">"Actualisation toutes les 60 s"</p>
                            </div>
                        </div>
                        <button
                            class=BOUTON_PRIMAIRE
                            prop:disabled=move || etat.chargement_cours.get()
                            on:click=move |_| etat.charger_cours(false)
                        >
                            <span class="inline-flex" class:animate-spin=move || etat.chargement_cours.get()>
                                <Icone nom="refresh" class="w-4 h-4" />
                            </span>
                            {move || if etat.chargement_cours.get() { "Synchro…" } else { "Rafraîchir" }}
                        </button>
                    </div>

                    {move || etat.erreur_cours.get().map(|m| view! { <p class=format!("mt-5 {ALERTE_ERREUR}")>{m}</p> })}
                    {move || etat.hors_ligne.get().then(|| view! {
                        <p class=format!("mt-5 {ALERTE_HORS_LIGNE}")>
                            "Mode hors ligne : emploi du temps du dernier chargement réussi"
                            {maj_libelle(etat)}
                            "."
                        </p>
                    })}

                    {move || {
                        let liste = etat.cours.get();
                        if liste.is_empty() {
                            view! {
                                <div class=format!("mt-6 {VIDE} p-10")>
                                    <p class="font-medium text-slate-600">"Aucun cours prévu aujourd'hui"</p>
                                    <p class="text-sm text-slate-400 mt-1">"Profites-en pour avancer sur tes devoirs ou préparer tes révisions."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <ul class="mt-5 space-y-3">
                                    {liste.into_iter().map(|c| {
                                        let (debut, fin) = (c.debut.clone(), c.fin.clone());
                                        let en_cours = {
                                            let (d, f) = (debut.clone(), fin.clone());
                                            move || { let h = etat.heure.get(); !h.is_empty() && h >= d && h < f }
                                        };
                                        let en_cours2 = en_cours.clone();
                                        view! {
                                            <li
                                                class=move || format!(
                                                    "flex items-start gap-4 rounded-xl border p-4 transition {}",
                                                    if en_cours() { "border-indigo-300 bg-indigo-50/70 shadow-sm" } else { "border-slate-100 hover:border-indigo-200 hover:shadow-sm" }
                                                )
                                            >
                                                <div class="w-16 text-center shrink-0">
                                                    <p class="font-bold text-slate-900 tabular-nums">{debut}</p>
                                                    <p class="text-slate-300 text-xs leading-none my-1">"—"</p>
                                                    <p class="text-sm text-slate-500 tabular-nums">{fin}</p>
                                                </div>
                                                <div class="flex-1 min-w-0">
                                                    <div class="flex items-center gap-2 flex-wrap">
                                                        {move || en_cours2().then(|| view! {
                                                            <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-bold text-white bg-emerald-500 animate-pulse">"EN COURS"</span>
                                                        })}
                                                        <h3 class="font-semibold text-slate-800">{c.titre.clone()}</h3>
                                                    </div>
                                                    <div class="mt-1 flex flex-wrap items-center gap-x-4 gap-y-1">
                                                        <p class="text-sm text-slate-500 flex items-center gap-1.5">
                                                            <Icone nom="pin" class="w-3.5 h-3.5 text-slate-400" />
                                                            {if c.salle.is_empty() { "Salle non renseignée".to_string() } else { c.salle.clone() }}
                                                        </p>
                                                        {(!c.enseignant.is_empty()).then(|| view! {
                                                            <p class="text-sm text-slate-500 flex items-center gap-1.5">
                                                                <Icone nom="user" class="w-3.5 h-3.5 text-slate-400" />
                                                                {c.enseignant.clone()}
                                                            </p>
                                                        })}
                                                    </div>
                                                </div>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                    }}
                </section>

                // ---- Devoirs rapides
                <section class=format!("lg:col-span-2 {CARTE} p-6")>
                    <div class="flex items-center justify-between gap-3">
                        <div class="flex items-center gap-3">
                            <div class="w-10 h-10 rounded-xl bg-violet-50 flex items-center justify-center">
                                <Icone nom="clipboard" class="w-5 h-5 text-violet-600" />
                            </div>
                            <div>
                                <h3 class="font-semibold text-slate-900">"Devoirs"</h3>
                                <p class="text-xs text-slate-400">{move || format!("{} au total", etat.devoirs.get().len())}</p>
                            </div>
                        </div>
                        <button class="text-sm font-medium text-indigo-600 hover:text-indigo-500" on:click=move |_| etat.onglet.set(Onglet::Devoirs)>
                            "Voir le Kanban →"
                        </button>
                    </div>

                    <form class="mt-5 flex gap-2" on:submit=ajouter_rapide>
                        <input
                            class=format!("flex-1 {CHAMP}")
                            type="text"
                            required
                            placeholder="Nouveau devoir…"
                            prop:value=move || nouveau.get()
                            on:input=move |ev| nouveau.set(event_target_value(&ev))
                        />
                        <button type="submit" class="inline-flex items-center justify-center w-11 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white shadow-sm transition">
                            <Icone nom="plus" class="w-4 h-4" />
                        </button>
                    </form>

                    {move || etat.erreur_devoirs.get().map(|m| view! { <p class="mt-4 rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-3">{m}</p> })}

                    {move || {
                        let liste = etat.devoirs.get();
                        if liste.is_empty() {
                            view! {
                                <div class=format!("mt-6 {VIDE} p-8")>
                                    <p class="font-medium text-slate-600">"Aucun devoir pour l'instant"</p>
                                    <p class="text-sm text-slate-400 mt-1">"Ajoute ton premier devoir avec le formulaire ci-dessus."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <ul class="mt-5 divide-y divide-slate-100">
                                    {liste.into_iter().take(6).map(|d| {
                                        let d_cycle = d.clone();
                                        let id = d.id;
                                        view! {
                                            <li class="flex items-center gap-3 py-3">
                                                <button
                                                    type="button"
                                                    title=format!("Statut : {}", statut_libelle(&d.statut))
                                                    class=format!("w-3 h-3 rounded-full shrink-0 transition {}", statut_point(&d.statut))
                                                    on:click=move |_| etat.cycle_statut(&d_cycle)
                                                ></button>
                                                <div class="flex-1 min-w-0">
                                                    <p class=format!("truncate font-medium {}", if d.statut == "termine" { "line-through text-slate-400" } else { "text-slate-800" })>{d.titre.clone()}</p>
                                                    <div class="mt-1 flex flex-wrap items-center gap-1.5">
                                                        <span class=format!("inline-flex items-center px-2 py-0.5 rounded-full text-xs {}", type_badge(&d.type_))>{type_libelle(&d.type_).to_string()}</span>
                                                        {(!d.matiere.is_empty()).then(|| view! { <span class="text-xs text-slate-400">{d.matiere.clone()}</span> })}
                                                        {(!d.echeance.is_empty()).then(|| view! {
                                                            <span class="inline-flex items-center text-xs text-slate-400">
                                                                <Icone nom="calendar" class="w-3.5 h-3.5 mr-1" />
                                                                {util::date_tres_courte(&d.echeance)}
                                                            </span>
                                                        })}
                                                        <span class=format!("inline-flex items-center px-2 py-0.5 rounded-full text-xs {}", statut_badge(&d.statut))>{statut_libelle(&d.statut).to_string()}</span>
                                                    </div>
                                                </div>
                                                <button type="button" title="Supprimer" class="text-slate-300 hover:text-rose-500 transition" on:click=move |_| etat.supprimer_devoir(id)>
                                                    <Icone nom="trash" class="w-4 h-4" />
                                                </button>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                    }}
                </section>

                // ---- Plan de révision IA
                <section class="lg:col-span-5 bg-gradient-to-br from-indigo-950 via-slate-900 to-indigo-900 rounded-3xl shadow-xl shadow-indigo-900/30 border border-indigo-500/20 p-6 md:p-8">
                    <div class="flex items-start justify-between gap-4 flex-wrap">
                        <div class="flex items-center gap-4">
                            <div class="w-11 h-11 rounded-xl bg-gradient-to-br from-indigo-500 to-violet-500 flex items-center justify-center shadow-lg shadow-indigo-500/40">
                                <Icone nom="sparkles" class="w-5 h-5 text-white" />
                            </div>
                            <div>
                                <h3 class="text-xl font-bold text-white">"Plan de révision IA"</h3>
                                <p class="text-sm text-indigo-200/70">"Coach personnel basé sur tes cours et tes devoirs, propulsé par Gemini."</p>
                            </div>
                        </div>
                        <button
                            class="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 hover:from-indigo-400 hover:to-violet-500 disabled:opacity-60 text-white text-sm font-semibold shadow-lg shadow-indigo-500/40 transition"
                            prop:disabled=move || etat.chargement_plan.get()
                            on:click=move |_| etat.generer_plan()
                        >
                            {move || if etat.chargement_plan.get() {
                                view! { <Spinner class="w-4 h-4" /> }.into_any()
                            } else {
                                view! { <Icone nom="bolt" class="w-4 h-4" /> }.into_any()
                            }}
                            {move || if etat.chargement_plan.get() {
                                "L'IA analyse ta journée…"
                            } else if etat.plan.get().is_some() {
                                "Régénérer le plan"
                            } else {
                                "Générer mon plan du soir"
                            }}
                        </button>
                    </div>

                    {move || {
                        if etat.chargement_plan.get() {
                            return view! {
                                <div class="mt-8 rounded-xl bg-white/5 border border-white/10 p-10 text-center">
                                    <Spinner class="w-8 h-8 text-indigo-300 mx-auto" />
                                    <p class="mt-4 text-slate-200 font-medium">"L'IA analyse ton emploi du temps et tes devoirs…"</p>
                                    <p class="mt-1 text-sm text-slate-400">"Le plan de la soirée arrive dans quelques secondes."</p>
                                </div>
                            }.into_any();
                        }
                        if let Some(m) = etat.erreur_plan.get() {
                            return view! { <div class="mt-8 rounded-xl bg-rose-500/10 border border-rose-400/30 p-4 text-rose-200 text-sm">{m}</div> }.into_any();
                        }
                        match etat.plan.get() {
                            None => view! {
                                <div class="mt-8 rounded-xl bg-white/5 border border-dashed border-white/15 p-10 text-center">
                                    <p class="text-slate-300 font-medium">"Aucun plan généré pour l'instant"</p>
                                    <p class="text-sm text-slate-400 mt-1">"L'IA analysera ton emploi du temps et tes devoirs pour proposer une soirée de révision optimale."</p>
                                </div>
                            }.into_any(),
                            Some(p) => {
                                let simulation_sans_alerte = p.mode == "simulation" && p.avertissement.is_none();
                                let avertissement = p.avertissement.clone();
                                view! {
                                    <div class="mt-8 space-y-6">
                                        <div class="rounded-xl bg-white/5 border border-white/10 p-5">
                                            <p class="text-xs font-semibold uppercase tracking-wider text-indigo-300 mb-2">"Conseil du soir"</p>
                                            <p class="text-slate-100 leading-relaxed">{p.conseil.clone()}</p>
                                            {avertissement.map(|a| view! { <p class="mt-3 text-xs text-amber-300">{a}</p> })}
                                            {simulation_sans_alerte.then(|| view! {
                                                <p class="mt-3 text-xs text-indigo-200/70">"Mode simulation : ajoute une clé Gemini dans Configuration pour un plan personnalisé par l'IA."</p>
                                            })}
                                        </div>
                                        {if p.planning.is_empty() {
                                            view! { <p class="text-sm text-slate-400">"L'IA n'a pas proposé de créneau structuré cette fois-ci."</p> }.into_any()
                                        } else {
                                            view! {
                                                <div class="space-y-3">
                                                    {p.planning.iter().enumerate().map(|(i, s)| view! {
                                                        <div class="rounded-xl bg-white/5 border border-white/10 p-5 flex flex-wrap items-center gap-4">
                                                            <span class="w-10 h-10 rounded-lg bg-indigo-500/20 text-indigo-200 font-semibold flex items-center justify-center text-sm tabular-nums">{i + 1}</span>
                                                            <div class="min-w-0 flex-1">
                                                                <h3 class="text-slate-50 font-semibold">{s.matiere.clone()}</h3>
                                                                <p class="text-sm text-slate-300 mt-0.5">{s.action.clone()}</p>
                                                            </div>
                                                            <span class="inline-flex items-center px-3 py-1.5 rounded-full bg-white/10 text-indigo-100 text-sm font-medium tabular-nums shrink-0">{s.heure.clone()}</span>
                                                        </div>
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }}
                                        <div class="flex items-center justify-between gap-3 flex-wrap">
                                            <p class="text-sm text-indigo-200/70">"Plan adapté à ton emploi du temps du jour"</p>
                                            <button class="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-white text-sm font-medium transition" on:click=copier_plan>
                                                <Icone nom="copy" class="w-4 h-4" />
                                                {move || if plan_copie.get() { "Copié !" } else { "Copier le plan" }}
                                            </button>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </section>
            </div>
        </section>
    }
}

/// « (21 octobre, 14:05) » quand la date de dernière synchronisation est connue.
pub fn maj_libelle(etat: Etat) -> String {
    let m = util::date_heure(&etat.maj_flux.get());
    if m.is_empty() {
        String::new()
    } else {
        format!(" ({m})")
    }
}
