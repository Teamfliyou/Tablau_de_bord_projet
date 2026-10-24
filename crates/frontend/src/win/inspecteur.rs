//! Inspecteur de devoir (panneau de droite) : chaque modification est enregistrée immédiatement.

use super::tags::{CouleursTag, COULEURS_TAG};
use crate::api;
use crate::icons::Icone;
use crate::state::{Etat, COLONNES, TYPES_DEVOIR};
use leptos::prelude::*;
use leptos::task::spawn_local;
use tdb_shared::{DevoirModification, TagEntree, TodoEntree, TODOS_PAR_DEVOIR_MAX};

#[component]
pub fn Inspecteur(id: i64) -> impl IntoView {
    let etat = expect_context::<Etat>();
    let Some(devoir) = etat
        .devoir_par_id(id)
        .or_else(|| etat.edition.get_untracked())
    else {
        return ().into_any();
    };
    let titre = RwSignal::new(devoir.titre.clone());
    let matiere = RwSignal::new(devoir.matiere.clone());
    let echeance = RwSignal::new(devoir.echeance.clone());
    let type_ = RwSignal::new(devoir.type_.clone());
    let statut = RwSignal::new(devoir.statut.clone());
    let description = RwSignal::new(devoir.description.clone());
    let priorite = RwSignal::new(devoir.priorite);
    let erreur = RwSignal::new(String::new());
    let tags_choisis = RwSignal::new(devoir.tags.iter().map(|t| t.id).collect::<Vec<i64>>());
    let todos = RwSignal::new(
        devoir
            .todos
            .iter()
            .map(|t| (t.texte.clone(), t.fait))
            .collect::<Vec<(String, bool)>>(),
    );
    let nouveau_todo = RwSignal::new(String::new());
    let nouveau_tag = RwSignal::new(String::new());
    let couleur_tag = RwSignal::new(COULEURS_TAG[5].to_string());
    let creation_tag = RwSignal::new(false);

    // Enregistre les champs simples ; le titre vide est refusé.
    let sauver_champs = move || {
        let t = titre.get_untracked().trim().to_string();
        if t.is_empty() {
            erreur.set("Le titre ne peut pas être vide.".into());
            return;
        }
        let m = DevoirModification {
            titre: Some(t),
            matiere: Some(matiere.get_untracked()),
            echeance: Some(echeance.get_untracked()),
            type_: Some(type_.get_untracked()),
            statut: Some(statut.get_untracked()),
            description: Some(description.get_untracked()),
            priorite: Some(priorite.get_untracked()),
        };
        spawn_local(async move {
            match api::modifier_devoir(id, &m).await {
                Ok(_) => {
                    erreur.set(String::new());
                    etat.charger_devoirs_async().await;
                }
                Err(e) => erreur.set(etat.message(e)),
            }
        });
    };
    let changer_statut = move |nouveau: String| {
        statut.set(nouveau.clone());
        if let Some(s) = COLONNES
            .iter()
            .find(|(c, _, _)| *c == nouveau)
            .map(|(c, _, _)| *c)
        {
            etat.deplacer_vers(id, s);
        }
    };

    let appliquer_tags = move |ids: Vec<i64>| {
        spawn_local(async move {
            match api::definir_tags_devoir(id, &ids).await {
                Ok(_) => {
                    tags_choisis.set(ids);
                    erreur.set(String::new());
                    etat.charger_devoirs_async().await;
                }
                Err(e) => erreur.set(etat.message(e)),
            }
        });
    };
    let basculer_tag = move |tag_id: i64| {
        let mut ids = tags_choisis.get_untracked();
        match ids.iter().position(|x| *x == tag_id) {
            Some(i) => {
                ids.remove(i);
            }
            None => ids.push(tag_id),
        }
        appliquer_tags(ids);
    };
    let creer_tag = move || {
        let nom = nouveau_tag.get_untracked().trim().to_string();
        if nom.is_empty() {
            return;
        }
        let entree = TagEntree {
            nom,
            couleur: couleur_tag.get_untracked(),
        };
        spawn_local(async move {
            match api::creer_tag(&entree).await {
                Ok(t) => {
                    nouveau_tag.set(String::new());
                    creation_tag.set(false);
                    let mut ids = tags_choisis.get_untracked();
                    ids.push(t.id);
                    match api::definir_tags_devoir(id, &ids).await {
                        Ok(_) => tags_choisis.set(ids),
                        Err(e) => erreur.set(etat.message(e)),
                    }
                    etat.charger_devoirs_async().await;
                }
                Err(e) => erreur.set(etat.message(e)),
            }
        });
    };

    let sauver_todos = move |liste: Vec<(String, bool)>| {
        let entrees = liste
            .iter()
            .map(|(texte, fait)| TodoEntree {
                texte: texte.clone(),
                fait: *fait,
            })
            .collect::<Vec<_>>();
        // Mise à jour optimiste : la case se coche immédiatement, le serveur confirme ensuite.
        todos.set(liste);
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
    let progression = move || {
        let l = todos.get();
        (l.iter().filter(|t| t.1).count(), l.len())
    };

    view! {
        <aside class="w-inspecteur" aria-label="Détail du devoir">
            <div class="w-insp-tete">
                <h2>"Détail"</h2>
                <button type="button" class="w-btn w-btn-icone" aria-label="Fermer l'inspecteur" title="Fermer (Échap)" on:click=move |_| etat.edition.set(None)>
                    <Icone nom="x" />
                </button>
            </div>
            <div class="w-insp-corps">
                <textarea class="w-insp-titre" rows="2" maxlength="200" aria-label="Titre" placeholder="Titre"
                    prop:value=move || titre.get()
                    on:input=move |ev| titre.set(event_target_value(&ev))
                    on:keydown=move |ev| if ev.key() == "Enter" { ev.prevent_default(); sauver_champs(); }
                    on:blur=move |_| sauver_champs()></textarea>

                <div class="w-grille">
                    <div>
                        <label class="w-label" for="insp-statut">"Statut"</label>
                        <select id="insp-statut" class="w-select" prop:value=move || statut.get() on:change=move |ev| changer_statut(event_target_value(&ev))>
                            {COLONNES.iter().map(|(c, l, _)| view! { <option value=*c>{*l}</option> }).collect_view()}
                        </select>
                    </div>
                    <div>
                        <label class="w-label" for="insp-type">"Type"</label>
                        <select id="insp-type" class="w-select" prop:value=move || type_.get()
                            on:change=move |ev| { type_.set(event_target_value(&ev)); sauver_champs(); }>
                            {TYPES_DEVOIR.iter().map(|(c, l)| view! { <option value=*c>{*l}</option> }).collect_view()}
                        </select>
                    </div>
                    <div>
                        <label class="w-label" for="insp-matiere">"Matière"</label>
                        <input id="insp-matiere" class="w-champ" type="text" maxlength="120" prop:value=move || matiere.get()
                            on:input=move |ev| matiere.set(event_target_value(&ev)) on:blur=move |_| sauver_champs()
                            on:keydown=move |ev| if ev.key() == "Enter" { sauver_champs(); } />
                    </div>
                    <div>
                        <label class="w-label" for="insp-echeance">"Échéance"</label>
                        <input id="insp-echeance" class="w-champ" type="date" prop:value=move || echeance.get()
                            on:change=move |ev| { echeance.set(event_target_value(&ev)); sauver_champs(); } />
                    </div>
                </div>

                <div>
                    <span class="w-label" id="insp-prio">"Priorité"</span>
                    <div class="w-segment" role="group" aria-labelledby="insp-prio">
                        {[(0u8, "Aucune"), (1, "Basse"), (2, "Moyenne"), (3, "Haute")].into_iter().map(|(p, l)| view! {
                            <button type="button" aria-pressed=move || if priorite.get() == p { "true" } else { "false" }
                                on:click=move |_| { priorite.set(p); sauver_champs(); }>{l}</button>
                        }).collect_view()}
                    </div>
                </div>

                <div>
                    <label class="w-label" for="insp-desc">"Description"</label>
                    <textarea id="insp-desc" class="w-textarea" rows="3" maxlength="2000" placeholder="Notes, consignes, liens…"
                        prop:value=move || description.get()
                        on:input=move |ev| description.set(event_target_value(&ev))
                        on:blur=move |_| sauver_champs()></textarea>
                </div>

                <div>
                    <span class="w-label">"Tags"</span>
                    <div class="w-tags" role="group" aria-label="Tags du devoir">
                        {move || etat.tags.get().into_iter().map(|t| {
                            let tid = t.id;
                            view! {
                                <button type="button" class="w-chip" style=format!("--c:{}", t.couleur)
                                    aria-pressed=move || if tags_choisis.get().contains(&tid) { "true" } else { "false" }
                                    on:click=move |_| basculer_tag(tid)>{t.nom}</button>
                            }
                        }).collect_view()}
                        <button type="button" class="w-chip w-chip-neutre" on:click=move |_| creation_tag.update(|c| *c = !*c)>
                            <Icone nom="plus" class="w-3 h-3" />
                            "Nouveau tag"
                        </button>
                    </div>
                    {move || creation_tag.get().then(|| view! {
                        <div style="margin-top:8px;display:flex;flex-direction:column;gap:8px">
                            <input class="w-champ" type="text" maxlength="30" placeholder="Nom du tag" aria-label="Nom du nouveau tag"
                                prop:value=move || nouveau_tag.get() on:input=move |ev| nouveau_tag.set(event_target_value(&ev))
                                on:keydown=move |ev| if ev.key() == "Enter" { ev.prevent_default(); creer_tag(); } />
                            <CouleursTag valeur=couleur_tag />
                            <button type="button" class="w-btn w-btn-primaire" style="align-self:flex-start" on:click=move |_| creer_tag()>"Créer et ajouter"</button>
                        </div>
                    })}
                </div>

                <div>
                    <div style="display:flex;justify-content:space-between;align-items:baseline">
                        <span class="w-label">"Sous-tâches"</span>
                        <span class="w-indic">{move || { let (f, t) = progression(); format!("{f}/{t}") }}</span>
                    </div>
                    {move || { let (f, t) = progression(); (t > 0).then(|| view! {
                        <div class="w-progres" class:complet=f == t role="progressbar" aria-valuemin="0" aria-valuemax=t aria-valuenow=f aria-label="Progression des sous-tâches" style="margin:4px 0 8px">
                            <i style=format!("width:{}%", f * 100 / t)></i>
                        </div>
                    }) }}
                    <ul style="list-style:none;margin:0;padding:0">
                        {move || {
                            let n = todos.get_untracked().len();
                            todos.get().into_iter().enumerate().map(|(i, (texte, fait))| view! {
                                <li class="w-todo" class:fait=fait>
                                    <input type="checkbox" class="w-case" prop:checked=fait aria-label=format!("Terminer : {texte}") on:change=move |_| cocher_todo(i) />
                                    <input type="text" maxlength="200" aria-label="Texte de la sous-tâche" prop:value=texte.clone()
                                        on:keydown=move |ev| if ev.key() == "Enter" { ev.prevent_default(); modifier_todo(i, event_target_value(&ev)); }
                                        on:change=move |ev| modifier_todo(i, event_target_value(&ev)) />
                                    <span class="w-actions">
                                        <button type="button" class="w-btn w-btn-icone" style="width:22px;min-height:22px" aria-label="Monter" prop:disabled=i == 0 on:click=move |_| deplacer_todo(i, -1)><Icone nom="chevron-up" /></button>
                                        <button type="button" class="w-btn w-btn-icone" style="width:22px;min-height:22px" aria-label="Descendre" prop:disabled=i + 1 == n on:click=move |_| deplacer_todo(i, 1)><Icone nom="chevron-down" /></button>
                                        <button type="button" class="w-btn w-btn-icone" style="width:22px;min-height:22px" aria-label="Supprimer la sous-tâche" on:click=move |_| supprimer_todo(i)><Icone nom="trash" /></button>
                                    </span>
                                </li>
                            }).collect_view()
                        }}
                    </ul>
                    <div style="margin-top:6px;display:flex;gap:6px">
                        <input class="w-champ" type="text" maxlength="200" placeholder="Ajouter une sous-tâche" aria-label="Nouvelle sous-tâche"
                            prop:value=move || nouveau_todo.get() on:input=move |ev| nouveau_todo.set(event_target_value(&ev))
                            on:keydown=move |ev| if ev.key() == "Enter" { ev.prevent_default(); ajouter_todo(); } />
                        <button type="button" class="w-btn" aria-label="Ajouter" on:click=move |_| ajouter_todo()><Icone nom="plus" /></button>
                    </div>
                </div>

                {move || { let m = erreur.get(); (!m.is_empty()).then(|| view! {
                    <p role="alert" style="margin:0;padding:8px 10px;border-radius:var(--r-sm);background:color-mix(in srgb,var(--danger) 12%,transparent);color:var(--danger);font-size:12px">{m}</p>
                }) }}
            </div>
            <div class="w-insp-pied">
                <button type="button" class="w-btn w-btn-danger" on:click=move |_| etat.supprimer_annulable(id)>
                    <Icone nom="trash" />
                    "Supprimer"
                </button>
                <button type="button" class="w-btn" on:click=move |_| etat.edition.set(None)>"Fermer"</button>
            </div>
        </aside>
    }.into_any()
}
