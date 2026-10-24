//! Import guidé : dépôt du fichier, aperçu avec contrôle des lignes, confirmation, résultat.

use crate::api;
use crate::icons::Icone;
use crate::state::Etat;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::Value;
use tdb_shared::{DevoirEntree, IMPORT_MAX};

#[derive(Clone)]
struct Ligne {
    titre: String,
    detail: String,
    erreur: Option<String>,
}

#[derive(Clone)]
struct Apercu {
    nom: String,
    contenu: String,
    lignes: Vec<Ligne>,
}

impl Apercu {
    fn invalides(&self) -> usize {
        self.lignes.iter().filter(|l| l.erreur.is_some()).count()
    }
}

#[derive(Clone)]
enum Etape {
    Choix,
    Apercu(Apercu),
    Termine(usize),
}

/// Analyse le contenu du fichier : liste de devoirs attendue, chaque entrée est validée comme à l'import.
fn analyser(nom: &str, contenu: String) -> Result<Apercu, String> {
    let valeur: Value = serde_json::from_str(&contenu)
        .map_err(|_| "Ce fichier n'est pas un JSON valide.".to_string())?;
    let Value::Array(entrees) = valeur else {
        return Err("Une liste de devoirs est attendue (un tableau JSON).".into());
    };
    if entrees.is_empty() {
        return Err("Le fichier ne contient aucun devoir.".into());
    }
    if entrees.len() > IMPORT_MAX {
        return Err(format!("Trop de devoirs : {IMPORT_MAX} au maximum par import."));
    }
    let lignes = entrees
        .into_iter()
        .enumerate()
        .map(|(i, v)| {
            let titre_brut = v.get("titre").and_then(Value::as_str).unwrap_or("").to_string();
            match serde_json::from_value::<DevoirEntree>(v).map_err(|e| e.to_string()).and_then(|d| d.valider()) {
                Ok(d) => {
                    let mut morceaux = vec![];
                    if !d.matiere.is_empty() { morceaux.push(d.matiere.clone()); }
                    if !d.echeance.is_empty() { morceaux.push(d.echeance.clone()); }
                    if !d.todos.is_empty() { morceaux.push(format!("{} sous-tâche(s)", d.todos.len())); }
                    if !d.tags.is_empty() { morceaux.push(format!("{} tag(s)", d.tags.len())); }
                    Ligne { titre: d.titre, detail: morceaux.join(" · "), erreur: None }
                }
                Err(e) => Ligne {
                    titre: if titre_brut.is_empty() { format!("Entrée {}", i + 1) } else { titre_brut },
                    detail: String::new(),
                    erreur: Some(e),
                },
            }
        })
        .collect();
    Ok(Apercu { nom: nom.to_string(), contenu, lignes })
}

#[component]
pub fn FeuilleImport() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let etape = RwSignal::new(Etape::Choix);
    let erreur = RwSignal::new(String::new());
    let survol = RwSignal::new(false);
    let en_cours = RwSignal::new(false);
    let fermer = move || etat.import_ouvert.set(false);

    let recevoir = move |nom: String, texte: Result<String, String>| match texte.and_then(|t| analyser(&nom, t)) {
        Ok(a) => { erreur.set(String::new()); etape.set(Etape::Apercu(a)); }
        Err(e) => erreur.set(e),
    };
    let choisir = move || {
        spawn_local(async move {
            match api::choisir_fichier_json().await {
                Ok(Some(t)) => recevoir("Fichier JSON".into(), Ok(t)),
                Ok(None) => {}
                Err(e) => erreur.set(e.message),
            }
        });
    };
    let deposer = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        survol.set(false);
        let Some(f) = ev.data_transfer().and_then(|d| d.files()).and_then(|l| l.get(0)) else { return };
        let nom = f.name();
        spawn_local(async move {
            let r = api::lire_fichier(f).await.map_err(|e| e.message);
            recevoir(nom, r);
        });
    };
    let confirmer = move |contenu: String| {
        en_cours.set(true);
        spawn_local(async move {
            match api::importer_contenu(&contenu).await {
                Ok(n) => {
                    etat.charger_devoirs_async().await;
                    etape.set(Etape::Termine(n));
                }
                Err(e) => erreur.set(etat.message(e)),
            }
            en_cours.set(false);
        });
    };
    let barre = move |n: usize| move || match etape.get() { Etape::Choix => n <= 1, Etape::Apercu(_) => n <= 2, Etape::Termine(_) => true };

    view! {
        <div class="w-voile centre" on:click=move |_| fermer()>
            <div class="w-feuille w-glass" role="dialog" aria-modal="true" aria-labelledby="import-titre" on:click=|ev| ev.stop_propagation()>
                <div class="w-etapes" aria-hidden="true">
                    <i class:on=barre(1)></i><i class:on=barre(2)></i><i class:on=barre(3)></i>
                </div>
                <h2 id="import-titre">"Importer des devoirs"</h2>
                {move || match etape.get() {
                    Etape::Choix => view! {
                        <p class="w-aide">"Dépose un fichier JSON exporté depuis l'application. Rien n'est modifié avant ta confirmation."</p>
                        <div class="w-depot" class:survol=move || survol.get() role="button" tabindex="0"
                            on:dragover=move |ev| { ev.prevent_default(); survol.set(true); }
                            on:dragleave=move |_| survol.set(false)
                            on:drop=deposer
                            on:click=move |_| choisir()
                            on:keydown=move |ev| if ev.key() == "Enter" || ev.key() == " " { ev.prevent_default(); choisir(); }>
                            <Icone nom="file-json" />
                            <strong style="color:var(--label)">"Glisse ton fichier ici"</strong>
                            <span>"ou clique pour le choisir · 2 Mo maximum"</span>
                        </div>
                        {move || { let m = erreur.get(); (!m.is_empty()).then(|| view! {
                            <p role="alert" style="margin:12px 0 0;display:flex;gap:8px;align-items:flex-start;color:var(--danger);font-size:13px">
                                <Icone nom="warning" class="w-4 h-4 shrink-0" />{m}
                            </p>
                        }) }}
                        <div class="w-feuille-pied"><button type="button" class="w-btn" on:click=move |_| fermer()>"Annuler"</button></div>
                    }.into_any(),
                    Etape::Apercu(a) => {
                        let total = a.lignes.len();
                        let invalides = a.invalides();
                        let valides = total - invalides;
                        let contenu = a.contenu.clone();
                        view! {
                            <p class="w-aide">
                                {format!("{} · {total} entrée{} détectée{}", a.nom, if total > 1 { "s" } else { "" }, if total > 1 { "s" } else { "" })}
                            </p>
                            {(invalides > 0).then(|| view! {
                                <p role="alert" style="margin:0 0 8px;color:var(--danger);font-size:13px">
                                    {format!("{invalides} entrée(s) invalide(s) : corrige le fichier, l'import est atomique (tout ou rien).")}
                                </p>
                            })}
                            <ul class="w-liste-apercu" aria-label="Aperçu des devoirs">
                                {a.lignes.iter().map(|l| view! {
                                    <li>
                                        <Icone nom=if l.erreur.is_some() { "warning" } else { "check" } class="w-4 h-4 shrink-0" />
                                        <span style="flex:1;min-width:0">
                                            <span style="font-weight:600">{l.titre.clone()}</span>
                                            <span style="display:block;font-size:12px;color:var(--label-2)">{l.erreur.clone().unwrap_or_else(|| l.detail.clone())}</span>
                                        </span>
                                    </li>
                                }).collect_view()}
                            </ul>
                            {move || { let m = erreur.get(); (!m.is_empty()).then(|| view! { <p role="alert" style="color:var(--danger);font-size:13px">{m}</p> }) }}
                            <div class="w-feuille-pied">
                                <button type="button" class="w-btn" on:click=move |_| { erreur.set(String::new()); etape.set(Etape::Choix); }>"Choisir un autre fichier"</button>
                                <button type="button" class="w-btn w-btn-primaire" prop:disabled=move || invalides > 0 || en_cours.get()
                                    on:click=move |_| confirmer(contenu.clone())>
                                    {move || if en_cours.get() { "Import…".to_string() } else { format!("Importer {valides} devoir{}", if valides > 1 { "s" } else { "" }) }}
                                </button>
                            </div>
                        }.into_any()
                    }
                    Etape::Termine(n) => view! {
                        <div class="w-vide" style="padding:24px 0">
                            <div>
                                <Icone nom="check" />
                                <p><strong>{format!("{n} devoir{} importé{}", if n > 1 { "s" } else { "" }, if n > 1 { "s" } else { "" })}</strong>"Ils apparaissent dans la colonne choisie par leur statut."</p>
                            </div>
                        </div>
                        <div class="w-feuille-pied"><button type="button" class="w-btn w-btn-primaire" on:click=move |_| fermer()>"Terminé"</button></div>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}
