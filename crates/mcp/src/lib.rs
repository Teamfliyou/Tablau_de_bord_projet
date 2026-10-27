//! Serveur MCP (Model Context Protocol) local du Tableau de bord étudiant, pour Hermes Agent.
//!
//! - Transport stdio uniquement : un message JSON-RPC 2.0 par ligne, aucun port réseau ouvert.
//! - Lit la base SQLite de l'application (mode WAL, accès concurrents gérés par SQLite) à chaque
//!   appel : Hermes voit toujours les données à jour, sans copie.
//! - Lecture seule par défaut ; les outils de modification exigent l'autorisation donnée dans
//!   l'application, puis une confirmation explicite (jeton à usage unique).
//! - Aucun accès au système de fichiers en dehors de la base et du journal.

mod outils;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

pub use outils::Outils;

/// Identifiant Tauri de l'application : nom du dossier de données dans %APPDATA%.
pub const IDENTIFIANT_APPLICATION: &str = "fr.teamfliyou.tableaudebord";
pub const NOM_BASE: &str = "tableau-de-bord.db";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Versions du protocole MCP acceptées (la première est proposée par défaut).
const VERSIONS_PROTOCOLE: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05", "2025-11-25"];

const JOURNAL_MAX_OCTETS: u64 = 1_000_000;

/// Emplacement de la base créée par l'application (même règle que `app_data_dir` de Tauri).
pub fn chemin_base_par_defaut() -> Option<PathBuf> {
    let dossier = if cfg!(windows) {
        PathBuf::from(std::env::var_os("APPDATA")?)
    } else if cfg!(target_os = "macos") {
        PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support")
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| Some(PathBuf::from(std::env::var_os("HOME")?).join(".local/share")))?
    };
    Some(dossier.join(IDENTIFIANT_APPLICATION).join(NOM_BASE))
}

/// Journal des appels (`logs/mcp.log` à côté de la base) : jamais sur stdout, réservé au protocole.
pub struct Journal {
    chemin: Option<PathBuf>,
}

impl Journal {
    pub fn pour_base(base: &Path) -> Self {
        let chemin = base.parent().map(|d| d.join("logs").join("mcp.log"));
        Journal { chemin }
    }

    pub fn inactif() -> Self {
        Journal { chemin: None }
    }

    pub fn ecrire(&self, niveau: &str, message: &str) {
        let Some(chemin) = &self.chemin else {
            return;
        };
        let Some(dossier) = chemin.parent() else {
            return;
        };
        if fs::create_dir_all(dossier).is_err() {
            return;
        }
        if fs::metadata(chemin).is_ok_and(|m| m.len() > JOURNAL_MAX_OCTETS) {
            let _ = fs::rename(chemin, chemin.with_extension("log.1"));
        }
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(chemin) {
            let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(f, "{date} [{niveau}] {message}");
        }
    }
}

/// Serveur MCP : traite une ligne JSON-RPC et renvoie la réponse (aucune pour une notification).
pub struct Serveur {
    outils: Outils,
    journal: Journal,
}

impl Serveur {
    pub fn nouveau(base: PathBuf, journal: Journal) -> Self {
        Serveur {
            outils: Outils::nouveau(base),
            journal,
        }
    }

    pub fn journal(&self) -> &Journal {
        &self.journal
    }

    pub fn traiter(&mut self, ligne: &str) -> Option<String> {
        let message: Value = match serde_json::from_str(ligne) {
            Ok(v) => v,
            Err(_) => return Some(erreur(Value::Null, -32700, "JSON invalide.").to_string()),
        };
        let Some(objet) = message.as_object() else {
            return Some(erreur(Value::Null, -32600, "Requête JSON-RPC invalide.").to_string());
        };
        let methode = objet.get("method").and_then(Value::as_str);
        let id = objet.get("id").cloned();
        let (Some(methode), Some(id)) = (methode, id) else {
            // Notification (`notifications/initialized`, `notifications/cancelled`…) ou réponse : rien à renvoyer.
            return None;
        };
        let params = objet.get("params").cloned().unwrap_or(Value::Null);
        let reponse = match methode {
            "initialize" => succes(id, self.initialiser(&params)),
            "ping" => succes(id, json!({})),
            "tools/list" => succes(id, json!({ "tools": Outils::catalogue() })),
            "tools/call" => self.appeler(id, &params),
            _ => erreur(
                id,
                -32601,
                &format!("Méthode non prise en charge : {methode}"),
            ),
        };
        Some(reponse.to_string())
    }

    fn initialiser(&self, params: &Value) -> Value {
        let demandee = params.get("protocolVersion").and_then(Value::as_str);
        let version = demandee
            .filter(|v| VERSIONS_PROTOCOLE.contains(v))
            .unwrap_or(VERSIONS_PROTOCOLE[0]);
        let client = params
            .pointer("/clientInfo/name")
            .and_then(Value::as_str)
            .unwrap_or("inconnu");
        self.journal.ecrire(
            "INFO",
            &format!("Connexion du client « {client} » (protocole {version})"),
        );
        json!({
            "protocolVersion": version,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "tableau-de-bord", "title": "Tableau de bord étudiant", "version": VERSION },
            "instructions": "Accès aux devoirs du Tableau de bord étudiant. Un « projet » correspond à une matière, une « tâche » à un devoir. \
        Les dates sont au format AAAA-MM-JJ, « aujourd'hui » est calculé à l'heure de Paris. \
        Les outils de modification sont désactivés sauf autorisation dans l'application ; quand ils sont autorisés, ils renvoient d'abord un aperçu \
        et un jeton de confirmation : demande l'accord de l'utilisateur avant de rappeler l'outil avec ce jeton."
        })
    }

    fn appeler(&mut self, id: Value, params: &Value) -> Value {
        let Some(nom) = params.get("name").and_then(Value::as_str) else {
            return erreur(id, -32602, "Paramètre « name » manquant.");
        };
        if !Outils::existe(nom) {
            return erreur(id, -32602, &format!("Outil inconnu : {nom}"));
        }
        let arguments = match params.get("arguments") {
            None | Some(Value::Null) => json!({}),
            Some(v @ Value::Object(_)) => v.clone(),
            Some(_) => return erreur(id, -32602, "« arguments » doit être un objet."),
        };
        let resultat = match self.outils.executer(nom, &arguments) {
            Ok(donnees) => {
                self.journal.ecrire("INFO", &format!("{nom} : ok"));
                let texte = serde_json::to_string_pretty(&donnees).unwrap_or_default();
                json!({
                    "content": [{ "type": "text", "text": texte }],
                    "structuredContent": donnees,
                    "isError": false
                })
            }
            Err(message) => {
                self.journal
                    .ecrire("AVERTISSEMENT", &format!("{nom} : {message}"));
                json!({ "content": [{ "type": "text", "text": message }], "isError": true })
            }
        };
        succes(id, resultat)
    }
}

fn succes(id: Value, resultat: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": resultat })
}

fn erreur(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
