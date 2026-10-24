//! Tableau Kanban : colonnes, cartes, glisser-déposer, création rapide, menu contextuel, clavier.

use super::{focaliser, raccourci, Ui};
use crate::icons::Icone;
use crate::state::{libelle_echeance, type_libelle, Etat, COLONNES};
use crate::util;
use leptos::prelude::*;
use tdb_shared::Devoir;

const COULEURS_COLONNE: [&str; 3] = ["#ff6b6b", "#f5a524", "#34c759"];

fn classe_echeance(d: &Devoir) -> &'static str {
    if d.statut == "termine" {
        return "w-chip w-chip-neutre";
    }
    match util::jours_restants(&d.echeance) {
        j if j < 0 => "w-chip w-chip-retard",
        j if j <= 3 => "w-chip w-chip-bientot",
        _ => "w-chip w-chip-neutre",
    }
}

#[component]
pub fn Tableau() -> impl IntoView {
    let etat = expect_context::<Etat>();
    // Devoir en cours de glissement et colonne survolée.
    let glisse: RwSignal<Option<i64>> = RwSignal::new(None);
    let cible: RwSignal<Option<&'static str>> = RwSignal::new(None);
    let filtre_actif =
        move || etat.filtre_tag.get().is_some() || !etat.recherche.get().trim().is_empty();

    view! {
        <div style="display:flex;flex-direction:column;flex:1;min-height:0">
            {move || filtre_actif().then(|| view! {
                <div style="display:flex;align-items:center;gap:8px;margin:0 2px 10px;font-size:12px;color:var(--label-2)">
                    <Icone nom="filter" class="w-4 h-4" />
                    <span>
                        "Filtre actif"
                        {move || { let q = etat.recherche.get(); (!q.trim().is_empty()).then(|| format!(" · « {} »", q.trim())) }}
                        {move || etat.filtre_tag.get().and_then(|id| etat.tags.with(|t| t.iter().find(|g| g.id == id).map(|g| format!(" · tag {}", g.nom))))}
                    </span>
                    <button type="button" class="w-btn w-btn-plat" style="min-height:22px;padding:0 8px"
                        on:click=move |_| { etat.filtre_tag.set(None); etat.recherche.set(String::new()); }>"Effacer"</button>
                </div>
            })}
            {move || etat.erreur_devoirs.get().map(|m| view! {
                <p role="alert" style="margin:0 2px 10px;padding:9px 12px;border-radius:var(--r-sm);background:color-mix(in srgb,var(--danger) 12%,transparent);color:var(--danger);font-size:13px">{m}</p>
            })}
            <div class="w-board" role="group" aria-label="Tableau Kanban">
                {COLONNES.iter().enumerate().map(|(i, (id, libelle, _))| {
                    colonne(etat, id, libelle, COULEURS_COLONNE[i], glisse, cible)
                }).collect_view()}
            </div>
        </div>
    }
}

fn colonne(
    etat: Etat,
    id: &'static str,
    libelle: &'static str,
    couleur: &'static str,
    glisse: RwSignal<Option<i64>>,
    cible: RwSignal<Option<&'static str>>,
) -> impl IntoView {
    let ui = expect_context::<Ui>();
    let nb = move || etat.devoirs_visibles(id).len();
    let titre_nouveau = RwSignal::new(String::new());
    let saisie_ouverte = move || ui.ajout_col.get() == Some(id);
    let valider = move || {
        let t = titre_nouveau.get_untracked();
        if !t.trim().is_empty() {
            etat.ajout_rapide(id, &t);
            titre_nouveau.set(String::new());
        }
    };

    view! {
        <section class="w-col" class:cible=move || cible.get() == Some(id) aria-label=libelle
            on:dragover=move |ev| {
                if glisse.get_untracked().is_some() {
                    ev.prevent_default();
                    if let Some(dt) = ev.data_transfer() { dt.set_drop_effect("move"); }
                    if cible.get_untracked() != Some(id) { cible.set(Some(id)); }
                }
            }
            on:dragleave=move |ev| {
                // On ne désactive que lorsqu'on quitte réellement la colonne.
                let sortie = ev.related_target().is_none();
                if sortie && cible.get_untracked() == Some(id) { cible.set(None); }
            }
            on:drop=move |ev| {
                ev.prevent_default();
                if let Some(d) = glisse.get_untracked() { etat.deplacer_vers(d, id); }
                glisse.set(None);
                cible.set(None);
            }>
            <div class="w-col-tete">
                <span class="w-col-puce" style=format!("background:{couleur}") aria-hidden="true"></span>
                <h3>{libelle}</h3>
                <span class="w-col-nb" aria-label=move || format!("{} élément(s)", nb())>{nb}</span>
            </div>
            <div class="w-col-liste" role="list">
                {move || {
                    let liste = etat.devoirs_visibles(id);
                    if liste.is_empty() {
                        let total = etat.devoirs.with(|l| l.iter().filter(|d| d.statut == id).count());
                        let (titre, aide) = if total > 0 {
                            ("Aucun résultat", "Aucun devoir ne correspond au filtre.")
                        } else {
                            match id {
                                "a_faire" => ("Rien à faire", "Ajoute un devoir ou importe-en depuis un fichier."),
                                "en_cours" => ("Rien en cours", "Glisse un devoir ici quand tu t'y mets."),
                                _ => ("Rien de terminé", "Les devoirs terminés apparaîtront ici."),
                            }
                        };
                        return view! {
                            <div class="w-vide" role="listitem">
                                <div>
                                    <Icone nom=if id == "termine" { "check" } else { "inbox" } />
                                    <p><strong>{titre}</strong>{aide}</p>
                                </div>
                            </div>
                        }.into_any();
                    }
                    liste.into_iter().map(|d| carte(etat, d, id, glisse, cible)).collect_view().into_any()
                }}
            </div>
            <div class="w-col-pied">
                {move || if saisie_ouverte() {
                    view! {
                        <input class="w-champ" type="text" maxlength="200" placeholder="Titre du devoir, puis Entrée" aria-label=format!("Nouveau devoir dans {libelle}")
                            autofocus
                            prop:value=move || titre_nouveau.get()
                            on:input=move |ev| titre_nouveau.set(event_target_value(&ev))
                            on:keydown=move |ev| match ev.key().as_str() {
                                "Enter" => { ev.prevent_default(); valider(); }
                                "Escape" => { ev.stop_propagation(); titre_nouveau.set(String::new()); ui.ajout_col.set(None); }
                                _ => {}
                            }
                            on:blur=move |_| {
                                if titre_nouveau.get_untracked().trim().is_empty() { ui.ajout_col.set(None); } else { valider(); ui.ajout_col.set(None); }
                            }
                            node_ref={
                                let r = NodeRef::<leptos::html::Input>::new();
                                r.on_load(move |el| { let _ = el.focus(); });
                                r
                            } />
                    }.into_any()
                } else {
                    view! {
                        <button type="button" class="w-ajout-rapide" title=format!("Nouveau devoir ({})", raccourci("N")) on:click=move |_| ui.ajout_col.set(Some(id))>
                            <Icone nom="plus" />
                            "Nouveau devoir"
                        </button>
                    }.into_any()
                }}
            </div>
        </section>
    }
}

fn carte(
    etat: Etat,
    d: Devoir,
    colonne_id: &'static str,
    glisse: RwSignal<Option<i64>>,
    cible: RwSignal<Option<&'static str>>,
) -> impl IntoView {
    let id = d.id;
    let total = d.todos.len();
    let faits = d.todos.iter().filter(|t| t.fait).count();
    let pourcent = (faits * 100).checked_div(total).unwrap_or(0);
    let aria = format!(
        "{}{}{}",
        d.titre,
        if d.matiere.is_empty() {
            String::new()
        } else {
            format!(", {}", d.matiere)
        },
        if d.echeance.is_empty() {
            String::new()
        } else {
            format!(", {}", libelle_echeance(&d.echeance))
        }
    );
    let selectionnee = move || {
        etat.edition
            .with(|e| e.as_ref().is_some_and(|x| x.id == id))
    };
    let d_ouvrir = d.clone();
    let ordre = ["a_faire", "en_cours", "termine"];

    // Navigation au clavier : flèches entre cartes et colonnes, Alt+flèche gauche/droite pour déplacer.
    let au_clavier = move |ev: leptos::ev::KeyboardEvent| {
        let cle = ev.key();
        match cle.as_str() {
            "Enter" | " " => {
                ev.prevent_default();
                if let Some(d) = etat.devoir_par_id(id) {
                    etat.edition.set(Some(d));
                }
            }
            "Delete" | "Backspace" => {
                ev.prevent_default();
                etat.supprimer_annulable(id);
            }
            "ArrowDown" | "ArrowUp" => {
                ev.prevent_default();
                let liste = etat.devoirs_visibles(colonne_id);
                if let Some(i) = liste.iter().position(|x| x.id == id) {
                    let j = if cle == "ArrowDown" {
                        i + 1
                    } else {
                        i.wrapping_sub(1)
                    };
                    if let Some(v) = liste.get(j) {
                        focaliser(&format!("[data-carte='{}']", v.id));
                    }
                }
            }
            "ArrowLeft" | "ArrowRight" => {
                ev.prevent_default();
                let Some(c) = ordre.iter().position(|s| *s == colonne_id) else {
                    return;
                };
                let n = if cle == "ArrowRight" {
                    c + 1
                } else {
                    c.wrapping_sub(1)
                };
                let Some(cible_col) = ordre.get(n) else {
                    return;
                };
                if ev.alt_key() {
                    etat.deplacer_vers(id, cible_col);
                    // Le focus suit la carte une fois la liste mise à jour.
                    gloo_timers::callback::Timeout::new(120, move || {
                        focaliser(&format!("[data-carte='{id}']"))
                    })
                    .forget();
                } else if let Some(v) = etat.devoirs_visibles(cible_col).first() {
                    focaliser(&format!("[data-carte='{}']", v.id));
                }
            }
            _ => {}
        }
    };

    view! {
        <article class="w-carte" class:fait=d.statut == "termine" class:glisse=move || glisse.get() == Some(id)
            role="listitem" tabindex="0" draggable="true" data-carte=id.to_string()
            aria-label=aria aria-selected=move || if selectionnee() { "true" } else { "false" }
            on:click=move |_| etat.edition.set(Some(d_ouvrir.clone()))
            on:keydown=au_clavier
            on:contextmenu=move |ev| { ev.prevent_default(); etat.menu_ctx.set(Some((ev.client_x() as f64, ev.client_y() as f64, id))); }
            on:dragstart=move |ev| {
                glisse.set(Some(id));
                if let Some(dt) = ev.data_transfer() {
                    let _ = dt.set_data("text/plain", &id.to_string());
                    dt.set_effect_allowed("move");
                }
            }
            on:dragend=move |_| { glisse.set(None); cible.set(None); }>
            {(d.priorite > 0).then(|| view! {
                <span class="w-priorite" style=format!("background:{}", ["", "var(--accent)", "var(--warning)", "var(--danger)"][d.priorite.min(3) as usize])
                    role="img" aria-label=format!("Priorité {}", ["", "basse", "moyenne", "haute"][d.priorite.min(3) as usize])></span>
            })}
            <p class="w-carte-titre">{d.titre.clone()}</p>
            {(!d.matiere.is_empty() || !d.echeance.is_empty() || d.type_ != "devoir" || !d.description.is_empty()).then(|| view! {
                <div class="w-carte-meta">
                    {(d.type_ != "devoir").then(|| view! { <span class="w-chip w-chip-neutre">{type_libelle(&d.type_).to_string()}</span> })}
                    {(!d.matiere.is_empty()).then(|| view! { <span>{d.matiere.clone()}</span> })}
                    {(!d.description.is_empty()).then(|| view! { <span title="Contient une description" aria-label="Contient une description"><Icone nom="devoir" class="w-3 h-3" /></span> })}
                    {(!d.echeance.is_empty()).then(|| view! {
                        <span class=classe_echeance(&d)>{libelle_echeance(&d.echeance)}</span>
                    })}
                </div>
            })}
            {(!d.tags.is_empty() || total > 0).then(|| view! {
                <div class="w-carte-pied">
                    <div class="w-tags">
                        {d.tags.iter().map(|t| view! { <span class="w-chip" style=format!("--c:{}", t.couleur)>{t.nom.clone()}</span> }).collect_view()}
                    </div>
                    {(total > 0).then(|| view! {
                        <span class="w-indic" title="Sous-tâches terminées">
                            <span class="w-anneau" class:complet=faits == total style=format!("--p:{pourcent}") role="progressbar"
                                aria-valuemin="0" aria-valuemax=total aria-valuenow=faits aria-label="Sous-tâches terminées"></span>
                            {format!("{faits}/{total}")}
                        </span>
                    })}
                </div>
            })}
        </article>
    }
}
