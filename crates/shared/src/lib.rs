//! Contrats de données communs : aucune dépendance système, compilable en WebAssembly.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub const TYPES: [&str; 4] = ["devoir", "ie", "ds", "exam"];
pub const STATUTS: [&str; 3] = ["a_faire", "en_cours", "termine"];
pub const IMPORT_MAX: usize = 500;
pub const IMPORT_MAX_OCTETS: u64 = 2 * 1024 * 1024;
pub const MODELE_PAR_DEFAUT: &str = "gemini-3.8-flash";

// ------------------------------------------------------------------ Devoirs

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Devoir {
    pub id: i64,
    pub titre: String,
    pub matiere: String,
    pub echeance: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub statut: String,
}

/// Données d'un devoir à créer / importer (champs absents = valeurs par défaut).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DevoirEntree {
    pub titre: String,
    #[serde(default)]
    pub matiere: String,
    #[serde(default)]
    pub echeance: String,
    #[serde(default = "type_defaut", rename = "type")]
    pub type_: String,
    #[serde(default = "statut_defaut")]
    pub statut: String,
}

fn type_defaut() -> String {
    "devoir".into()
}
fn statut_defaut() -> String {
    "a_faire".into()
}

/// Modification partielle d'un devoir.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct DevoirModification {
    pub titre: Option<String>,
    pub matiere: Option<String>,
    pub echeance: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub statut: Option<String>,
}

impl DevoirEntree {
    /// Nettoie et valide les champs (mêmes règles que le schéma Pydantic historique).
    pub fn valider(mut self) -> Result<Self, String> {
        self.titre = self.titre.trim().to_string();
        self.matiere = self.matiere.trim().to_string();
        self.echeance = self.echeance.trim().to_string();
        valider_titre(&self.titre)?;
        valider_matiere(&self.matiere)?;
        valider_echeance(&self.echeance)?;
        valider_type(&self.type_)?;
        valider_statut(&self.statut)?;
        Ok(self)
    }
}

pub fn valider_titre(t: &str) -> Result<(), String> {
    if t.is_empty() || t.chars().count() > 200 {
        return Err("Le titre doit contenir entre 1 et 200 caractères.".into());
    }
    Ok(())
}
pub fn valider_matiere(m: &str) -> Result<(), String> {
    if m.chars().count() > 120 {
        return Err("La matière est limitée à 120 caractères.".into());
    }
    Ok(())
}
pub fn valider_echeance(e: &str) -> Result<(), String> {
    if !e.is_empty() && NaiveDate::parse_from_str(e, "%Y-%m-%d").is_err() {
        return Err("L'échéance doit être au format AAAA-MM-JJ.".into());
    }
    Ok(())
}
pub fn valider_type(t: &str) -> Result<(), String> {
    if !TYPES.contains(&t) {
        return Err("Type de devoir invalide.".into());
    }
    Ok(())
}
pub fn valider_statut(s: &str) -> Result<(), String> {
    if !STATUTS.contains(&s) {
        return Err("Statut de devoir invalide.".into());
    }
    Ok(())
}

impl DevoirModification {
    /// Applique la modification à un devoir existant en validant chaque champ présent.
    pub fn appliquer(self, mut d: Devoir) -> Result<Devoir, String> {
        if let Some(v) = self.titre {
            d.titre = v.trim().to_string();
            valider_titre(&d.titre)?;
        }
        if let Some(v) = self.matiere {
            d.matiere = v.trim().to_string();
            valider_matiere(&d.matiere)?;
        }
        if let Some(v) = self.echeance {
            d.echeance = v.trim().to_string();
            valider_echeance(&d.echeance)?;
        }
        if let Some(v) = self.type_ {
            valider_type(&v)?;
            d.type_ = v;
        }
        if let Some(v) = self.statut {
            valider_statut(&v)?;
            d.statut = v;
        }
        Ok(d)
    }
}

// ------------------------------------------------------------- Emploi du temps

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cours {
    pub titre: String,
    pub salle: String,
    pub enseignant: String,
    pub enseignants: Vec<String>,
    pub debut: String,
    pub fin: String,
    pub debut_iso: String,
    pub fin_iso: String,
    pub date: String,
    pub groupes: Vec<String>,
    pub types: Vec<String>,
    pub matiere: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub struct Filtres {
    #[serde(default)]
    pub groupes: Vec<String>,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub matieres: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Categories {
    pub groupes: Vec<String>,
    pub types: Vec<String>,
    pub matieres: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoursDuJour {
    pub date: String,
    pub nb_cours: usize,
    pub cours: Vec<Cours>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoursSemaine {
    pub date_debut: String,
    pub date_fin: String,
    pub cours: Vec<Cours>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct EtatFlux {
    /// `true` si l'emploi du temps affiché provient du dernier cache faute de connexion.
    #[serde(default)]
    pub hors_ligne: bool,
    /// Date de la dernière récupération réussie (RFC 3339).
    #[serde(default)]
    pub maj: Option<String>,
    #[serde(default)]
    pub avertissement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoursDuJourEtat {
    #[serde(flatten)]
    pub jour: CoursDuJour,
    #[serde(flatten)]
    pub etat: EtatFlux,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoursSemaineEtat {
    #[serde(flatten)]
    pub semaine: CoursSemaine,
    #[serde(flatten)]
    pub etat: EtatFlux,
}

// -------------------------------------------------------------------- Plan IA

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Creneau {
    pub heure: String,
    pub matiere: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Plan {
    pub conseil: String,
    pub planning: Vec<Creneau>,
    /// "gemini" ou "simulation"
    pub mode: String,
    /// Raison pour laquelle la simulation a remplacé Gemini (réseau, quota, clé invalide…).
    pub avertissement: Option<String>,
}

// -------------------------------------------------------------- Configuration

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Config {
    pub username: String,
    pub ade_url: String,
    pub gemini_model: String,
    pub gemini_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigUpdate {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub ade_url: String,
    #[serde(default)]
    pub gemini_model: Option<String>,
    /// Nouvelle clé (jamais relue côté interface). Vide ou absente : clé inchangée.
    #[serde(default)]
    pub gemini_key: Option<String>,
    #[serde(default)]
    pub remove_gemini_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RapportMigration {
    pub devoirs_importes: usize,
    pub devoirs_ignores: usize,
    pub parametres_importes: bool,
    pub cle_gemini_trouvee: bool,
}

// ----------------------------------------------------------- Authentification

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inscription {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connexion {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profil {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub ade_url: String,
    pub gemini_configured: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReponseAuth {
    pub token: String,
    pub user: Profil,
}
