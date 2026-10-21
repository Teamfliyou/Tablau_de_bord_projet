// Aucune console visible au lancement sous Windows (build release).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tableau_de_bord_etudiant_lib::run()
}
