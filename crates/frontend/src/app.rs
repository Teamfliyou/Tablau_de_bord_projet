//! Coquille de l'application : navigation (dock ou barre latérale), onglets, fenêtres modales, notifications.

use crate::api::{self, Mode};
use crate::icons::Icone;
use crate::pages::auth::Authentification;
use crate::pages::config::Configuration;
use crate::pages::dashboard::Dashboard;
use crate::pages::kanban::Kanban;
use crate::pages::semaine::Semaine;
use crate::state::{Etat, Onglet, StyleNav, COLONNES, TYPES_DEVOIR};
use crate::ui::*;
use crate::util;
use gloo_timers::callback::Interval;
use leptos::prelude::*;
use leptos::task::spawn_local;
use tdb_shared::{ConfigUpdate, Devoir, DevoirModification, TodoEntree, TODOS_PAR_DEVOIR_MAX};

#[component]
pub fn App() -> impl IntoView {
    let etat = Etat::nouveau(api::mode());
    provide_context(etat);

    // Une session mémorisée n'est acceptée qu'une fois validée par le serveur.
    if etat.mode == Mode::Web && etat.authentifie.get_untracked() {
        spawn_local(async move {
            if !api::profil_valide().await {
                api::oublier_jeton();
                etat.authentifie.set(false);
            }
        });
    }

    // Chargement des données dès que l'utilisateur est authentifié (immédiat dans l'application Windows).
    let demarre = RwSignal::new(false);
    Effect::new(move |_| {
        if etat.authentifie.get() && !demarre.get_untracked() {
            demarre.set(true);
            etat.demarrer();
        } else if !etat.authentifie.get() {
            demarre.set(false);
        }
    });

    // Actualisation automatique toutes les 60 secondes ; horloge toutes les 30 secondes.
    Interval::new(60_000, move || etat.charger_cours(true)).forget();
    Interval::new(30_000, move || etat.heure.set(util::heure_actuelle())).forget();
    util::surveiller_theme_systeme(move || etat.appliquer_theme());
    etat.appliquer_theme();

    // L'application Windows utilise la nouvelle interface ; la version Web garde l'interface historique.
    let windows = etat.mode == Mode::Tauri;
    util::definir_classe_racine("win", windows);

    view! {
        {move || if etat.authentifie.get() {
            if windows {
                view! { <crate::win::shell::ApplicationWin /> }.into_any()
            } else {
                view! { <Application /> }.into_any()
            }
        } else {
            view! { <Authentification /> }.into_any()
        }}
    }
}

#[component]
fn Application() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let classic = move || etat.style_nav.get() == StyleNav::Classic;
    let identifiant = move || {
        let u = etat.config.get().username;
        if u.is_empty() {
            "Étudiant".to_string()
        } else {
            u
        }
    };
    let date_jour = util::date_longue(&js_sys::Date::new_0());
    let pied = if etat.mode == Mode::Web {
        "Emploi du temps ADE · Devoirs · Génération IA à la demande · Application Web"
    } else {
        "Emploi du temps ADE · Devoirs · Génération IA à la demande · Application locale hors ligne"
    };

    view! {
        <div id="app" class="min-h-screen">
            // ---- Sidebar classique
            {move || classic().then(|| view! {
                <aside class="hidden lg:flex fixed inset-y-0 left-0 w-64 flex-col bg-white/60 backdrop-blur-2xl border-r border-white/40 px-4 py-8">
                    <div class="flex items-center gap-3 px-2">
                        <div class="w-11 h-11 rounded-2xl bg-gradient-to-br from-indigo-600 to-violet-600 flex items-center justify-center shadow-lg shadow-indigo-500/30">
                            <Icone nom="mortarboard" class="w-6 h-6 text-white" />
                        </div>
                        <div>
                            <h1 class="text-lg font-bold text-slate-900 leading-tight">"Tableau de bord"</h1>
                            <p class="text-xs text-slate-500">{identifiant}</p>
                        </div>
                    </div>
                    <nav class="mt-10 space-y-1.5 flex-1">
                        {Onglet::TOUS.iter().map(|o| {
                            let o = *o;
                            view! {
                                <button
                                    class=move || format!("w-full flex items-center gap-3 px-3.5 py-3 rounded-xl text-sm font-medium transition {}",
                                        if etat.onglet.get() == o { "bg-gradient-to-r from-indigo-500 to-violet-500 text-white shadow-md shadow-indigo-500/30" } else { "text-slate-500 hover:bg-white/60 hover:text-slate-800" })
                                    on:click=move |_| etat.onglet.set(o)>
                                    <Icone nom=o.icone() class="w-5 h-5 shrink-0" />
                                    {o.libelle()}
                                </button>
                            }
                        }).collect_view()}
                    </nav>
                    <div class="pt-4 border-t border-white/40 px-2">
                        <p class="text-xs text-slate-400">{date_jour.clone()}</p>
                    </div>
                    <button type="button" class="mt-4 w-full flex items-center gap-3 px-3.5 py-3 rounded-xl text-sm font-medium text-slate-500 hover:bg-white/60 hover:text-slate-800 transition"
                        on:click=move |_| etat.basculer_theme()>
                        <Icone nom="moon" class="w-5 h-5 shrink-0" />
                        {move || if etat.theme_sombre() { "Mode clair" } else { "Mode sombre" }}
                    </button>
                </aside>
                <div class="lg:hidden sticky top-0 z-30 bg-white/60 backdrop-blur-xl border-b border-white/40 px-3 py-2 flex gap-1.5 overflow-x-auto">
                    {Onglet::TOUS.iter().map(|o| {
                        let o = *o;
                        view! {
                            <button
                                class=move || format!("shrink-0 inline-flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm font-medium transition {}",
                                    if etat.onglet.get() == o { "bg-gradient-to-r from-indigo-500 to-violet-500 text-white shadow-sm" } else { "bg-white/60 text-slate-600" })
                                on:click=move |_| etat.onglet.set(o)>
                                <Icone nom=o.icone() class="w-5 h-5 shrink-0" />
                                <span>{o.libelle()}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>
            })}

            // ---- Dock de navigation (par défaut)
            {move || (!classic()).then(|| view! {
                <nav class="fixed bottom-5 left-1/2 -translate-x-1/2 z-50 flex items-center gap-0.5 sm:gap-1 p-1 sm:p-1.5 rounded-full bg-white/30 backdrop-blur-xl border border-white/20 shadow-2xl shadow-indigo-900/10">
                    {Onglet::TOUS.iter().map(|o| {
                        let o = *o;
                        view! {
                            <button
                                title=o.libelle()
                                class=move || format!("relative flex flex-col items-center justify-center gap-1 px-2 sm:px-4 py-2 sm:py-2.5 rounded-full transition-all duration-300 {}",
                                    if etat.onglet.get() == o { "text-white" } else { "text-slate-500 hover:text-slate-800" })
                                on:click=move |_| etat.onglet.set(o)>
                                {move || (etat.onglet.get() == o).then(|| view! {
                                    <span class="absolute inset-0 rounded-full bg-gradient-to-br from-indigo-500 via-violet-500 to-fuchsia-500 dock-active" aria-hidden="true"></span>
                                })}
                                <Icone nom=o.icone() class="relative w-6 h-6 shrink-0" />
                                <span class="relative text-[9px] sm:text-[10px] font-semibold leading-none whitespace-nowrap">{o.libelle()}</span>
                            </button>
                        }
                    }).collect_view()}
                    <span class="w-px h-8 bg-white/40 mx-1 sm:mx-2 hidden sm:block" aria-hidden="true"></span>
                    <button type="button"
                        title=move || if etat.theme_sombre() { "Passer en mode clair" } else { "Passer en mode sombre" }
                        class="relative flex flex-col items-center justify-center gap-1 px-2 sm:px-4 py-2 sm:py-2.5 rounded-full transition-all duration-300 text-slate-500 hover:text-slate-800"
                        on:click=move |_| etat.basculer_theme()>
                        <Icone nom="moon" class="relative w-6 h-6 shrink-0" />
                        <span class="relative text-[9px] sm:text-[10px] font-semibold leading-none whitespace-nowrap">{move || if etat.theme_sombre() { "Clair" } else { "Sombre" }}</span>
                    </button>
                </nav>
            })}

            // ---- Contenu
            <main class=move || format!("flex-1 px-4 sm:px-6 lg:px-10 pt-10 sm:pt-14 {}", if classic() { "lg:ml-64 pb-16" } else { "pb-28" })>
                {move || match etat.onglet.get() {
                    Onglet::Dashboard => view! { <Dashboard /> }.into_any(),
                    Onglet::Semaine => view! { <Semaine /> }.into_any(),
                    Onglet::Devoirs => view! { <Kanban /> }.into_any(),
                    Onglet::Config => view! { <Configuration /> }.into_any(),
                }}
                <footer class="max-w-7xl mx-auto mt-14">
                    <p class="text-xs text-slate-400 text-center">{pied}</p>
                </footer>
            </main>

            {move || etat.onboarding_visible.get().then(|| view! { <Onboarding /> })}
            {move || etat.maj_fenetre.get().then(|| view! { <crate::maj::FenetreMiseAJour /> })}
            {move || etat.edition.get().map(|d| view! { <EditionDevoir devoir=d /> })}
            {move || { let t = etat.toast.get(); (!t.is_empty()).then(|| view! {
                <div class="fixed top-5 right-5 z-[70] max-w-sm rounded-2xl bg-slate-900/90 text-white text-sm px-4 py-3 shadow-xl backdrop-blur">{t}</div>
            }) }}
        </div>
    }
}

/// Première configuration : lien ADE (obligatoire) et clé Gemini (facultative).
#[component]
pub fn Onboarding() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let ade = RwSignal::new(String::new());
    let cle = RwSignal::new(String::new());
    let erreur = RwSignal::new(String::new());
    let en_cours = RwSignal::new(false);

    let ignorer = move || {
        etat.onboarding_ignore.set(true);
        etat.onboarding_visible.set(false);
    };

    let valider = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            en_cours.set(true);
            erreur.set(String::new());
            let url = ade.get_untracked().trim().to_string();
            if url.is_empty() {
                erreur.set("Renseigne ton lien ADE pour continuer.".into());
                en_cours.set(false);
                return;
            }
            let c = etat.config.get_untracked();
            let maj = ConfigUpdate {
                username: c.username,
                ade_url: url,
                gemini_key: Some(cle.get_untracked().trim().to_string()).filter(|k| !k.is_empty()),
                gemini_model: Some(c.gemini_model).filter(|m| !m.is_empty()),
                remove_gemini_key: false,
            };
            match etat.enregistrer_config(maj).await {
                Ok(()) => etat.onboarding_visible.set(false),
                Err(e) => erreur.set(e),
            }
            en_cours.set(false);
        });
    };

    view! {
        <div class="fixed inset-0 z-[60] flex items-center justify-center p-4">
            <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-sm" on:click=move |_| ignorer()></div>
            <div class="relative w-full max-w-lg bg-white/90 backdrop-blur-2xl rounded-3xl border border-white/50 shadow-2xl p-6 sm:p-8">
                <div class="flex items-center gap-3">
                    <div class="w-11 h-11 rounded-2xl bg-gradient-to-br from-indigo-600 to-violet-600 flex items-center justify-center shadow-lg shadow-indigo-500/30 shrink-0">
                        <Icone nom="bolt" class="w-6 h-6 text-white" />
                    </div>
                    <div>
                        <h3 class="text-lg font-bold text-slate-900">
                            {move || { let u = etat.config.get().username; format!("Bienvenue, {} !", if u.is_empty() { "étudiant".to_string() } else { u }) }}
                        </h3>
                        <p class="text-sm text-slate-500">"Terminons la configuration en 30 secondes."</p>
                    </div>
                </div>
                <form class="mt-6 space-y-4" on:submit=valider>
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1.5">"Lien d'export iCal / ADE " <span class="text-rose-500">"*"</span></label>
                        <input class=CHAMP_MONO type="url" required placeholder="https://ton-ecole.example.com/ade/ics?data=…"
                            prop:value=move || ade.get() on:input=move |ev| ade.set(event_target_value(&ev)) />
                        <p class="text-xs text-slate-400 mt-1.5">"Indispensable pour afficher ton emploi du temps. Trouvé dans ton application ADE : Emploi du temps → Export / iCal."</p>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1.5">"Clé API Gemini " <span class="text-slate-400">"(optionnel)"</span></label>
                        <input class=CHAMP_MONO type="password" autocomplete="off" placeholder="AIza…"
                            prop:value=move || cle.get() on:input=move |ev| cle.set(event_target_value(&ev)) />
                        <p class="text-xs text-slate-400 mt-1.5">"Gratuite sur " <span class="font-medium">"https://aistudio.google.com/apikey"</span> ". Sans clé, le plan de révision fonctionne en mode simulation."</p>
                    </div>
                    {move || { let m = erreur.get(); (!m.is_empty()).then(|| view! { <p class="rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-3.5">{m}</p> }) }}
                    <div class="flex flex-wrap items-center gap-3">
                        <button type="submit" class="flex-1 inline-flex items-center justify-center gap-2 px-5 py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 text-white text-sm font-semibold shadow-sm transition" prop:disabled=move || en_cours.get()>
                            {move || if en_cours.get() { "Enregistrement…" } else { "Commencer" }}
                        </button>
                        <button type="button" class="px-4 py-3 rounded-xl text-sm font-medium text-slate-500 hover:text-slate-700 transition" on:click=move |_| ignorer()>"Plus tard"</button>
                    </div>
                </form>
            </div>
        </div>
    }
}

/// Fenêtre de modification d'un devoir.
#[component]
fn EditionDevoir(devoir: Devoir) -> impl IntoView {
    let etat = expect_context::<Etat>();
    let id = devoir.id;
    let titre = RwSignal::new(devoir.titre);
    let matiere = RwSignal::new(devoir.matiere);
    let echeance = RwSignal::new(devoir.echeance);
    let type_ = RwSignal::new(devoir.type_);
    let statut = RwSignal::new(devoir.statut);
    let erreur = RwSignal::new(String::new());
    let en_cours = RwSignal::new(false);
    let fermer = move || etat.edition.set(None);

    // Tags du devoir : chaque clic enregistre immédiatement l'ensemble choisi.
    let tags_choisis = RwSignal::new(devoir.tags.iter().map(|t| t.id).collect::<Vec<i64>>());
    let basculer = move |tag_id: i64| {
        let mut ids = tags_choisis.get_untracked();
        match ids.iter().position(|x| *x == tag_id) {
            Some(i) => {
                ids.remove(i);
            }
            None => ids.push(tag_id),
        }
        spawn_local(async move {
            match api::definir_tags_devoir(id, &ids).await {
                Ok(_) => {
                    tags_choisis.set(ids);
                    etat.charger_devoirs_async().await;
                }
                Err(e) => erreur.set(etat.message(e)),
            }
        });
    };

    // To-do list : chaque action enregistre immédiatement la liste complète (ordre et états).
    let todos = RwSignal::new(
        devoir
            .todos
            .iter()
            .map(|t| (t.texte.clone(), t.fait))
            .collect::<Vec<(String, bool)>>(),
    );
    let nouveau_todo = RwSignal::new(String::new());
    let sauver_todos = move |liste: Vec<(String, bool)>| {
        let entrees = liste
            .iter()
            .map(|(texte, fait)| TodoEntree {
                texte: texte.clone(),
                fait: *fait,
            })
            .collect::<Vec<_>>();
        spawn_local(async move {
            match api::definir_todos_devoir(id, &entrees).await {
                Ok(d) => {
                    erreur.set(String::new());
                    todos.set(d.todos.into_iter().map(|t| (t.texte, t.fait)).collect());
                    etat.charger_devoirs_async().await;
                }
                Err(e) => erreur.set(etat.message(e)),
            }
        });
    };
    let ajouter_todo = move || {
        let texte = nouveau_todo.get_untracked().trim().to_string();
        if texte.is_empty() {
            return;
        }
        let mut liste = todos.get_untracked();
        if liste.len() >= TODOS_PAR_DEVOIR_MAX {
            erreur.set("Un devoir ne peut pas avoir plus de 100 sous-tâches.".into());
            return;
        }
        liste.push((texte, false));
        nouveau_todo.set(String::new());
        sauver_todos(liste);
    };
    let modifier_todo = move |i: usize, texte: String| {
        let texte = texte.trim().to_string();
        let mut liste = todos.get_untracked();
        match liste.get(i) {
            Some((ancien, _)) if *ancien == texte => {}
            Some(_) if texte.is_empty() => {
                liste.remove(i);
                sauver_todos(liste);
            }
            Some(_) => {
                liste[i].0 = texte;
                sauver_todos(liste);
            }
            None => {}
        }
    };
    let cocher_todo = move |i: usize| {
        let mut liste = todos.get_untracked();
        if let Some(t) = liste.get_mut(i) {
            t.1 = !t.1;
            sauver_todos(liste);
        }
    };
    let supprimer_todo = move |i: usize| {
        let mut liste = todos.get_untracked();
        if i < liste.len() {
            liste.remove(i);
            sauver_todos(liste);
        }
    };
    let deplacer_todo = move |i: usize, sens: i32| {
        let mut liste = todos.get_untracked();
        let j = i as i32 + sens;
        if j >= 0 && (j as usize) < liste.len() {
            liste.swap(i, j as usize);
            sauver_todos(liste);
        }
    };

    let enregistrer = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            en_cours.set(true);
            erreur.set(String::new());
            let m = DevoirModification {
                titre: Some(titre.get_untracked()),
                matiere: Some(matiere.get_untracked()),
                echeance: Some(echeance.get_untracked()),
                type_: Some(type_.get_untracked()),
                statut: Some(statut.get_untracked()),
                ..Default::default()
            };
            match api::modifier_devoir(id, &m).await {
                Ok(_) => {
                    etat.edition.set(None);
                    etat.charger_devoirs_async().await;
                }
                Err(e) => erreur.set(etat.message(e)),
            }
            en_cours.set(false);
        });
    };

    let champ = "w-full rounded-xl border border-white/50 bg-white/60 px-3.5 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500";
    view! {
        <div class="fixed inset-0 z-[60] flex items-center justify-center p-4">
            <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-sm" on:click=move |_| fermer()></div>
            <div class="relative w-full max-w-lg max-h-[90vh] overflow-y-auto bg-white/90 backdrop-blur-2xl rounded-3xl border border-white/50 shadow-2xl p-6 sm:p-8">
                <h3 class="text-lg font-bold text-slate-900">"Modifier le devoir"</h3>
                <form class="mt-5 space-y-4" on:submit=enregistrer>
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1.5">"Titre"</label>
                        <input class=champ type="text" required maxlength="200" prop:value=move || titre.get() on:input=move |ev| titre.set(event_target_value(&ev)) />
                    </div>
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-slate-700 mb-1.5">"Matière"</label>
                            <input class=champ type="text" maxlength="120" prop:value=move || matiere.get() on:input=move |ev| matiere.set(event_target_value(&ev)) />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-700 mb-1.5">"Échéance"</label>
                            <input class=champ type="date" prop:value=move || echeance.get() on:input=move |ev| echeance.set(event_target_value(&ev)) />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-700 mb-1.5">"Type"</label>
                            <select class=champ prop:value=move || type_.get() on:change=move |ev| type_.set(event_target_value(&ev))>
                                {TYPES_DEVOIR.iter().map(|(id, l)| view! { <option value=*id>{*l}</option> }).collect_view()}
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-700 mb-1.5">"Statut"</label>
                            <select class=champ prop:value=move || statut.get() on:change=move |ev| statut.set(event_target_value(&ev))>
                                {COLONNES.iter().map(|(id, l, _)| view! { <option value=*id>{*l}</option> }).collect_view()}
                            </select>
                        </div>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1.5">"Tags"</label>
                        <div class="flex flex-wrap gap-2">
                            {move || {
                                let liste = etat.tags.get();
                                if liste.is_empty() {
                                    return view! { <p class="text-xs text-slate-400">"Aucun tag : crée-les avec « Gérer les tags » dans le Kanban."</p> }.into_any();
                                }
                                liste.into_iter().map(|t| {
                                    let tid = t.id;
                                    let couleur = t.couleur.clone();
                                    view! {
                                        <button type="button"
                                            class=move || format!("inline-flex items-center px-3 py-1 rounded-full text-xs font-medium border transition {}", if tags_choisis.get().contains(&tid) { "text-white border-transparent" } else { "bg-white/60 text-slate-600 border-slate-200" })
                                            style=move || if tags_choisis.get().contains(&tid) { format!("background-color:{couleur}") } else { String::new() }
                                            on:click=move |_| basculer(tid)>
                                            {t.nom.clone()}
                                        </button>
                                    }
                                }).collect_view().into_any()
                            }}
                        </div>
                    </div>
                    <div>
                        <div class="flex items-center justify-between mb-1.5">
                            <label class="block text-sm font-medium text-slate-700">"Sous-tâches"</label>
                            <span class="text-xs text-slate-500 tabular-nums">{move || { let l = todos.get(); format!("{}/{}", l.iter().filter(|t| t.1).count(), l.len()) }}</span>
                        </div>
                        <ul class="space-y-1.5">
                            {move || todos.get().into_iter().enumerate().map(|(i, (texte, fait))| {
                                let dernier = i + 1 == todos.get_untracked().len();
                                view! {
                                    <li class="flex items-center gap-1.5 rounded-xl bg-slate-50/60 border border-slate-100 p-1.5">
                                        <input type="checkbox" class="h-5 w-5 shrink-0 accent-indigo-600 cursor-pointer" prop:checked=fait on:change=move |_| cocher_todo(i) />
                                        <input type="text" maxlength="200"
                                            class=format!("min-w-0 flex-1 bg-transparent px-1.5 py-1 text-sm focus:outline-none focus:bg-white/70 rounded-lg {}", if fait { "line-through text-slate-400" } else { "text-slate-800" })
                                            prop:value=texte.clone()
                                            on:keydown=move |ev| if ev.key() == "Enter" { ev.prevent_default(); modifier_todo(i, event_target_value(&ev)); }
                                            on:change=move |ev| modifier_todo(i, event_target_value(&ev)) />
                                        <button type="button" title="Monter" prop:disabled=move || i == 0 class="text-slate-300 hover:text-indigo-500 disabled:opacity-30 transition" on:click=move |_| deplacer_todo(i, -1)>
                                            <Icone nom="chevron-up" class="w-4 h-4" />
                                        </button>
                                        <button type="button" title="Descendre" prop:disabled=dernier class="text-slate-300 hover:text-indigo-500 disabled:opacity-30 transition" on:click=move |_| deplacer_todo(i, 1)>
                                            <Icone nom="chevron-down" class="w-4 h-4" />
                                        </button>
                                        <button type="button" title="Supprimer" class="text-slate-300 hover:text-rose-500 transition" on:click=move |_| supprimer_todo(i)>
                                            <Icone nom="trash" class="w-4 h-4" />
                                        </button>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                        <div class="mt-2 flex items-center gap-2">
                            <input class=format!("{champ} flex-1 min-w-0") type="text" maxlength="200" placeholder="Ajouter une sous-tâche (ex. Exercice 1)"
                                prop:value=move || nouveau_todo.get()
                                on:input=move |ev| nouveau_todo.set(event_target_value(&ev))
                                on:keydown=move |ev| if ev.key() == "Enter" { ev.prevent_default(); ajouter_todo(); } />
                            <button type="button" class=BOUTON_PILULE on:click=move |_| ajouter_todo()>
                                <Icone nom="plus" class="w-4 h-4" />
                                "Ajouter"
                            </button>
                        </div>
                    </div>
                    {move || { let m = erreur.get(); (!m.is_empty()).then(|| view! { <p class="rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-3.5">{m}</p> }) }}
                    <div class="flex items-center gap-3">
                        <button type="submit" class="flex-1 px-5 py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 text-white text-sm font-semibold shadow-sm transition" prop:disabled=move || en_cours.get()>
                            {move || if en_cours.get() { "Enregistrement…" } else { "Enregistrer" }}
                        </button>
                        <button type="button" class="px-4 py-3 rounded-xl text-sm font-medium text-slate-500 hover:text-slate-700 transition" on:click=move |_| fermer()>"Annuler"</button>
                    </div>
                </form>
            </div>
        </div>
    }
}
