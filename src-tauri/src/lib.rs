//! Application Tauri : expose le cœur métier (`tdb-core`) au frontend Vue via des commandes IPC.
//! Aucun serveur HTTP local n'est démarré.

mod coffre;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
use tdb_core::ade::{Categories, Filtres};
use tdb_core::db::{
    Db, Devoir, DevoirEntree, DevoirModification, RapportMigration, IMPORT_MAX_OCTETS,
};
use tdb_core::planner::Plan;
use tdb_core::service::{Config, ConfigUpdate, CoursDuJourEtat, CoursSemaineEtat};
use tdb_core::{Erreur, Resultat, Service};

type Etat<'a> = State<'a, Arc<Service>>;

#[tauri::command]
fn obtenir_config(s: Etat) -> Resultat<Config> {
    s.obtenir_config()
}

#[tauri::command]
fn enregistrer_config(s: Etat, maj: ConfigUpdate) -> Resultat<Config> {
    s.enregistrer_config(maj)
}

#[tauri::command]
fn obtenir_preferences(s: Etat) -> Resultat<serde_json::Value> {
    s.obtenir_preferences()
}

#[tauri::command]
fn enregistrer_preferences(s: Etat, preferences: serde_json::Value) -> Resultat<()> {
    s.enregistrer_preferences(preferences)
}

#[tauri::command]
async fn cours_du_jour(
    s: Etat<'_>,
    date: Option<String>,
    filtres: Option<Filtres>,
) -> Resultat<CoursDuJourEtat> {
    s.cours_du_jour(date.as_deref(), &filtres.unwrap_or_default())
        .await
}

#[tauri::command]
async fn cours_semaine(
    s: Etat<'_>,
    date: Option<String>,
    filtres: Option<Filtres>,
) -> Resultat<CoursSemaineEtat> {
    s.cours_semaine(date.as_deref(), &filtres.unwrap_or_default())
        .await
}

#[tauri::command]
async fn analyser_categories(s: Etat<'_>) -> Resultat<Categories> {
    s.analyser_categories().await
}

#[tauri::command]
fn lister_devoirs(s: Etat) -> Resultat<Vec<Devoir>> {
    s.db.lister_devoirs()
}

#[tauri::command]
fn creer_devoir(s: Etat, devoir: DevoirEntree) -> Resultat<Devoir> {
    s.db.creer_devoir(devoir)
}

#[tauri::command]
fn modifier_devoir(s: Etat, id: i64, modification: DevoirModification) -> Resultat<Devoir> {
    s.db.modifier_devoir(id, modification)
}

#[tauri::command]
fn supprimer_devoir(s: Etat, id: i64) -> Resultat<()> {
    s.db.supprimer_devoir(id)
}

/// Export JSON : la boîte « Enregistrer sous » est ouverte côté Rust, le frontend n'a aucun accès disque.
#[tauri::command]
async fn exporter_devoirs(app: tauri::AppHandle, s: Etat<'_>) -> Resultat<Option<String>> {
    let jour = chrono_jour();
    let Some(chemin) = app
        .dialog()
        .file()
        .set_file_name(format!("devoirs_{jour}.json"))
        .add_filter("Fichier JSON", &["json"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let chemin: PathBuf = chemin
        .into_path()
        .map_err(|_| Erreur::Interne("Chemin de fichier invalide.".into()))?;
    let json = serde_json::to_string_pretty(&s.db.exporter_devoirs()?)
        .map_err(|_| Erreur::Interne("Export impossible.".into()))?;
    std::fs::write(&chemin, json)
        .map_err(|e| Erreur::Interne(format!("Écriture impossible : {e}")))?;
    Ok(Some(chemin.display().to_string()))
}

/// Import JSON : sélection du fichier côté Rust, taille bornée, validation atomique.
#[tauri::command]
async fn importer_devoirs(app: tauri::AppHandle, s: Etat<'_>) -> Resultat<Option<usize>> {
    let Some(chemin) = app
        .dialog()
        .file()
        .add_filter("Fichier JSON", &["json"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let chemin: PathBuf = chemin
        .into_path()
        .map_err(|_| Erreur::Interne("Chemin de fichier invalide.".into()))?;
    let contenu = lire_fichier_borne(&chemin)?;
    Ok(Some(s.db.importer_devoirs_json(&contenu)?))
}

/// Importe les devoirs et paramètres d'une ancienne base SQLite de la version Python (lecture seule).
#[tauri::command]
async fn migrer_ancienne_base(
    app: tauri::AppHandle,
    s: Etat<'_>,
) -> Resultat<Option<RapportMigration>> {
    let Some(chemin) = app
        .dialog()
        .file()
        .add_filter("Base SQLite", &["db", "sqlite", "sqlite3"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let chemin: PathBuf = chemin
        .into_path()
        .map_err(|_| Erreur::Interne("Chemin de fichier invalide.".into()))?;
    Ok(Some(s.migrer_ancienne_base(&chemin)?))
}

#[tauri::command]
async fn generer_plan(s: Etat<'_>, filtres: Option<Filtres>) -> Resultat<Plan> {
    s.generer_plan(&filtres.unwrap_or_default()).await
}

#[tauri::command]
fn version_application(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

fn lire_fichier_borne(chemin: &std::path::Path) -> Resultat<String> {
    let meta =
        std::fs::metadata(chemin).map_err(|_| Erreur::Validation("Fichier illisible.".into()))?;
    if !meta.is_file() || meta.len() > IMPORT_MAX_OCTETS {
        return Err(Erreur::Validation(
            "Fichier invalide ou trop volumineux (2 Mo maximum).".into(),
        ));
    }
    std::fs::read_to_string(chemin)
        .map_err(|_| Erreur::Validation("Le fichier n'est pas un texte UTF-8 valide.".into()))
}

fn chrono_jour() -> String {
    // Date locale AAAA-MM-JJ sans dépendre d'une crate supplémentaire côté application.
    tdb_core::ade::aujourdhui().to_string()
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(fenetre) = app.get_webview_window("main") {
                let _ = fenetre.unminimize();
                let _ = fenetre.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .max_file_size(1_000_000)
                .build(),
        )
        .setup(|app| {
            // Les données vivent dans %APPDATA% (jamais dans un dossier temporaire), y compris en version portable.
            let dossier = app.path().app_data_dir()?;
            let db = Db::ouvrir(&dossier.join("tableau-de-bord.db"))?;
            if let Err(e) = db.sauvegarde_quotidienne() {
                log::warn!("Sauvegarde quotidienne impossible : {e}");
            }
            let service = Arc::new(Service::nouveau(db, Arc::new(coffre::CoffreWindows)));
            app.manage(service);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            obtenir_config,
            enregistrer_config,
            obtenir_preferences,
            enregistrer_preferences,
            cours_du_jour,
            cours_semaine,
            analyser_categories,
            lister_devoirs,
            creer_devoir,
            modifier_devoir,
            supprimer_devoir,
            exporter_devoirs,
            importer_devoirs,
            migrer_ancienne_base,
            generer_plan,
            version_application,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'application Tableau de bord étudiant");
}
