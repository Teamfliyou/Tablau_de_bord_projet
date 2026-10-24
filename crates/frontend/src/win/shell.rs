//! Coquille : barre latérale translucide, barre d'outils unifiée, contenu, inspecteur, raccourcis.

use super::{dans_saisie, import::FeuilleImport, inspecteur::Inspecteur, kanban::Tableau, menu::MenuContextuel, palette::Palette, raccourci, tags::GestionTags, Ui};
use crate::app::Onboarding;
use crate::icons::Icone;
use crate::pages::config::Configuration;
use crate::pages::dashboard::Dashboard;
use crate::pages::semaine::Semaine;
use crate::state::{Etat, Onglet};
use crate::util;
use leptos::ev;
use leptos::prelude::*;

#[component]
pub fn ApplicationWin() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let ui = Ui {
        ajout_col: RwSignal::new(None),
        recherche_ref: NodeRef::new(),
    };
    provide_context(ui);

    // Sur petit écran, la barre latérale démarre repliée.
    if util::fenetre()
        .and_then(|w| w.inner_width().ok())
        .and_then(|v| v.as_f64())
        .is_some_and(|l| l < 900.0)
    {
        etat.sb_ferme.set(true);
    }

    // Raccourcis clavier (Ctrl sous Windows, ⌘ sous Mac).
    let _ = window_event_listener(ev::keydown, move |ev| {
        let cmd = ev.meta_key() || ev.ctrl_key();
        let touche = ev.key().to_lowercase();
        let saisie = dans_saisie(&ev);
        if cmd {
            match touche.as_str() {
                "k" => {
                    ev.prevent_default();
                    etat.palette.update(|p| *p = !*p);
                }
                "n" => {
                    ev.prevent_default();
                    etat.onglet.set(Onglet::Devoirs);
                    ui.ajout_col.set(Some("a_faire"));
                }
                "f" => {
                    ev.prevent_default();
                    if let Some(champ) = ui.recherche_ref.get() {
                        let _ = champ.focus();
                        champ.select();
                    }
                }
                "\\" => {
                    ev.prevent_default();
                    etat.sb_ferme.update(|f| *f = !*f);
                }
                "," => {
                    ev.prevent_default();
                    etat.onglet.set(Onglet::Config);
                }
                "z" if !saisie => {
                    ev.prevent_default();
                    if ev.shift_key() {
                        etat.retablir();
                    } else {
                        etat.annuler();
                    }
                }
                "y" if !saisie => {
                    ev.prevent_default();
                    etat.retablir();
                }
                _ => {}
            }
            return;
        }
        if touche == "escape" {
            if etat.palette.get_untracked() {
                etat.palette.set(false);
            } else if etat.menu_ctx.get_untracked().is_some() {
                etat.menu_ctx.set(None);
            } else if etat.import_ouvert.get_untracked() {
                etat.import_ouvert.set(false);
            } else if etat.gestion_tags.get_untracked() {
                etat.gestion_tags.set(false);
            } else if ui.ajout_col.get_untracked().is_some() {
                ui.ajout_col.set(None);
            } else if etat.edition.get_untracked().is_some() && !saisie {
                etat.edition.set(None);
            }
        }
    });

    let identifiant = move || {
        let u = etat.config.get().username;
        if u.is_empty() { "Étudiant".to_string() } else { u }
    };
    let date_jour = util::date_longue(&js_sys::Date::new_0());
    let zone_kanban = move || etat.onglet.get() == Onglet::Devoirs;
    let titre_onglet = move || etat.onglet.get().libelle();
    let sombre = move || etat.theme_sombre();
    let nb_tag = move |id: i64| etat.devoirs.with(|l| l.iter().filter(|d| d.tags.iter().any(|t| t.id == id)).count());
    let aller = move |o: Onglet| {
        etat.onglet.set(o);
        if util::fenetre()
            .and_then(|w| w.inner_width().ok())
            .and_then(|v| v.as_f64())
            .is_some_and(|l| l < 900.0)
        {
            etat.sb_ferme.set(true);
        }
    };

    view! {
        <div class="w-app" class:sb-ferme=move || etat.sb_ferme.get()>
            <aside class="w-sidebar" aria-label="Barre latérale" inert=move || etat.sb_ferme.get()>
                <div class="w-brand">
                    <span class="w-brand-icone"><Icone nom="mortarboard" class="w-4 h-4" /></span>
                    <div>
                        <b>"Tableau de bord"</b>
                        <small>{identifiant}</small>
                    </div>
                </div>
                <div class="w-section">"Vues"</div>
                <nav class="w-nav" aria-label="Vues">
                    {[Onglet::Dashboard, Onglet::Semaine, Onglet::Devoirs].into_iter().map(|o| view! {
                        <button type="button" class="w-nav-item"
                            aria-current=move || if etat.onglet.get() == o { "page" } else { "false" }
                            on:click=move |_| aller(o)>
                            <Icone nom=o.icone() />
                            {o.libelle()}
                            {(o == Onglet::Devoirs).then(|| view! {
                                <span class="w-count">{move || etat.devoirs.with(|l| l.iter().filter(|d| d.statut != "termine").count())}</span>
                            })}
                        </button>
                    }).collect_view()}
                </nav>
                <div class="w-section">
                    "Tags"
                    <button type="button" class="w-btn w-btn-icone" style="width:22px;min-height:22px" title="Gérer les tags" aria-label="Gérer les tags"
                        on:click=move |_| etat.gestion_tags.set(true)>
                        <Icone nom="plus" />
                    </button>
                </div>
                <nav class="w-nav" aria-label="Filtrer par tag">
                    {move || {
                        let liste = etat.tags.get();
                        if liste.is_empty() {
                            return view! { <p style="margin:2px 9px;font-size:12px;color:var(--label-2)">"Aucun tag pour l'instant."</p> }.into_any();
                        }
                        liste.into_iter().map(|t| {
                            let id = t.id;
                            view! {
                                <button type="button" class="w-nav-item"
                                    aria-pressed=move || if etat.filtre_tag.get() == Some(id) { "true" } else { "false" }
                                    on:click=move |_| {
                                        etat.filtre_tag.set(if etat.filtre_tag.get_untracked() == Some(id) { None } else { Some(id) });
                                        etat.onglet.set(Onglet::Devoirs);
                                    }>
                                    <span class="w-tag-dot" style=format!("background:{}", t.couleur)></span>
                                    {t.nom}
                                    <span class="w-count">{move || nb_tag(id)}</span>
                                </button>
                            }
                        }).collect_view().into_any()
                    }}
                </nav>
                <div class="w-sidebar-pied">
                    <p style="margin:0 9px 6px;font-size:11px;color:var(--label-2)">{date_jour}</p>
                    <button type="button" class="w-nav-item"
                        aria-current=move || if etat.onglet.get() == Onglet::Config { "page" } else { "false" }
                        on:click=move |_| aller(Onglet::Config)>
                        <Icone nom="gear" />
                        "Configuration"
                    </button>
                </div>
            </aside>

            <div class="w-main">
                <header class="w-toolbar">
                    <button type="button" class="w-btn w-btn-icone" title=move || format!("Barre latérale ({})", raccourci("\\")) aria-label="Afficher ou masquer la barre latérale"
                        aria-expanded=move || if etat.sb_ferme.get() { "false" } else { "true" }
                        on:click=move |_| etat.sb_ferme.update(|f| *f = !*f)>
                        <Icone nom="sidebar" />
                    </button>
                    <h1 class="w-titre">{titre_onglet}</h1>
                    <span class="w-sous-titre">
                        {move || match etat.onglet.get() {
                            Onglet::Devoirs => {
                                let n = etat.devoirs.with(|l| l.len());
                                format!("{n} élément{}", if n > 1 { "s" } else { "" })
                            }
                            _ => String::new(),
                        }}
                    </span>
                    <span class="w-spacer"></span>
                    {move || zone_kanban().then(|| view! {
                        <div style="display:flex;gap:2px">
                            <button type="button" class="w-btn w-btn-icone" title=format!("Annuler ({})", raccourci("Z")) aria-label="Annuler"
                                prop:disabled=move || etat.annulables.with(|p| p.is_empty()) on:click=move |_| etat.annuler()>
                                <Icone nom="undo" />
                            </button>
                            <button type="button" class="w-btn w-btn-icone" title=format!("Rétablir ({})", raccourci("⇧Z")) aria-label="Rétablir"
                                prop:disabled=move || etat.refaisables.with(|p| p.is_empty()) on:click=move |_| etat.retablir()>
                                <Icone nom="redo" />
                            </button>
                        </div>
                    })}
                    <div class="w-recherche" role="search">
                        <Icone nom="search" />
                        <input class="w-champ" type="search" node_ref=ui.recherche_ref placeholder="Rechercher" aria-label="Rechercher un devoir"
                            prop:value=move || etat.recherche.get()
                            on:input=move |ev| {
                                etat.recherche.set(event_target_value(&ev));
                                etat.onglet.set(Onglet::Devoirs);
                            }
                            on:keydown=move |ev| if ev.key() == "Escape" { etat.recherche.set(String::new()); }
                        />
                        <kbd>{raccourci("F")}</kbd>
                    </div>
                    <button type="button" class="w-btn" title=move || format!("Importer des devoirs") on:click=move |_| etat.import_ouvert.set(true)>
                        <Icone nom="upload" />
                        <span class="w-cache-mobile">"Importer"</span>
                    </button>
                    <button type="button" class="w-btn w-btn-primaire" title=format!("Nouveau devoir ({})", raccourci("N"))
                        on:click=move |_| { etat.onglet.set(Onglet::Devoirs); ui.ajout_col.set(Some("a_faire")); }>
                        <Icone nom="plus" />
                        "Nouveau"
                    </button>
                    <button type="button" class="w-btn w-btn-icone" aria-label="Palette de commandes" title=format!("Commandes ({})", raccourci("K"))
                        on:click=move |_| etat.palette.set(true)>
                        <Icone nom="command" />
                    </button>
                    <button type="button" class="w-btn w-btn-icone" aria-label=move || if sombre() { "Passer en mode clair" } else { "Passer en mode sombre" }
                        title=move || if sombre() { "Mode clair" } else { "Mode sombre" } on:click=move |_| etat.basculer_theme()>
                        {move || view! { <Icone nom=if sombre() { "sun" } else { "moon" } /> }}
                    </button>
                </header>

                <div class="w-body">
                    <main class=move || if zone_kanban() { "w-content w-kanban-zone" } else { "w-content" } id="contenu">
                        {move || match etat.onglet.get() {
                            Onglet::Dashboard => view! { <Dashboard /> }.into_any(),
                            Onglet::Semaine => view! { <Semaine /> }.into_any(),
                            Onglet::Devoirs => view! { <Tableau /> }.into_any(),
                            Onglet::Config => view! { <Configuration /> }.into_any(),
                        }}
                    </main>
                    {
                        let id_ouvert = Memo::new(move |_| etat.edition.get().map(|d| d.id));
                        move || id_ouvert.get().map(|id| view! { <Inspecteur id=id /> })
                    }
                </div>
            </div>

            <div class="w-scrim" on:click=move |_| etat.sb_ferme.set(true)></div>

            <nav class="w-tabbar w-glass" aria-label="Navigation principale">
                {[Onglet::Dashboard, Onglet::Semaine, Onglet::Devoirs, Onglet::Config].into_iter().map(|o| view! {
                    <button type="button" aria-current=move || if etat.onglet.get() == o { "page" } else { "false" } on:click=move |_| etat.onglet.set(o)>
                        <Icone nom=o.icone() />
                        {o.libelle()}
                    </button>
                }).collect_view()}
            </nav>

            {move || etat.onboarding_visible.get().then(|| view! { <Onboarding /> })}
            {move || etat.maj_fenetre.get().then(|| view! { <crate::maj::FenetreMiseAJour /> })}
            {move || etat.gestion_tags.get().then(|| view! { <GestionTags /> })}
            {move || etat.import_ouvert.get().then(|| view! { <FeuilleImport /> })}
            {move || etat.palette.get().then(|| view! { <Palette /> })}
            {move || etat.menu_ctx.get().map(|(x, y, id)| view! { <MenuContextuel x=x y=y id=id /> })}
            {move || {
                let t = etat.toast.get();
                (!t.is_empty()).then(|| {
                    let annulable = t.ends_with("annuler.");
                    view! {
                        <div class="w-toast w-glass" role="status" aria-live="polite">
                            <span>{t}</span>
                            {annulable.then(|| view! {
                                <button type="button" class="w-btn" on:click=move |_| { etat.annuler(); etat.toast.set(String::new()); }>"Annuler"</button>
                            })}
                        </div>
                    }
                })
            }}
        </div>
    }
}
