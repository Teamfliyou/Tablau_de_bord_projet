//! État global de l'interface (signaux Leptos) et actions asynchrones associées.

use crate::api::{self, ErreurApi, Mode};
use crate::util;
use gloo_timers::callback::Timeout;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::{json, Value};
use tdb_shared::{
    Categories, Config, ConfigUpdate, Cours, Devoir, DevoirEntree, DevoirModification, Filtres,
    InfoMiseAJour, Plan, Tag, VerificationMiseAJour,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Onglet {
    Dashboard,
    Semaine,
    Devoirs,
    Config,
}

impl Onglet {
    pub const TOUS: [Onglet; 4] = [
        Onglet::Dashboard,
        Onglet::Semaine,
        Onglet::Devoirs,
        Onglet::Config,
    ];
    pub fn libelle(self) -> &'static str {
        match self {
            Onglet::Dashboard => "Dashboard",
            Onglet::Semaine => "Semaine",
            Onglet::Devoirs => "Kanban",
            Onglet::Config => "Configuration",
        }
    }
    pub fn icone(self) -> &'static str {
        match self {
            Onglet::Dashboard => "dashboard",
            Onglet::Semaine => "calendar",
            Onglet::Devoirs => "list",
            Onglet::Config => "gear",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StyleNav {
    Dock,
    Classic,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Auto,
    Light,
    Dark,
}

impl Theme {
    pub fn code(self) -> &'static str {
        match self {
            Theme::Auto => "auto",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
    pub fn depuis_code(c: &str) -> Option<Theme> {
        match c {
            "auto" => Some(Theme::Auto),
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Dimension {
    Groupes,
    Types,
    Matieres,
}

#[derive(Clone, Copy)]
pub struct Etat {
    pub mode: Mode,
    pub authentifie: RwSignal<bool>,
    pub onglet: RwSignal<Onglet>,
    pub style_nav: RwSignal<StyleNav>,
    pub theme: RwSignal<Theme>,
    pub heure: RwSignal<String>,
    pub toast: RwSignal<String>,
    pub version: RwSignal<String>,
    // Mise à jour (Windows)
    pub maj_info: RwSignal<Option<InfoMiseAJour>>,
    pub maj_fenetre: RwSignal<bool>,
    pub maj_verification: RwSignal<bool>,
    pub maj_message: RwSignal<String>,
    pub maj_erreur: RwSignal<String>,
    /// Résultat de la dernière recherche (dernière version publiée, date), même sans mise à jour.
    pub maj_derniere: RwSignal<Option<VerificationMiseAJour>>,
    /// Échec de la recherche automatique au démarrage (affiché seulement dans la Configuration).
    pub maj_echec_auto: RwSignal<String>,
    // Emploi du temps
    pub cours: RwSignal<Vec<Cours>>,
    pub chargement_cours: RwSignal<bool>,
    pub erreur_cours: RwSignal<Option<String>>,
    pub cours_semaine: RwSignal<Vec<Cours>>,
    pub chargement_semaine: RwSignal<bool>,
    pub erreur_semaine: RwSignal<Option<String>>,
    pub semaine_date: RwSignal<String>,
    pub semaine_info: RwSignal<(String, String)>,
    pub hors_ligne: RwSignal<bool>,
    pub maj_flux: RwSignal<String>,
    // Devoirs
    pub devoirs: RwSignal<Vec<Devoir>>,
    pub erreur_devoirs: RwSignal<Option<String>>,
    pub edition: RwSignal<Option<Devoir>>,
    // Tags
    pub tags: RwSignal<Vec<Tag>>,
    pub filtre_tag: RwSignal<Option<i64>>,
    pub gestion_tags: RwSignal<bool>,
    // Plan IA
    pub plan: RwSignal<Option<Plan>>,
    pub chargement_plan: RwSignal<bool>,
    pub erreur_plan: RwSignal<Option<String>>,
    // Configuration
    pub config: RwSignal<Config>,
    pub categories: RwSignal<Categories>,
    pub filtres_groupes: RwSignal<Vec<String>>,
    pub filtres_types: RwSignal<Vec<String>>,
    pub filtres_matieres: RwSignal<Vec<String>>,
    pub onboarding_visible: RwSignal<bool>,
    pub onboarding_ignore: RwSignal<bool>,
    // Interface Windows (coquille type app Mac)
    pub recherche: RwSignal<String>,
    pub palette: RwSignal<bool>,
    pub sb_ferme: RwSignal<bool>,
    pub import_ouvert: RwSignal<bool>,
    pub menu_ctx: RwSignal<Option<(f64, f64, i64)>>,
    pub annulables: RwSignal<Vec<crate::state_win::Annul>>,
    pub refaisables: RwSignal<Vec<crate::state_win::Annul>>,
}

impl Etat {
    pub fn nouveau(mode: Mode) -> Etat {
        Etat {
            mode,
            authentifie: RwSignal::new(mode == Mode::Tauri || api::jeton().is_some()),
            onglet: RwSignal::new(Onglet::Dashboard),
            style_nav: RwSignal::new(StyleNav::Dock),
            theme: RwSignal::new(Theme::Auto),
            heure: RwSignal::new(util::heure_actuelle()),
            toast: RwSignal::new(String::new()),
            version: RwSignal::new(String::new()),
            maj_info: RwSignal::new(None),
            maj_fenetre: RwSignal::new(false),
            maj_verification: RwSignal::new(false),
            maj_message: RwSignal::new(String::new()),
            maj_erreur: RwSignal::new(String::new()),
            maj_derniere: RwSignal::new(None),
            maj_echec_auto: RwSignal::new(String::new()),
            cours: RwSignal::new(vec![]),
            chargement_cours: RwSignal::new(false),
            erreur_cours: RwSignal::new(None),
            cours_semaine: RwSignal::new(vec![]),
            chargement_semaine: RwSignal::new(false),
            erreur_semaine: RwSignal::new(None),
            semaine_date: RwSignal::new(util::aujourdhui_iso()),
            semaine_info: RwSignal::new((String::new(), String::new())),
            hors_ligne: RwSignal::new(false),
            maj_flux: RwSignal::new(String::new()),
            devoirs: RwSignal::new(vec![]),
            erreur_devoirs: RwSignal::new(None),
            tags: RwSignal::new(vec![]),
            filtre_tag: RwSignal::new(None),
            gestion_tags: RwSignal::new(false),
            edition: RwSignal::new(None),
            plan: RwSignal::new(None),
            chargement_plan: RwSignal::new(false),
            erreur_plan: RwSignal::new(None),
            config: RwSignal::new(Config::default()),
            categories: RwSignal::new(Categories::default()),
            filtres_groupes: RwSignal::new(vec![]),
            filtres_types: RwSignal::new(vec![]),
            filtres_matieres: RwSignal::new(vec![]),
            onboarding_visible: RwSignal::new(false),
            onboarding_ignore: RwSignal::new(false),
            recherche: RwSignal::new(String::new()),
            palette: RwSignal::new(false),
            sb_ferme: RwSignal::new(false),
            import_ouvert: RwSignal::new(false),
            menu_ctx: RwSignal::new(None),
            annulables: RwSignal::new(vec![]),
            refaisables: RwSignal::new(vec![]),
        }
    }

    pub fn theme_sombre(self) -> bool {
        match self.theme.get() {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::Auto => util::prefere_sombre(),
        }
    }

    pub fn appliquer_theme(self) {
        let sombre = match self.theme.get_untracked() {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::Auto => util::prefere_sombre(),
        };
        util::appliquer_classe_sombre(sombre);
    }

    pub fn notifier(self, message: impl Into<String>) {
        self.toast.set(message.into());
        let t = Timeout::new(4000, move || self.toast.set(String::new()));
        t.forget();
    }

    /// Message affiché pour une erreur ; une session expirée renvoie à l'écran de connexion.
    pub fn message(self, e: ErreurApi) -> String {
        if e.non_auth && self.mode == Mode::Web {
            api::oublier_jeton();
            self.authentifie.set(false);
        }
        e.message
    }

    pub fn filtres(self) -> Filtres {
        Filtres {
            groupes: self.filtres_groupes.get_untracked(),
            types: self.filtres_types.get_untracked(),
            matieres: self.filtres_matieres.get_untracked(),
        }
    }

    pub fn signal_filtre(self, dim: Dimension) -> RwSignal<Vec<String>> {
        match dim {
            Dimension::Groupes => self.filtres_groupes,
            Dimension::Types => self.filtres_types,
            Dimension::Matieres => self.filtres_matieres,
        }
    }

    // ------------------------------------------------------------- Démarrage

    /// Charge préférences puis données (après connexion ou au lancement de l'application Windows).
    pub fn demarrer(self) {
        spawn_local(async move {
            self.charger_preferences().await;
            self.initialiser_donnees();
            if let Some(v) = api::version().await {
                self.version.set(v);
            }
        });
        if self.mode == Mode::Tauri {
            // Vérification discrète, après l'affichage de l'interface : hors ligne, l'application continue normalement.
            spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(4_000).await;
                self.verifier_maj(false).await;
            });
            // Hermes Agent (MCP) peut modifier les devoirs pendant que l'application est ouverte :
            // ils sont relus quand la fenêtre reprend le focus.
            let _ = window_event_listener(leptos::ev::focus, move |_| self.charger_devoirs());
        }
    }

    /// Recherche une mise à jour. Automatique : silencieuse en cas d'échec (hors ligne). Manuelle : affiche le résultat.
    pub async fn verifier_maj(self, manuel: bool) {
        if self.maj_verification.get_untracked() {
            return;
        }
        self.maj_verification.set(true);
        self.maj_message.set(String::new());
        self.maj_erreur.set(String::new());
        match api::verifier_mise_a_jour().await {
            Ok(verification) => {
                self.maj_echec_auto.set(String::new());
                let mise_a_jour = verification.mise_a_jour.clone();
                self.maj_derniere.set(Some(verification));
                match mise_a_jour {
                    Some(info) => {
                        self.maj_info.set(Some(info));
                        self.maj_fenetre.set(true);
                    }
                    None => {
                        self.maj_info.set(None);
                        if manuel {
                            self.maj_message
                                .set("Tu utilises déjà la dernière version.".into());
                        }
                    }
                }
            }
            Err(e) if manuel => self.maj_erreur.set(e.message),
            Err(e) => self.maj_echec_auto.set(e.message),
        }
        self.maj_verification.set(false);
    }

    pub fn initialiser_donnees(self) {
        spawn_local(async move {
            self.charger_config().await;
            if !self.onboarding_ignore.get_untracked()
                && self.config.get_untracked().ade_url.is_empty()
            {
                self.onboarding_visible.set(true);
            }
        });
        self.charger_cours(false);
        self.charger_semaine();
        self.charger_devoirs();
    }

    // ------------------------------------------------------------ Préférences

    pub async fn charger_preferences(self) {
        if let Ok(p) = api::obtenir_preferences().await {
            match p.get("interface").and_then(Value::as_str) {
                Some("dock") => self.style_nav.set(StyleNav::Dock),
                Some("classic") => self.style_nav.set(StyleNav::Classic),
                _ => {}
            }
            if let Some(t) = p
                .get("theme")
                .and_then(Value::as_str)
                .and_then(Theme::depuis_code)
            {
                self.theme.set(t);
            }
            let liste = |cle: &str| -> Vec<String> {
                p.get("filtres")
                    .and_then(|f| f.get(cle))
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .filter(|s| !s.is_empty())
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default()
            };
            self.filtres_groupes.set(liste("groupes"));
            self.filtres_types.set(liste("types"));
            self.filtres_matieres.set(liste("matieres"));
        }
        self.appliquer_theme();
    }

    pub fn enregistrer_preferences(self) {
        self.appliquer_theme();
        let f = self.filtres();
        let prefs = json!({
            "interface": match self.style_nav.get_untracked() {
                StyleNav::Dock => "dock",
                StyleNav::Classic => "classic",
            },
            "theme": self.theme.get_untracked().code(),
            "filtres": { "groupes": f.groupes, "types": f.types, "matieres": f.matieres },
        });
        spawn_local(async move {
            let _ = api::enregistrer_preferences(&prefs).await;
        });
    }

    pub fn basculer_theme(self) {
        self.theme.set(if self.theme_sombre() {
            Theme::Light
        } else {
            Theme::Dark
        });
        self.enregistrer_preferences();
    }

    // ---------------------------------------------------------- Emploi du temps

    pub fn charger_cours(self, silencieux: bool) {
        if !self.authentifie.get_untracked() {
            return;
        }
        spawn_local(async move {
            if !silencieux {
                self.chargement_cours.set(true);
            }
            self.erreur_cours.set(None);
            match api::cours_du_jour(None, &self.filtres()).await {
                Ok(d) => {
                    self.cours.set(d.jour.cours);
                    self.hors_ligne.set(d.etat.hors_ligne);
                    self.maj_flux.set(d.etat.maj.unwrap_or_default());
                }
                Err(e) => {
                    let m = self.message(e);
                    self.erreur_cours.set(Some(m));
                }
            }
            if !silencieux {
                self.chargement_cours.set(false);
            }
        });
    }

    pub fn charger_semaine(self) {
        if !self.authentifie.get_untracked() {
            return;
        }
        spawn_local(async move {
            self.chargement_semaine.set(true);
            self.erreur_semaine.set(None);
            let date = self.semaine_date.get_untracked();
            match api::cours_semaine(Some(&date), &self.filtres()).await {
                Ok(d) => {
                    self.cours_semaine.set(d.semaine.cours);
                    self.semaine_info
                        .set((d.semaine.date_debut, d.semaine.date_fin));
                    self.hors_ligne.set(d.etat.hors_ligne);
                    self.maj_flux.set(d.etat.maj.unwrap_or_default());
                }
                Err(e) => {
                    let m = self.message(e);
                    self.erreur_semaine.set(Some(m));
                }
            }
            self.chargement_semaine.set(false);
        });
    }

    pub fn changer_semaine(self, ecart_jours: i32) {
        let date = self.semaine_date.get_untracked();
        if date.is_empty() {
            return;
        }
        self.semaine_date
            .set(util::decaler_jours(&date, ecart_jours));
        self.charger_semaine();
    }

    pub fn recharger_selon_filtres(self) {
        self.enregistrer_preferences();
        self.charger_cours(false);
        self.charger_semaine();
    }

    // ------------------------------------------------------------------ Devoirs

    pub fn charger_devoirs(self) {
        if !self.authentifie.get_untracked() {
            return;
        }
        spawn_local(async move {
            self.charger_devoirs_async().await;
        });
    }

    pub async fn charger_devoirs_async(self) {
        self.erreur_devoirs.set(None);
        if let Ok(t) = api::lister_tags().await {
            self.tags.set(t);
        }
        match api::lister_devoirs().await {
            Ok(l) => self.devoirs.set(l),
            Err(e) => {
                let m = self.message(e);
                self.erreur_devoirs.set(Some(m));
            }
        }
    }

    pub fn ajouter_devoir(self, devoir: DevoirEntree, apres: impl FnOnce() + 'static) {
        spawn_local(async move {
            match api::creer_devoir(&devoir).await {
                Ok(_) => {
                    apres();
                    self.charger_devoirs_async().await;
                }
                Err(e) => {
                    let m = self.message(e);
                    self.erreur_devoirs.set(Some(m));
                }
            }
        });
    }

    pub fn changer_statut(self, id: i64, statut: &'static str) {
        spawn_local(async move {
            let m = DevoirModification {
                statut: Some(statut.to_string()),
                ..Default::default()
            };
            match api::modifier_devoir(id, &m).await {
                Ok(_) => self.devoirs.update(|l| {
                    if let Some(d) = l.iter_mut().find(|d| d.id == id) {
                        d.statut = statut.to_string();
                    }
                }),
                Err(e) => {
                    let m = self.message(e);
                    self.erreur_devoirs.set(Some(m));
                }
            }
        });
    }

    pub fn cycle_statut(self, d: &Devoir) {
        let suivant = match d.statut.as_str() {
            "a_faire" => "en_cours",
            "en_cours" => "termine",
            _ => "a_faire",
        };
        self.changer_statut(d.id, suivant);
    }

    pub fn deplacer(self, d: &Devoir, direction: i32) {
        const ORDRE: [&str; 3] = ["a_faire", "en_cours", "termine"];
        let Some(i) = ORDRE.iter().position(|s| *s == d.statut) else {
            return;
        };
        let j = i as i32 + direction;
        if (0..3).contains(&j) {
            self.changer_statut(d.id, ORDRE[j as usize]);
        }
    }

    pub fn supprimer_devoir(self, id: i64) {
        spawn_local(async move {
            match api::supprimer_devoir(id).await {
                Ok(()) => self.charger_devoirs_async().await,
                Err(e) => {
                    let m = self.message(e);
                    self.erreur_devoirs.set(Some(m));
                }
            }
        });
    }

    pub fn exporter_devoirs(self) {
        spawn_local(async move {
            match api::exporter_devoirs().await {
                Ok(Some(cible)) => self.notifier(match self.mode {
                    Mode::Tauri => format!("Devoirs exportés dans {cible}"),
                    Mode::Web => format!("Devoirs exportés : {cible}"),
                }),
                Ok(None) => {}
                Err(e) => {
                    let m = self.message(e);
                    self.erreur_devoirs.set(Some(m));
                }
            }
        });
    }

    pub fn importer_devoirs(self) {
        spawn_local(async move {
            match api::importer_devoirs().await {
                Ok(Some(n)) => {
                    self.notifier(format!("{n} devoir(s) importé(s)."));
                    self.charger_devoirs_async().await;
                }
                Ok(None) => {}
                Err(e) => {
                    let m = self.message(e);
                    self.erreur_devoirs.set(Some(m));
                }
            }
        });
    }

    // ------------------------------------------------------------------ Plan IA

    pub fn generer_plan(self, debut: String, consignes: String) {
        spawn_local(async move {
            self.chargement_plan.set(true);
            self.erreur_plan.set(None);
            match api::generer_plan(&self.filtres(), &debut, &consignes).await {
                Ok(p) => self.plan.set(Some(p)),
                Err(e) => {
                    let m = self.message(e);
                    self.erreur_plan.set(Some(m));
                }
            }
            self.chargement_plan.set(false);
        });
    }

    // ------------------------------------------------------------ Configuration

    pub async fn charger_config(self) {
        if let Ok(c) = api::obtenir_config().await {
            self.config.set(c);
        }
    }

    pub async fn enregistrer_config(self, maj: ConfigUpdate) -> Result<(), String> {
        match api::enregistrer_config(&maj).await {
            Ok(c) => {
                self.config.set(c);
                self.charger_cours(false);
                self.charger_semaine();
                Ok(())
            }
            Err(e) => Err(self.message(e)),
        }
    }

    pub async fn analyser_categories(self) -> Result<String, String> {
        match api::analyser_categories().await {
            Ok(c) => {
                let message = format!(
                    "{} groupe(s), {} type(s) et {} matière(s) détectés dans ton flux ADE.",
                    c.groupes.len(),
                    c.types.len(),
                    c.matieres.len()
                );
                if self.filtres_groupes.get_untracked().is_empty() {
                    self.filtres_groupes.set(c.groupes.clone());
                }
                if self.filtres_types.get_untracked().is_empty() {
                    self.filtres_types.set(c.types.clone());
                }
                if self.filtres_matieres.get_untracked().is_empty() {
                    self.filtres_matieres.set(c.matieres.clone());
                }
                self.categories.set(c);
                self.recharger_selon_filtres();
                Ok(message)
            }
            Err(e) => Err(self.message(e)),
        }
    }

    pub fn basculer_filtre(self, dim: Dimension, valeur: &str) {
        self.signal_filtre(dim)
            .update(|l| util::basculer(l, valeur));
        self.recharger_selon_filtres();
    }

    pub fn tout_cocher(self, dim: Dimension) {
        let toutes = match dim {
            Dimension::Groupes => self.categories.get_untracked().groupes,
            Dimension::Types => self.categories.get_untracked().types,
            Dimension::Matieres => self.categories.get_untracked().matieres,
        };
        self.signal_filtre(dim).set(toutes);
        self.recharger_selon_filtres();
    }

    pub fn tout_decocher(self, dim: Dimension) {
        self.signal_filtre(dim).set(vec![]);
        self.recharger_selon_filtres();
    }

    // -------------------------------------------------------------- Session Web

    pub fn deconnecter(self) {
        api::oublier_jeton();
        self.cours.set(vec![]);
        self.cours_semaine.set(vec![]);
        self.devoirs.set(vec![]);
        self.plan.set(None);
        self.config.set(Config::default());
        self.onboarding_ignore.set(false);
        self.onboarding_visible.set(false);
        self.onglet.set(Onglet::Dashboard);
        self.authentifie.set(false);
    }
}

// --------------------------------------------------------------- Libellés

pub fn statut_libelle(s: &str) -> &str {
    match s {
        "a_faire" => "À faire",
        "en_cours" => "En cours",
        "termine" => "Terminé",
        autre => autre,
    }
}

pub fn statut_point(s: &str) -> &'static str {
    match s {
        "a_faire" => "bg-rose-500",
        "en_cours" => "bg-amber-400",
        "termine" => "bg-emerald-500",
        _ => "bg-slate-400",
    }
}

pub fn statut_badge(s: &str) -> &'static str {
    match s {
        "a_faire" => "bg-rose-100 text-rose-700",
        "en_cours" => "bg-amber-100 text-amber-700",
        "termine" => "bg-emerald-100 text-emerald-700",
        _ => "bg-slate-100 text-slate-600",
    }
}

pub fn type_libelle(t: &str) -> &str {
    match t {
        "devoir" => "Devoir",
        "ie" => "IE",
        "ds" => "DS",
        "exam" => "Examen",
        autre => autre,
    }
}

pub fn type_badge(t: &str) -> &'static str {
    match t {
        "ie" => "bg-sky-100 text-sky-700",
        "ds" => "bg-orange-100 text-orange-700",
        "exam" => "bg-rose-100 text-rose-700",
        _ => "bg-slate-100 text-slate-600",
    }
}

pub fn libelle_echeance(e: &str) -> String {
    match util::jours_restants(e) {
        d if d < 0 => "En retard".into(),
        0 => "Aujourd'hui".into(),
        1 => "Demain".into(),
        d => format!("J-{d}"),
    }
}

pub fn couleur_echeance(e: &str) -> &'static str {
    match util::jours_restants(e) {
        d if d <= 0 => "bg-rose-50 text-rose-600 border-rose-200",
        d if d <= 3 => "bg-amber-50 text-amber-600 border-amber-200",
        d if d <= 7 => "bg-sky-50 text-sky-600 border-sky-200",
        _ => "bg-slate-50 text-slate-500 border-slate-200",
    }
}

pub const COLONNES: [(&str, &str, &str); 3] = [
    ("a_faire", "À faire", "bg-rose-100 text-rose-700"),
    ("en_cours", "En cours", "bg-amber-100 text-amber-700"),
    ("termine", "Terminé", "bg-emerald-100 text-emerald-700"),
];

pub const TYPES_DEVOIR: [(&str, &str); 4] = [
    ("devoir", "Devoir"),
    ("ie", "IE"),
    ("ds", "DS"),
    ("exam", "Examen"),
];

pub const MODELES_SUGGERES: [&str; 4] = [
    "gemini-3.8-flash",
    "gemini-2.5-flash",
    "gemini-2.5-flash-lite",
    "gemini-2.5-pro",
];
