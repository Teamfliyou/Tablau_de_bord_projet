//! Fenêtre de mise à jour : annonce, téléchargement avec progression réelle, installation.

use crate::api;
use crate::icons::{Icone, Spinner};
use crate::state::Etat;
use crate::ui::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use tdb_shared::{PhaseMiseAJour, ProgressionMiseAJour};

const OCTETS_PAR_MO: f64 = 1024.0 * 1024.0;

fn mo(octets: u64) -> String {
    format!("{:.1}", octets as f64 / OCTETS_PAR_MO).replace('.', ",")
}

/// Pourcentage réel (octets reçus / taille annoncée) ; `None` si le serveur n'annonce pas la taille.
fn pourcentage(p: &ProgressionMiseAJour) -> Option<u32> {
    let total = p.total.filter(|t| *t > 0)?;
    Some(((p.telecharge.min(total) as f64 / total as f64) * 100.0).round() as u32)
}

#[component]
pub fn FenetreMiseAJour() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let en_cours = RwSignal::new(false);
    let progression = RwSignal::new(ProgressionMiseAJour::default());
    let erreur = RwSignal::new(String::new());

    let installer = move |_| {
        if en_cours.get_untracked() {
            return;
        }
        en_cours.set(true);
        erreur.set(String::new());
        progression.set(ProgressionMiseAJour::default());
        // Lecture périodique de la progression réelle mesurée côté Rust.
        spawn_local(async move {
            while en_cours.get_untracked() {
                if let Ok(p) = api::progression_mise_a_jour().await {
                    progression.set(p);
                }
                gloo_timers::future::TimeoutFuture::new(250).await;
            }
        });
        spawn_local(async move {
            // En cas de succès sous Windows, l'installateur ferme l'application puis la relance.
            if let Err(e) = api::installer_mise_a_jour().await {
                erreur.set(etat.message(e));
            }
            en_cours.set(false);
        });
    };
    let plus_tard = move |_| {
        if !en_cours.get_untracked() {
            etat.maj_fenetre.set(false);
        }
    };

    view! {
        <div class="fixed inset-0 z-[65] flex items-center justify-center p-4" role="dialog" aria-modal="true" aria-labelledby="maj-titre">
            <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-sm" on:click=plus_tard></div>
            <div class="relative w-full max-w-lg bg-white/90 backdrop-blur-2xl rounded-3xl border border-white/50 shadow-2xl p-6 sm:p-8">
                <div class="flex items-center gap-3">
                    <div class="w-11 h-11 rounded-2xl bg-gradient-to-br from-indigo-600 to-violet-600 flex items-center justify-center shadow-lg shadow-indigo-500/30 shrink-0">
                        <Icone nom="download" class="w-6 h-6 text-white" />
                    </div>
                    <div>
                        <h3 id="maj-titre" class="text-lg font-bold text-slate-900">"Mise à jour disponible"</h3>
                        <p class="text-sm text-slate-500">"Une nouvelle version de Tableau de bord étudiant est disponible."</p>
                    </div>
                </div>

                {move || etat.maj_info.get().map(|info| {
                    let notes = info.notes.clone();
                    view! {
                        <dl class="mt-5 grid grid-cols-2 gap-3 text-sm">
                            <div class="rounded-xl bg-white/60 border border-white/50 p-3">
                                <dt class="text-xs text-slate-400">"Version actuelle"</dt>
                                <dd class="font-semibold text-slate-700">{info.version_actuelle.clone()}</dd>
                            </div>
                            <div class="rounded-xl bg-indigo-50 border border-indigo-100 p-3">
                                <dt class="text-xs text-indigo-400">"Nouvelle version"</dt>
                                <dd class="font-semibold text-indigo-700">{info.version.clone()}</dd>
                            </div>
                        </dl>
                        {notes.map(|n| view! {
                            <div class="mt-4">
                                <p class="text-sm font-medium text-slate-700">"Améliorations"</p>
                                <pre class="mt-1.5 max-h-40 overflow-y-auto whitespace-pre-wrap font-sans text-xs text-slate-500 rounded-xl bg-white/50 border border-white/50 p-3">{n}</pre>
                            </div>
                        })}
                        {(!info.installable).then(|| view! {
                            <p class=format!("mt-4 {ALERTE_HORS_LIGNE}")>"Tu utilises la version portable : elle ne peut pas se mettre à jour toute seule. Télécharge le nouvel installateur depuis la page GitHub Releases du projet (Teamfliyou/Tablau_de_bord_projet)."</p>
                        })}
                    }
                })}

                <p class="mt-4 text-xs text-slate-400">"Tes devoirs, réglages et sauvegardes sont conservés. La signature de la mise à jour est vérifiée avant l'installation."</p>

                {move || en_cours.get().then(|| {
                    let p = progression.get();
                    let pct = pourcentage(&p);
                    let texte = match p.phase {
                        PhaseMiseAJour::Installation => "Installation en cours : l'application va redémarrer…".to_string(),
                        _ => match p.total {
                            Some(t) if t > 0 => format!("Téléchargement : {} Mo sur {} Mo", mo(p.telecharge), mo(t)),
                            _ => format!("Téléchargement : {} Mo", mo(p.telecharge)),
                        },
                    };
                    let largeur = match p.phase {
                        PhaseMiseAJour::Installation => 100,
                        _ => pct.unwrap_or(0),
                    };
                    view! {
                        <div class="mt-5">
                            <div class="flex items-center justify-between text-xs text-slate-500 mb-1.5">
                                <span>{texte}</span>
                                <span>{pct.filter(|_| p.phase == PhaseMiseAJour::Telechargement).map(|v| format!("{v} %"))}</span>
                            </div>
                            <div class="h-2 rounded-full bg-slate-200/70 overflow-hidden" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow=largeur>
                                <div class="h-full rounded-full bg-gradient-to-r from-indigo-500 to-violet-500 transition-all duration-200" style=format!("width: {largeur}%")></div>
                            </div>
                        </div>
                    }
                })}

                {move || { let e = erreur.get(); (!e.is_empty()).then(|| view! { <p class=format!("mt-5 {ALERTE_ERREUR}")>{e}</p> }) }}

                <div class="mt-6 flex flex-wrap items-center gap-3">
                    {move || {
                        let installable = etat.maj_info.get().is_some_and(|i| i.installable);
                        installable.then(|| view! {
                            <button type="button" class="flex-1 inline-flex items-center justify-center gap-2 px-5 py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 text-white text-sm font-semibold shadow-sm transition"
                                prop:disabled=move || en_cours.get() on:click=installer>
                                {move || if en_cours.get() { view! { <Spinner class="w-4 h-4" /> }.into_any() } else { ().into_any() }}
                                {move || if en_cours.get() { "Mise à jour en cours…" } else if erreur.get().is_empty() { "Mettre à jour maintenant" } else { "Réessayer" }}
                            </button>
                        })
                    }}
                    <button type="button" class="px-4 py-3 rounded-xl text-sm font-medium text-slate-500 hover:text-slate-700 disabled:opacity-40 transition"
                        prop:disabled=move || en_cours.get() on:click=plus_tard>"Plus tard"</button>
                </div>
            </div>
        </div>
    }
}
