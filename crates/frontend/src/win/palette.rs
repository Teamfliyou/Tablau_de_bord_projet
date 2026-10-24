//! Palette de commandes (⌘K / Ctrl+K) : navigation, actions et recherche de devoirs.

use super::{raccourci, Ui};
use crate::icons::Icone;
use crate::state::{Etat, Onglet};
use leptos::prelude::*;
use std::rc::Rc;

struct Commande {
    libelle: String,
    icone: &'static str,
    raccourci: String,
    action: Rc<dyn Fn()>,
}

fn commande(libelle: &str, icone: &'static str, raccourci: String, action: impl Fn() + 'static) -> Commande {
    Commande { libelle: libelle.to_string(), icone, raccourci, action: Rc::new(action) }
}

#[component]
pub fn Palette() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let ui = expect_context::<Ui>();
    let requete = RwSignal::new(String::new());
    let choix = RwSignal::new(0usize);

    let commandes = move || -> Vec<Commande> {
        let mut v = vec![
            commande("Aller au Dashboard", "dashboard", String::new(), move || etat.onglet.set(Onglet::Dashboard)),
            commande("Aller à la Semaine", "calendar", String::new(), move || etat.onglet.set(Onglet::Semaine)),
            commande("Aller au Kanban", "list", String::new(), move || etat.onglet.set(Onglet::Devoirs)),
            commande("Ouvrir la Configuration", "gear", raccourci(","), move || etat.onglet.set(Onglet::Config)),
            commande("Nouveau devoir", "plus", raccourci("N"), move || { etat.onglet.set(Onglet::Devoirs); ui.ajout_col.set(Some("a_faire")); }),
            commande("Importer des devoirs…", "upload", String::new(), move || etat.import_ouvert.set(true)),
            commande("Exporter les devoirs", "download", String::new(), move || etat.exporter_devoirs()),
            commande("Gérer les tags", "tag", String::new(), move || etat.gestion_tags.set(true)),
            commande("Afficher ou masquer la barre latérale", "sidebar", raccourci("\\"), move || etat.sb_ferme.update(|f| *f = !*f)),
            commande("Basculer clair / sombre", "moon", String::new(), move || etat.basculer_theme()),
            commande("Annuler", "undo", raccourci("Z"), move || etat.annuler()),
            commande("Rétablir", "redo", raccourci("⇧Z"), move || etat.retablir()),
            commande("Rechercher un devoir", "search", raccourci("F"), move || {
                if let Some(c) = ui.recherche_ref.get_untracked() { let _ = c.focus(); }
            }),
        ];
        let q = requete.get().trim().to_lowercase();
        if q.is_empty() {
            return v;
        }
        v.retain(|c| c.libelle.to_lowercase().contains(&q));
        // Devoirs correspondants : ouvre l'inspecteur.
        let devoirs: Vec<Commande> = etat.devoirs.with(|l| {
            l.iter()
                .filter(|d| d.titre.to_lowercase().contains(&q) || d.matiere.to_lowercase().contains(&q))
                .take(8)
                .map(|d| {
                    let d = d.clone();
                    commande(&format!("{}{}", d.titre, if d.matiere.is_empty() { String::new() } else { format!(" — {}", d.matiere) }), "devoir", String::new(), move || {
                        etat.onglet.set(Onglet::Devoirs);
                        etat.edition.set(Some(d.clone()));
                    })
                })
                .collect()
        });
        v.extend(devoirs);
        v
    };

    let lancer = move |i: usize| {
        let action = commandes().into_iter().nth(i).map(|c| c.action);
        etat.palette.set(false);
        if let Some(a) = action {
            a();
        }
    };

    view! {
        <div class="w-voile" on:click=move |_| etat.palette.set(false)>
            <div class="w-palette w-glass" role="dialog" aria-modal="true" aria-label="Palette de commandes" on:click=|ev| ev.stop_propagation()>
                <input type="text" placeholder="Tape une commande ou le nom d'un devoir…" aria-label="Commande" role="combobox"
                    aria-expanded="true" aria-controls="palette-liste" autofocus
                    node_ref={
                        let r = NodeRef::<leptos::html::Input>::new();
                        r.on_load(move |el| { let _ = el.focus(); });
                        r
                    }
                    prop:value=move || requete.get()
                    on:input=move |ev| { requete.set(event_target_value(&ev)); choix.set(0); }
                    on:keydown=move |ev| {
                        let n = commandes().len();
                        match ev.key().as_str() {
                            "ArrowDown" => { ev.prevent_default(); if n > 0 { choix.update(|c| *c = (*c + 1) % n); } }
                            "ArrowUp" => { ev.prevent_default(); if n > 0 { choix.update(|c| *c = (*c + n - 1) % n); } }
                            "Enter" => { ev.prevent_default(); lancer(choix.get_untracked()); }
                            _ => {}
                        }
                    } />
                <ul id="palette-liste" role="listbox">
                    {move || {
                        let liste = commandes();
                        if liste.is_empty() {
                            return view! { <li class="w-vide" style="padding:22px"><p>"Aucun résultat."</p></li> }.into_any();
                        }
                        liste.into_iter().enumerate().map(|(i, c)| view! {
                            <li role="option" aria-selected=move || if choix.get() == i { "true" } else { "false" }>
                                <button type="button" tabindex="-1" aria-selected=move || if choix.get() == i { "true" } else { "false" }
                                    on:mousemove=move |_| if choix.get_untracked() != i { choix.set(i) }
                                    on:click=move |_| lancer(i)>
                                    <Icone nom=c.icone />
                                    {c.libelle}
                                    {(!c.raccourci.is_empty()).then(|| view! { <span class="w-raccourci">{c.raccourci.clone()}</span> })}
                                </button>
                            </li>
                        }).collect_view().into_any()
                    }}
                </ul>
            </div>
        </div>
    }
}
