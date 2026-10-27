//! Onglet Configuration : ADE, Gemini, apparence, données locales, mises à jour,
//! intégrations IA (Hermes Agent / MCP), filtres et personnalisation.

use crate::api::{self, Mode};
use crate::icons::{Icone, Spinner};
use crate::pages::integrations::{SectionHermes, SectionMisesAJour};
use crate::state::{Dimension, Etat, StyleNav, Theme, MODELES_SUGGERES};
use crate::ui::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use tdb_shared::ConfigUpdate;

#[component]
pub fn Configuration() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let web = etat.mode == Mode::Web;

    let pseudo = RwSignal::new(String::new());
    let ade_url = RwSignal::new(String::new());
    let cle = RwSignal::new(String::new());
    let modele = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let erreur = RwSignal::new(String::new());
    let enregistrement = RwSignal::new(false);

    // Les champs suivent la configuration chargée ; la clé n'est jamais relue.
    Effect::new(move |_| {
        let c = etat.config.get();
        pseudo.set(c.username);
        ade_url.set(c.ade_url);
        modele.set(c.gemini_model);
        cle.set(String::new());
    });

    let enregistrer = move |retirer: bool| {
        spawn_local(async move {
            enregistrement.set(true);
            message.set(String::new());
            erreur.set(String::new());
            let maj = ConfigUpdate {
                username: pseudo.get_untracked(),
                ade_url: ade_url.get_untracked(),
                gemini_key: Some(cle.get_untracked()).filter(|k| !k.is_empty()),
                gemini_model: Some(modele.get_untracked()).filter(|m| !m.is_empty()),
                remove_gemini_key: retirer,
            };
            match etat.enregistrer_config(maj).await {
                Ok(()) => message.set(if retirer {
                    "Clé Gemini supprimée.".into()
                } else {
                    "Configuration enregistrée avec succès.".into()
                }),
                Err(e) => erreur.set(e),
            }
            enregistrement.set(false);
        });
    };

    // Données locales (application Windows) : import d'une ancienne base SQLite.
    let migration_en_cours = RwSignal::new(false);
    let migration_message = RwSignal::new(String::new());
    let migration_erreur = RwSignal::new(String::new());
    let migrer =
        move |_| {
            spawn_local(async move {
                migration_en_cours.set(true);
                migration_message.set(String::new());
                migration_erreur.set(String::new());
                match api::migrer_ancienne_base().await {
                    Ok(Some(r)) => {
                        migration_message.set(format!(
                        "{} devoir(s) importé(s), {} ignoré(s) (doublons ou invalides){}{}",
                        r.devoirs_importes,
                        r.devoirs_ignores,
                        if r.parametres_importes { ", paramètres repris" } else { "" },
                        if r.cle_gemini_trouvee {
                            ", clé Gemini transférée dans le gestionnaire d'identifiants Windows."
                        } else {
                            "."
                        }
                    ));
                        etat.charger_devoirs_async().await;
                        etat.charger_config().await;
                    }
                    Ok(None) => {}
                    Err(e) => migration_erreur.set(etat.message(e)),
                }
                migration_en_cours.set(false);
            });
        };

    // Filtres et personnalisation.
    let analyse_en_cours = RwSignal::new(false);
    let analyse_erreur = RwSignal::new(String::new());
    let analyse_message = RwSignal::new(String::new());
    let analyser = move |_| {
        spawn_local(async move {
            analyse_en_cours.set(true);
            analyse_erreur.set(String::new());
            analyse_message.set(String::new());
            match etat.analyser_categories().await {
                Ok(m) => analyse_message.set(m),
                Err(e) => analyse_erreur.set(e),
            }
            analyse_en_cours.set(false);
        });
    };

    view! {
        <section class="max-w-7xl mx-auto">
            <header>
                <h2 class="text-2xl font-bold text-slate-900">"Configuration"</h2>
                <p class="text-sm text-slate-500">"Paramètres de l'emploi du temps ADE, de l'IA et de l'affichage."</p>
            </header>

            <form class=format!("mt-8 {CARTE} p-6 space-y-5") on:submit=move |ev: leptos::ev::SubmitEvent| { ev.prevent_default(); enregistrer(false); }>
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1.5">"Pseudo / Identifiant"</label>
                    <input class=CHAMP_PLEIN type="text" maxlength="80" placeholder="Ton pseudo GitHub ou ton nom"
                        prop:value=move || pseudo.get() on:input=move |ev| pseudo.set(event_target_value(&ev)) />
                </div>
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1.5">"Lien d'export iCal / ADE"</label>
                    <input class=CHAMP_MONO type="url" placeholder="https://ton-ecole.example.com/ade/ics?data=…"
                        prop:value=move || ade_url.get() on:input=move |ev| ade_url.set(event_target_value(&ev)) />
                    <p class="text-xs text-slate-400 mt-1.5">"Trouvé dans ton application ADE : Emploi du temps → Export / iCal."</p>
                </div>
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1.5">"Clé API Gemini"</label>
                    <input class=CHAMP_MONO type="password" autocomplete="off"
                        placeholder=move || if etat.config.get().gemini_configured { "•••••••• (clé enregistrée — laisse vide pour la conserver)" } else { "AIza…" }
                        prop:value=move || cle.get() on:input=move |ev| cle.set(event_target_value(&ev)) />
                    <p class="text-xs text-slate-400 mt-1.5">
                        "Gratuite sur "<span class="font-medium">"https://aistudio.google.com/apikey"</span>
                        {if web { " — chiffrée sur le serveur, jamais renvoyée à l'interface." } else { " — stockée dans le gestionnaire d'identifiants Windows, jamais dans un fichier." }}
                    </p>
                    {move || etat.config.get().gemini_configured.then(|| view! {
                        <div class="mt-2 flex items-center gap-3 text-xs">
                            <span class="inline-flex items-center gap-1 text-emerald-600 font-medium"><Icone nom="check" class="w-3.5 h-3.5" />"Clé enregistrée"</span>
                            <button type="button" class="text-rose-500 hover:text-rose-600 font-medium" on:click=move |_| enregistrer(true)>"Supprimer la clé"</button>
                        </div>
                    })}
                </div>
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1.5">"Modèle Gemini"</label>
                    <input class=CHAMP_MONO type="text" list="modeles-gemini" placeholder="gemini-3.8-flash"
                        prop:value=move || modele.get() on:input=move |ev| modele.set(event_target_value(&ev)) />
                    <datalist id="modeles-gemini">
                        {MODELES_SUGGERES.iter().map(|m| view! { <option value=*m></option> }).collect_view()}
                    </datalist>
                    <p class="text-xs text-slate-400 mt-1.5">"Choisis un modèle proposé ou saisis le nom exact d'un modèle Gemini."</p>
                </div>

                {move || { let m = message.get(); (!m.is_empty()).then(|| view! { <p class=ALERTE_OK>{m}</p> }) }}
                {move || { let e = erreur.get(); (!e.is_empty()).then(|| view! { <p class="rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-3.5">{e}</p> }) }}

                <button type="submit" class=BOUTON_LARGE prop:disabled=move || enregistrement.get()>
                    {move || enregistrement.get().then(|| view! { <Spinner class="w-4 h-4" /> })}
                    {move || if enregistrement.get() { "Enregistrement…" } else { "Enregistrer la configuration" }}
                </button>
            </form>

            // ---- Apparence & interface
            <section class=format!("mt-8 {CARTE} p-6")>
                <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-xl bg-violet-50 flex items-center justify-center">
                        <svg class="w-5 h-5 text-violet-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 12h18M3 6h18M3 18h18M6 3v18M12 3v18M18 3v18" />
                        </svg>
                    </div>
                    <div>
                        <h3 class="font-semibold text-slate-900">"🎨 Apparence & Interface"</h3>
                        <p class="text-xs text-slate-400">"Choisis le style de navigation principal de ton tableau de bord."</p>
                    </div>
                </div>
                <div class="mt-5">
                    <label class="block text-sm font-medium text-slate-700 mb-1.5">"Style de navigation"</label>
                    <select class=format!("{CHAMP_PLEIN} text-slate-700")
                        prop:value=move || if etat.style_nav.get() == StyleNav::Dock { "dock" } else { "classic" }
                        on:change=move |ev| {
                            etat.style_nav.set(if event_target_value(&ev) == "classic" { StyleNav::Classic } else { StyleNav::Dock });
                            etat.enregistrer_preferences();
                        }>
                        <option value="dock">"Modern Dock (Bandeau en bas)"</option>
                        <option value="classic">"Classic Sidebar (Barre latérale)"</option>
                    </select>
                    <p class="text-xs text-slate-400 mt-1.5">"Le choix est enregistré automatiquement et appliqué immédiatement."</p>
                </div>
                <div class="mt-5">
                    <label class="block text-sm font-medium text-slate-700 mb-1.5">"Thème"</label>
                    <select class=format!("{CHAMP_PLEIN} text-slate-700")
                        prop:value=move || etat.theme.get().code()
                        on:change=move |ev| {
                            if let Some(t) = Theme::depuis_code(&event_target_value(&ev)) { etat.theme.set(t); }
                            etat.enregistrer_preferences();
                        }>
                        <option value="auto">{if web { "Automatique (suit le système)" } else { "Automatique (suit Windows)" }}</option>
                        <option value="light">"Clair"</option>
                        <option value="dark">"Sombre"</option>
                    </select>
                </div>
            </section>

            // ---- Données locales (application Windows uniquement)
            {(!web).then(|| view! {
                <section class=format!("mt-8 {CARTE} p-6")>
                    <h3 class="font-semibold text-slate-900">"💾 Données locales"</h3>
                    <p class="text-xs text-slate-400 mt-1">"Tes devoirs et réglages sont enregistrés sur ce PC (dossier AppData) et conservés entre les mises à jour. Une sauvegarde automatique est créée chaque jour."</p>
                    <div class="mt-4 flex flex-wrap items-center gap-3">
                        <button type="button" class="px-4 py-2.5 rounded-xl bg-white/70 border border-white/50 text-sm font-semibold text-slate-700 hover:border-indigo-300 disabled:opacity-60 transition"
                            prop:disabled=move || migration_en_cours.get() on:click=migrer>
                            {move || if migration_en_cours.get() { "Import en cours…" } else { "Importer l'ancienne base (app.db)" }}
                        </button>
                        {move || { let v = etat.version.get(); (!v.is_empty()).then(|| view! { <span class="text-xs text-slate-400">{format!("Version {v}")}</span> }) }}
                    </div>
                    {move || { let m = migration_message.get(); (!m.is_empty()).then(|| view! { <p class=format!("mt-4 {ALERTE_OK}")>{m}</p> }) }}
                    {move || { let e = migration_erreur.get(); (!e.is_empty()).then(|| view! { <p class="mt-4 rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-3.5">{e}</p> }) }}
                </section>
                <SectionMisesAJour />
                <SectionHermes />
            })}

            // ---- Compte (version Web)
            {web.then(|| view! {
                <section class=format!("mt-8 {CARTE} p-6")>
                    <h3 class="font-semibold text-slate-900">"👤 Compte"</h3>
                    <p class="text-xs text-slate-400 mt-1">"Tes données sont enregistrées sur le serveur et liées à ton compte."</p>
                    <div class="mt-4 flex flex-wrap items-center gap-3">
                        <button type="button" class="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-white/70 border border-white/50 text-sm font-semibold text-slate-700 hover:border-rose-300 hover:text-rose-600 transition"
                            on:click=move |_| etat.deconnecter()>
                            <Icone nom="logout" class="w-4 h-4" />
                            "Se déconnecter"
                        </button>
                        {move || { let v = etat.version.get(); (!v.is_empty()).then(|| view! { <span class="text-xs text-slate-400">{format!("Version {v}")}</span> }) }}
                    </div>
                </section>
            })}

            // ---- Filtres & personnalisation
            <section class=format!("mt-8 {CARTE} p-6")>
                <div class="flex flex-wrap items-start justify-between gap-4">
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 rounded-xl bg-amber-50 flex items-center justify-center">
                            <Icone nom="filter" class="w-5 h-5 text-amber-600" />
                        </div>
                        <div>
                            <h3 class="font-semibold text-slate-900">"Filtres & Personnalisation"</h3>
                            <p class="text-xs text-slate-400">"Analyse automatique de ton flux ADE : coche tes groupes (TP/TD/Khôlles), tes épreuves et tes matières."</p>
                        </div>
                    </div>
                    <button class="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-gradient-to-r from-amber-500 to-orange-500 hover:from-amber-400 hover:to-orange-400 disabled:opacity-60 text-white text-sm font-semibold shadow-sm transition"
                        prop:disabled=move || analyse_en_cours.get() on:click=analyser>
                        {move || if analyse_en_cours.get() { view! { <Spinner class="w-4 h-4" /> }.into_any() } else { view! { <Icone nom="search" class="w-4 h-4" /> }.into_any() }}
                        {move || if analyse_en_cours.get() { "Analyse en cours…" } else { "Analyser mon lien ADE" }}
                    </button>
                </div>

                {move || { let e = analyse_erreur.get(); (!e.is_empty()).then(|| view! { <p class=format!("mt-5 {ALERTE_ERREUR}")>{e}</p> }) }}
                {move || { let m = analyse_message.get(); (!m.is_empty()).then(|| view! { <p class="mt-5 rounded-xl bg-emerald-50 border border-emerald-200 text-emerald-700 text-sm p-4">{m}</p> }) }}

                {move || {
                    let cat = etat.categories.get();
                    if cat.groupes.is_empty() {
                        if analyse_en_cours.get() { return ().into_any(); }
                        return view! {
                            <div class=format!("mt-6 {VIDE} p-8")>
                                <p class="text-sm text-slate-500">"Clique sur « Analyser mon lien ADE » pour détecter automatiquement tes groupes (TP/TD/Khôlles), les types d'épreuves et les matières."</p>
                            </div>
                        }.into_any();
                    }
                    view! {
                        <div class="mt-6 border-t border-slate-100 pt-5">
                            <div class="flex flex-wrap items-center justify-between gap-3">
                                <h4 class="font-semibold text-slate-800 text-sm">"Groupes détectés"</h4>
                                <div class="flex items-center gap-2 text-xs">
                                    <button type="button" class="px-3 py-1.5 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium transition" on:click=move |_| etat.tout_cocher(Dimension::Groupes)>"Tout cocher"</button>
                                    <button type="button" class="px-3 py-1.5 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium transition" on:click=move |_| etat.tout_decocher(Dimension::Groupes)>"Tout décocher"</button>
                                    <span class="text-slate-400 tabular-nums">{move || format!("{}/{} actifs", etat.filtres_groupes.get().len(), etat.categories.get().groupes.len())}</span>
                                </div>
                            </div>
                            <div class="mt-3 flex flex-wrap gap-2">
                                {cat.groupes.iter().map(|g| {
                                    let (g1, g2, g3) = (g.clone(), g.clone(), g.clone());
                                    view! {
                                        <button type="button"
                                            class=move || format!("inline-flex items-center gap-1.5 px-3.5 py-2 rounded-full text-sm font-medium border transition {}",
                                                if etat.filtres_groupes.get().contains(&g1) { "bg-indigo-600 text-white border-indigo-600 shadow-sm" } else { "bg-white/60 backdrop-blur text-slate-600 border-white/50 hover:border-indigo-300" })
                                            on:click=move |_| etat.basculer_filtre(Dimension::Groupes, &g2)>
                                            {move || etat.filtres_groupes.get().contains(&g3).then(|| view! { <Icone nom="check" class="w-3.5 h-3.5" /> })}
                                            {g.clone()}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                            <p class="text-xs text-slate-400 mt-3">"Les cours sans groupe (CM, réunions…) restent toujours visibles. Les TP/TD/Khôlles des groupes non cochés sont masqués du Dashboard et de la Semaine."</p>
                        </div>
                        <div class="mt-6 grid grid-cols-1 sm:grid-cols-2 gap-5 border-t border-slate-100 pt-5">
                            {puces(etat, Dimension::Types, "Types d'épreuves", "Aucun type spécifique détecté.", "Coche les épreuves à garder (TP, TD, DS…). Les séances d'un type non coché sont masquées.", "bg-sky-600 text-white border-sky-600", "bg-sky-50 text-sky-700 border-sky-100 hover:border-sky-300")}
                            {puces(etat, Dimension::Matieres, "Matières", "Aucune matière détectée.", "Coche les matières à garder. Les cours des matières non cochées sont masqués.", "bg-violet-600 text-white border-violet-600", "bg-violet-50 text-violet-700 border-violet-100 hover:border-violet-300")}
                        </div>
                    }.into_any()
                }}
            </section>
        </section>
    }
}

/// Colonne de puces cochables (types d'épreuves ou matières).
fn puces(
    etat: Etat,
    dim: Dimension,
    titre: &'static str,
    vide: &'static str,
    aide: &'static str,
    actif: &'static str,
    inactif: &'static str,
) -> impl IntoView {
    let valeurs = move || match dim {
        Dimension::Groupes => etat.categories.get().groupes,
        Dimension::Types => etat.categories.get().types,
        Dimension::Matieres => etat.categories.get().matieres,
    };
    let coches = etat.signal_filtre(dim);
    view! {
        <div>
            <div class="flex flex-wrap items-center justify-between gap-3 mb-2">
                <h4 class="font-semibold text-slate-800 text-sm">{titre}</h4>
                <div class="flex items-center gap-2 text-xs">
                    <button type="button" class="px-2 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium transition" on:click=move |_| etat.tout_cocher(dim)>"Tout"</button>
                    <button type="button" class="px-2 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium transition" on:click=move |_| etat.tout_decocher(dim)>"Aucun"</button>
                    <span class="text-slate-400 tabular-nums">{move || format!("{}/{}", coches.get().len(), valeurs().len())}</span>
                </div>
            </div>
            <div class="flex flex-wrap gap-1.5">
                {move || {
                    let liste = valeurs();
                    if liste.is_empty() {
                        return view! { <span class="text-xs text-slate-400">{vide}</span> }.into_any();
                    }
                    liste.into_iter().map(|v| {
                        let (v1, v2, v3) = (v.clone(), v.clone(), v.clone());
                        view! {
                            <button type="button"
                                class=move || format!("inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium border transition {}",
                                    if coches.get().contains(&v1) { actif } else { inactif })
                                on:click=move |_| etat.basculer_filtre(dim, &v2)>
                                {move || coches.get().contains(&v3).then(|| view! { <Icone nom="check" class="w-3 h-3" /> })}
                                {v}
                            </button>
                        }
                    }).collect_view().into_any()
                }}
            </div>
            <p class="text-xs text-slate-400 mt-2">{aide}</p>
        </div>
    }
}
