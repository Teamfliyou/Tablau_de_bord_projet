// Aucune console visible au lancement sous Windows (build release).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(erreur) = tableau_de_bord_etudiant_lib::run() {
        eprintln!("Tableau de bord étudiant : lancement impossible : {erreur}");
        std::process::exit(1);
    }
}
