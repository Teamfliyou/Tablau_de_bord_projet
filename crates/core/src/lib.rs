//! Cœur métier du Tableau de bord étudiant : aucune dépendance à Tauri, entièrement testable.

pub mod ade;
pub mod db;
pub mod error;
pub mod maj;
pub mod planner;
pub mod secrets;
pub mod service;

pub use error::{Erreur, Resultat};
pub use service::Service;
