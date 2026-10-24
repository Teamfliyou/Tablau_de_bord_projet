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
    #[serde(default)]
    pub tags: Vec<Tag>,
    /// Sous-tâches (to-do list) ; absentes des anciennes données : liste vide.
    #[serde(default)]
    pub todos: Vec<Todo>,
    /// Notes libres (absentes des anciennes données : vide).
    #[serde(default)]
    pub description: String,
    /// 0 = aucune, 1 = basse, 2 = moyenne, 3 = haute.
    #[serde(default)]
    pub priorite: u8,
}

/// Élément de to-do d'un devoir.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    pub id: i64,
    pub texte: String,
    pub fait: bool,
}

/// Élément de to-do sans identifiant : enregistrement, import / export JSON.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TodoEntree {
    pub texte: String,
    #[serde(default)]
    pub fait: bool,
}

pub const TODOS_PAR_DEVOIR_MAX: usize = 100;

impl TodoEntree {
    /// Nettoie et valide un élément (texte de 1 à 200 caractères).
    pub fn valider(mut self) -> Result<Self, String> {
        self.texte = self.texte.trim().to_string();
        if self.texte.is_empty() || self.texte.chars().count() > 200 {
            return Err("Une sous-tâche doit contenir entre 1 et 200 caractères.".into());
        }
        Ok(self)
    }

    /// Valide toute une liste (100 éléments maximum).
    pub fn valider_liste(liste: Vec<Self>) -> Result<Vec<Self>, String> {
        if liste.len() > TODOS_PAR_DEVOIR_MAX {
            return Err("Un devoir ne peut pas avoir plus de 100 sous-tâches.".into());
        }
        liste.into_iter().map(Self::valider).collect()
    }
}

/// Tag personnalisé (réutilisable sur plusieurs devoirs).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub id: i64,
    pub nom: String,
    pub couleur: String,
}

/// Tag sans identifiant : création, modification, import / export JSON.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TagEntree {
    pub nom: String,
    #[serde(default = "couleur_tag_defaut")]
    pub couleur: String,
}

pub const TAGS_PAR_DEVOIR_MAX: usize = 20;

fn couleur_tag_defaut() -> String {
    "#6366f1".into()
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
    /// Absent dans les anciens fichiers : liste vide, export identique à l'ancien format.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<TagEntree>,
    /// Sous-tâches avec leur état ; absentes des anciens fichiers : liste vide.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub todos: Vec<TodoEntree>,
    /// Absents des anciens fichiers : valeurs par défaut, export identique à l'ancien format.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "est_zero")]
    pub priorite: u8,
}

fn est_zero(p: &u8) -> bool {
    *p == 0
}

pub const DESCRIPTION_MAX: usize = 2000;
pub const PRIORITE_MAX: u8 = 3;

pub fn valider_description(d: &str) -> Result<(), String> {
    if d.chars().count() > DESCRIPTION_MAX {
        return Err("La description est limitée à 2000 caractères.".into());
    }
    Ok(())
}
pub fn valider_priorite(p: u8) -> Result<(), String> {
    if p > PRIORITE_MAX {
        return Err("Priorité invalide.".into());
    }
    Ok(())
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
    pub description: Option<String>,
    pub priorite: Option<u8>,
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
        self.description = self.description.trim().to_string();
        valider_description(&self.description)?;
        valider_priorite(self.priorite)?;
        if self.tags.len() > TAGS_PAR_DEVOIR_MAX {
            return Err("Un devoir ne peut pas avoir plus de 20 tags.".into());
        }
        self.tags = self
            .tags
            .into_iter()
            .map(TagEntree::valider)
            .collect::<Result<_, _>>()?;
        self.todos = TodoEntree::valider_liste(self.todos)?;
        Ok(self)
    }
}

impl TagEntree {
    /// Nettoie et valide un tag (nom 1 à 30 caractères, couleur `#RRGGBB`).
    pub fn valider(mut self) -> Result<Self, String> {
        self.nom = self.nom.trim().to_string();
        self.couleur = self.couleur.to_ascii_lowercase();
        if self.nom.is_empty() || self.nom.chars().count() > 30 {
            return Err("Le nom d'un tag doit contenir entre 1 et 30 caractères.".into());
        }
        let hex = self.couleur.strip_prefix('#').unwrap_or("");
        if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("La couleur d'un tag doit être au format #RRGGBB.".into());
        }
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
        if let Some(v) = self.description {
            d.description = v.trim().to_string();
            valider_description(&d.description)?;
        }
        if let Some(v) = self.priorite {
            valider_priorite(v)?;
            d.priorite = v;
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

// ------------------------------------------------------- Mise à jour (Windows)

/// Mise à jour plus récente trouvée sur GitHub Releases.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InfoMiseAJour {
    pub version_actuelle: String,
    pub version: String,
    /// Notes de version publiées avec la release (texte brut).
    pub notes: Option<String>,
    /// `false` pour l'exécutable portable : installation automatique refusée.
    pub installable: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PhaseMiseAJour {
    #[default]
    Inactif,
    Telechargement,
    Installation,
}

/// Progression réelle du téléchargement (octets reçus / taille annoncée par le serveur).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ProgressionMiseAJour {
    pub phase: PhaseMiseAJour,
    pub telecharge: u64,
    pub total: Option<u64>,
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
