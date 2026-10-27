//! Sections de la Configuration propres à l'application Windows :
//! mises à jour (version installée / publiée) et Intégrations IA (Hermes Agent via MCP).

use crate::api;
use crate::icons::{Icone, Spinner};
use crate::state::Etat;
use crate::ui::*;
use crate::util;
use leptos::prelude::*;
use leptos::task::spawn_local;
use tdb_shared::{EtatMcp, TestMcp, NOM_SERVEUR_MCP, OUTILS_MCP};

const BOUTON_SECONDAIRE: &str = "inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-white/70 border border-white/50 text-sm font-semibold text-slate-700 hover:border-indigo-300 disabled:opacity-60 transition";
const LIGNE: &str = "rounded-xl bg-white/60 border border-white/50 p-3";

/// Paramètres → Rechercher des mises à jour.
#[component]
pub fn SectionMisesAJour() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let derniere = move || {
        etat.maj_derniere
            .get()
            .and_then(|v| v.derniere_version)
            .unwrap_or_else(|| "—".into())
    };
    let date = move || {
        etat.maj_derniere
            .get()
            .and_then(|v| v.date)
            .map(|d| util::date_heure(&d))
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| "—".into())
    };
    view! {
        <section class=format!("mt-8 {CARTE} p-6")>
            <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-xl bg-sky-50 flex items-center justify-center">
                    <Icone nom="download" class="w-5 h-5 text-sky-600" />
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900">"Mises à jour"</h3>
                    <p class="text-xs text-slate-400">"Les nouvelles versions publiées sur GitHub sont vérifiées au démarrage. Leur signature est contrôlée avant toute installation ; tes données sont conservées."</p>
                </div>
            </div>
            <dl class="mt-5 grid grid-cols-1 sm:grid-cols-3 gap-3 text-sm">
                <div class=LIGNE>
                    <dt class="text-xs text-slate-400">"Version installée"</dt>
                    <dd class="font-semibold text-slate-700">{move || { let v = etat.version.get(); if v.is_empty() { "—".to_string() } else { v } }}</dd>
                </div>
                <div class=LIGNE>
                    <dt class="text-xs text-slate-400">"Dernière version publiée"</dt>
                    <dd class="font-semibold text-slate-700">{derniere}</dd>
                </div>
                <div class=LIGNE>
                    <dt class="text-xs text-slate-400">"Date de publication"</dt>
                    <dd class="font-semibold text-slate-700">{date}</dd>
                </div>
            </dl>
            <div class="mt-4 flex flex-wrap items-center gap-3">
                <button type="button" class=BOUTON_SECONDAIRE
                    prop:disabled=move || etat.maj_verification.get()
                    on:click=move |_| { spawn_local(async move { etat.verifier_maj(true).await; }); }>
                    {move || if etat.maj_verification.get() { view! { <Spinner class="w-4 h-4" /> }.into_any() } else { view! { <Icone nom="refresh" class="w-4 h-4" /> }.into_any() }}
                    {move || if etat.maj_verification.get() { "Recherche en cours…" } else { "Rechercher des mises à jour" }}
                </button>
                {move || etat.maj_info.get().map(|i| view! {
                    <button type="button" class=BOUTON_PRIMAIRE on:click=move |_| etat.maj_fenetre.set(true)>
                        <Icone nom="download" class="w-4 h-4" />
                        {format!("Installer la version {}", i.version)}
                    </button>
                })}
            </div>
            <p class="mt-3 text-xs text-slate-400">"Le téléchargement et l'installation s'affichent avec leur progression dans la fenêtre de mise à jour ; l'application redémarre ensuite toute seule."</p>
            {move || { let m = etat.maj_message.get(); (!m.is_empty()).then(|| view! { <p class=format!("mt-4 {ALERTE_OK}")>{m}</p> }) }}
            {move || { let e = etat.maj_erreur.get(); (!e.is_empty()).then(|| view! { <p class=format!("mt-4 {ALERTE_ERREUR}")>{e}</p> }) }}
            {move || { let e = etat.maj_echec_auto.get(); (!e.is_empty() && etat.maj_erreur.get().is_empty()).then(|| view! {
                <p class=format!("mt-4 {ALERTE_HORS_LIGNE}")>{format!("La vérification automatique au démarrage a échoué : {e}")}</p>
            }) }}
        </section>
    }
}

/// Paramètres → Intégrations IA → Hermes Agent / MCP.
#[component]
pub fn SectionHermes() -> impl IntoView {
    let etat_app = expect_context::<Etat>();
    let etat = RwSignal::new(None::<EtatMcp>);
    let erreur = RwSignal::new(String::new());
    let enregistrement = RwSignal::new(false);
    let test = RwSignal::new(None::<TestMcp>);
    let test_en_cours = RwSignal::new(false);
    let copie = RwSignal::new(String::new());

    spawn_local(async move {
        match api::etat_mcp().await {
            Ok(e) => etat.set(Some(e)),
            Err(e) => erreur.set(etat_app.message(e)),
        }
    });

    let definir = move |actif: bool, ecriture: bool| {
        if enregistrement.get_untracked() {
            return;
        }
        enregistrement.set(true);
        erreur.set(String::new());
        spawn_local(async move {
            match api::definir_mcp(actif, ecriture).await {
                Ok(e) => etat.set(Some(e)),
                Err(e) => erreur.set(etat_app.message(e)),
            }
            enregistrement.set(false);
        });
    };
    let actif = move || etat.get().is_some_and(|e| e.actif);
    let ecriture = move || etat.get().is_some_and(|e| e.ecriture);

    let tester = move |_| {
        test_en_cours.set(true);
        test.set(None);
        spawn_local(async move {
            match api::tester_mcp().await {
                Ok(r) => test.set(Some(r)),
                Err(e) => test.set(Some(TestMcp {
                    ok: false,
                    message: etat_app.message(e),
                    ..Default::default()
                })),
            }
            test_en_cours.set(false);
        });
    };
    let copier = move |_| {
        let Some(texte) = etat.get_untracked().map(|e| e.configuration) else {
            return;
        };
        spawn_local(async move {
            copie.set(if util::copier(&texte).await {
                "Configuration copiée.".into()
            } else {
                "Copie impossible : sélectionne le texte ci-dessus et copie-le avec Ctrl+C.".into()
            });
        });
    };

    let interrupteur = |allume: bool| {
        format!(
            "relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition disabled:opacity-50 {}",
            if allume { "bg-indigo-600" } else { "bg-slate-200" }
        )
    };
    let pastille = |allume: bool| {
        format!(
            "inline-block h-5 w-5 rounded-full bg-white shadow transition {}",
            if allume {
                "translate-x-5"
            } else {
                "translate-x-0.5"
            }
        )
    };

    view! {
        <section class=format!("mt-8 {CARTE} p-6")>
            <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-xl bg-violet-50 flex items-center justify-center">
                    <Icone nom="sparkles" class="w-5 h-5 text-violet-600" />
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900">"Intégrations IA — Hermes Agent / MCP"</h3>
                    <p class="text-xs text-slate-400">"Permet à Hermes Agent de consulter tes devoirs (un projet = une matière) grâce à un serveur MCP local. Aucun port réseau n'est ouvert : Hermes lance le serveur sur ce PC, même quand le tableau de bord est fermé."</p>
                </div>
            </div>

            {move || { let e = erreur.get(); (!e.is_empty()).then(|| view! { <p class=format!("mt-5 {ALERTE_ERREUR}")>{e}</p> }) }}

            {move || etat.get().map(|e| {
                let disponible = e.executable.clone();
                view! {
                    <div class="mt-5">
                        {match disponible {
                            Some(chemin) => view! {
                                <p class="inline-flex items-center gap-1.5 text-xs font-medium text-emerald-600"><Icone nom="check" class="w-3.5 h-3.5" />"Serveur MCP disponible"</p>
                                <p class="mt-1 text-xs text-slate-400 font-mono break-all">{chemin}</p>
                            }.into_any(),
                            None => view! {
                                <p class=ALERTE_HORS_LIGNE>"Serveur MCP introuvable à côté de l'application (tdb-mcp.exe). Il est fourni par l'installateur à partir de la version 5.1.0 : la version portable n'en dispose pas."</p>
                            }.into_any(),
                        }}
                    </div>
                }
            })}

            <div class="mt-5 space-y-4">
                <div class="flex items-start justify-between gap-4">
                    <div>
                        <p class="text-sm font-medium text-slate-700">"Activer l'intégration"</p>
                        <p class="text-xs text-slate-400">"Désactivée : Hermes ne peut rien lire. L'effet est immédiat, sans redémarrage."</p>
                    </div>
                    <button type="button" role="switch" aria-label="Activer l'intégration Hermes Agent"
                        aria-checked=move || actif().to_string()
                        class=move || interrupteur(actif())
                        prop:disabled=move || enregistrement.get() || etat.get().is_none()
                        on:click=move |_| definir(!actif(), ecriture())>
                        <span class=move || pastille(actif())></span>
                    </button>
                </div>
                <div class="flex items-start justify-between gap-4">
                    <div>
                        <p class="text-sm font-medium text-slate-700">"Autoriser les modifications"</p>
                        <p class="text-xs text-slate-400">"Désactivé par défaut (lecture seule). Activé : Hermes peut créer ou modifier des tâches et des projets, toujours après un aperçu que tu dois confirmer. Aucune suppression n'est possible."</p>
                    </div>
                    <button type="button" role="switch" aria-label="Autoriser les modifications par Hermes Agent"
                        aria-checked=move || ecriture().to_string()
                        class=move || interrupteur(ecriture())
                        prop:disabled=move || enregistrement.get() || !actif()
                        on:click=move |_| definir(actif(), !ecriture())>
                        <span class=move || pastille(ecriture())></span>
                    </button>
                </div>
            </div>

            <div class="mt-6 border-t border-slate-100 pt-5">
                <h4 class="font-semibold text-slate-800 text-sm">"Outils disponibles pour Hermes"</h4>
                <ul class="mt-3 grid grid-cols-1 md:grid-cols-2 gap-2">
                    {OUTILS_MCP.iter().map(|o| {
                        let ecr = o.ecriture;
                        view! {
                            <li class=move || format!("{LIGNE} text-xs transition {}", if ecr && !ecriture() { "opacity-50" } else { "" })>
                                <div class="flex items-center gap-2">
                                    <span class="font-mono font-semibold text-slate-700">{o.nom}</span>
                                    <span class=if ecr { "px-1.5 py-0.5 rounded-md bg-amber-50 text-amber-700" } else { "px-1.5 py-0.5 rounded-md bg-sky-50 text-sky-700" }>
                                        {if ecr { "écriture" } else { "lecture" }}
                                    </span>
                                </div>
                                <p class="mt-1 text-slate-500">{o.description}</p>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </div>

            <div class="mt-6 border-t border-slate-100 pt-5">
                <h4 class="font-semibold text-slate-800 text-sm">"Connexion à Hermes Agent"</h4>
                <ol class="mt-2 text-xs text-slate-500 list-decimal list-inside space-y-1">
                    <li>"Ouvre la configuration de Hermes : commande "<span class="font-mono">"hermes config edit"</span>" (fichier "<span class="font-mono">"%LOCALAPPDATA%\\hermes\\config.yaml"</span>")."</li>
                    <li>"Ajoute le bloc ci-dessous (fusionne-le avec une section "<span class="font-mono">"mcp_servers"</span>" existante)."</li>
                    <li>"Vérifie avec "<span class="font-mono">{format!("hermes mcp test {NOM_SERVEUR_MCP}")}</span>", puis ouvre une nouvelle session Hermes (ou "<span class="font-mono">"/reload-mcp"</span>")."</li>
                </ol>
                <pre class="mt-3 max-h-48 overflow-auto whitespace-pre rounded-xl bg-slate-900/90 text-slate-100 text-xs font-mono p-4 select-all">
                    {move || etat.get().map(|e| e.configuration).unwrap_or_default()}
                </pre>
                <div class="mt-3 flex flex-wrap items-center gap-3">
                    <button type="button" class=BOUTON_SECONDAIRE prop:disabled=move || etat.get().is_none() on:click=copier>
                        <Icone nom="copy" class="w-4 h-4" />"Copier la configuration"
                    </button>
                    <button type="button" class=BOUTON_SECONDAIRE prop:disabled=move || test_en_cours.get() on:click=tester>
                        {move || if test_en_cours.get() { view! { <Spinner class="w-4 h-4" /> }.into_any() } else { view! { <Icone nom="bolt" class="w-4 h-4" /> }.into_any() }}
                        {move || if test_en_cours.get() { "Test en cours…" } else { "Tester la connexion" }}
                    </button>
                    {move || { let m = copie.get(); (!m.is_empty()).then(|| view! { <span class="text-xs text-slate-500">{m}</span> }) }}
                </div>
                {move || test.get().map(|t| {
                    let details = t.version.clone().map(|v| format!(" Serveur tdb-mcp {v}, {} outil(s).", t.outils.len())).unwrap_or_default();
                    view! {
                        <p class=format!("mt-4 {}", if t.ok { ALERTE_OK } else { ALERTE_ERREUR })>{format!("{}{details}", t.message)}</p>
                    }
                })}
            </div>
        </section>
    }
}
