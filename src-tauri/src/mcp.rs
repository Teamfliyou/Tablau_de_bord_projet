//! Intégration Hermes Agent / MCP : réglages, configuration à copier et test de connexion.
//!
//! Le serveur MCP (`tdb-mcp.exe`) est un exécutable distinct installé à côté de l'application :
//! Hermes le lance lui-même (stdio), sans que la fenêtre du tableau de bord soit ouverte.
//! L'application ne fait que publier les autorisations dans la base partagée.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager, State};
use tdb_core::{Erreur, Resultat, Service};
use tdb_shared::{
    EtatMcp, TestMcp, EXECUTABLE_MCP, NOM_SERVEUR_MCP, PARAM_MCP_ACTIF, PARAM_MCP_ECRITURE,
};

const DELAI_TEST: Duration = Duration::from_secs(10);
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// `tdb-mcp.exe` installé à côté de l'exécutable de l'application.
pub fn chemin_executable() -> Option<PathBuf> {
    let dossier = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let nom = format!("{EXECUTABLE_MCP}{}", std::env::consts::EXE_SUFFIX);
    Some(dossier.join(nom)).filter(|p| p.is_file())
}

fn chemin_base(app: &AppHandle) -> Resultat<PathBuf> {
    app.path()
        .app_data_dir()
        .map(|d| d.join("tableau-de-bord.db"))
        .map_err(|_| Erreur::Interne("Dossier de données introuvable.".into()))
}

fn actif(s: &Service, cle: &str) -> Resultat<bool> {
    Ok(s.db.lire_parametre(cle)?.as_deref() == Some("1"))
}

/// Chaîne YAML entre apostrophes : aucun échappement des barres obliques inverses de Windows.
fn yaml(valeur: &Path) -> String {
    format!("'{}'", valeur.display().to_string().replace('\'', "''"))
}

/// Bloc à ajouter dans `config.yaml` de Hermes Agent (clé `mcp_servers`).
fn configuration(executable: &Path, base: &Path) -> String {
    format!(
        "mcp_servers:\n  {NOM_SERVEUR_MCP}:\n    command: {}\n    args: ['--base', {}]\n    timeout: 30\n    connect_timeout: 15\n    idle_timeout_seconds: 600\n",
        yaml(executable),
        yaml(base)
    )
}

fn etat(app: &AppHandle, s: &Service) -> Resultat<EtatMcp> {
    let base = chemin_base(app)?;
    let executable = chemin_executable();
    let exemple = executable.clone().unwrap_or_else(|| {
        PathBuf::from(format!("{EXECUTABLE_MCP}{}", std::env::consts::EXE_SUFFIX))
    });
    Ok(EtatMcp {
        actif: actif(s, PARAM_MCP_ACTIF)?,
        ecriture: actif(s, PARAM_MCP_ECRITURE)?,
        executable: executable.map(|p| p.display().to_string()),
        base: base.display().to_string(),
        configuration: configuration(&exemple, &base),
    })
}

#[tauri::command]
pub fn etat_mcp(app: AppHandle, s: State<'_, Arc<Service>>) -> Resultat<EtatMcp> {
    etat(&app, &s)
}

/// Enregistre les autorisations ; le serveur MCP les relit à chaque appel (effet immédiat).
#[tauri::command]
pub fn definir_mcp(
    app: AppHandle,
    s: State<'_, Arc<Service>>,
    actif: bool,
    ecriture: bool,
) -> Resultat<EtatMcp> {
    let valeur = |b: bool| if b { "1" } else { "0" };
    s.db.ecrire_parametre(PARAM_MCP_ACTIF, valeur(actif))?;
    // Sans intégration active, l'écriture est toujours retirée.
    s.db.ecrire_parametre(PARAM_MCP_ECRITURE, valeur(actif && ecriture))?;
    log::info!(
        "Intégration MCP : active={actif}, écriture={}",
        actif && ecriture
    );
    etat(&app, &s)
}

/// Lance réellement le serveur MCP comme le ferait Hermes, puis l'interroge (initialize, tools/list, un outil).
#[tauri::command]
pub async fn tester_mcp(app: AppHandle) -> Resultat<TestMcp> {
    let base = chemin_base(&app)?;
    let Some(executable) = chemin_executable() else {
        return Ok(TestMcp {
            ok: false,
            message: format!(
                "Serveur MCP introuvable ({EXECUTABLE_MCP}.exe absent du dossier de l'application). Réinstalle l'application avec l'installateur de la version 5.1.0 ou plus récente."
            ),
            ..Default::default()
        });
    };
    let mut processus = match lancer(&executable, &base) {
        Ok(p) => p,
        Err(message) => {
            return Ok(TestMcp {
                ok: false,
                message,
                ..Default::default()
            })
        }
    };
    let (Some(entree), Some(sortie)) = (processus.stdin.take(), processus.stdout.take()) else {
        let _ = processus.kill();
        return Err(Erreur::Interne(
            "Communication avec le serveur MCP impossible.".into(),
        ));
    };
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(dialoguer(entree, BufReader::new(sortie)));
    });
    let resultat = tauri::async_runtime::spawn_blocking(move || {
        let r = rx.recv_timeout(DELAI_TEST);
        // Toujours arrêté, y compris après un dépassement de délai (le fil de dialogue se termine alors).
        let _ = processus.kill();
        let _ = processus.wait();
        r
    })
    .await
    .map_err(|_| Erreur::Interne("Test interrompu.".into()))?;
    Ok(match resultat {
        Ok(Ok(test)) => test,
        Ok(Err(message)) => TestMcp {
            ok: false,
            message,
            ..Default::default()
        },
        Err(_) => TestMcp {
            ok: false,
            message: "Le serveur MCP ne répond pas (délai de 10 secondes dépassé).".into(),
            ..Default::default()
        },
    })
}

fn lancer(executable: &Path, base: &Path) -> Result<Child, String> {
    let mut commande = Command::new(executable);
    commande
        .arg("--base")
        .arg(base)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        commande.creation_flags(CREATE_NO_WINDOW);
    }
    commande
        .spawn()
        .map_err(|e| format!("Impossible de lancer le serveur MCP : {e}"))
}

fn dialoguer(
    mut entree: ChildStdin,
    mut sortie: BufReader<ChildStdout>,
) -> Result<TestMcp, String> {
    let mut id = 0;
    let mut requete = |methode: &str, params: Value| -> Result<Value, String> {
        id += 1;
        let message = json!({ "jsonrpc": "2.0", "id": id, "method": methode, "params": params });
        writeln!(entree, "{message}")
            .and_then(|_| entree.flush())
            .map_err(|e| format!("Envoi impossible : {e}"))?;
        let mut ligne = String::new();
        match sortie.read_line(&mut ligne) {
            Ok(0) => return Err("Le serveur MCP s'est arrêté sans répondre.".into()),
            Ok(_) => {}
            Err(e) => return Err(format!("Lecture impossible : {e}")),
        }
        let reponse: Value = serde_json::from_str(&ligne)
            .map_err(|_| "Réponse illisible du serveur MCP.".to_string())?;
        if let Some(e) = reponse.get("error") {
            return Err(format!(
                "Erreur du serveur MCP : {}",
                e["message"].as_str().unwrap_or("inconnue")
            ));
        }
        Ok(reponse["result"].clone())
    };
    let init = requete(
        "initialize",
        json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "tableau-de-bord-test", "version": env!("CARGO_PKG_VERSION") } }),
    )?;
    let version = init["serverInfo"]["version"].as_str().map(str::to_string);
    let outils: Vec<String> = requete("tools/list", json!({}))?["tools"]
        .as_array()
        .map(|l| {
            l.iter()
                .filter_map(|t| t["name"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let appel = requete(
        "tools/call",
        json!({ "name": "get_project_progress", "arguments": {} }),
    )?;
    let (ok, message) = if appel["isError"].as_bool() == Some(false) {
        let taches = appel["structuredContent"]["global"]["taches"]
            .as_u64()
            .unwrap_or(0);
        (true, format!("Connexion réussie : le serveur MCP lit ta base ({taches} tâche(s)). Hermes Agent peut s'y connecter."))
    } else {
        let detail = appel["content"][0]["text"]
            .as_str()
            .unwrap_or("erreur inconnue");
        (
            false,
            format!("Le serveur MCP démarre, mais refuse l'accès aux données : {detail}"),
        )
    };
    Ok(TestMcp {
        ok,
        message,
        version,
        outils,
    })
}

/// Avant une mise à jour : arrête les serveurs MCP lancés par Hermes depuis le dossier
/// d'installation, sinon Windows verrouille `tdb-mcp.exe` et l'installateur ne peut pas le remplacer.
/// Hermes relance le serveur à sa prochaine utilisation.
pub fn arreter_serveurs_mcp() {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let image = format!("{EXECUTABLE_MCP}.exe");
        match Command::new("taskkill")
            .args(["/F", "/IM", &image])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(s) if s.success() => log::info!("Serveur(s) MCP arrêté(s) avant la mise à jour"),
            // Code non nul : aucun serveur MCP en cours d'exécution.
            Ok(_) => {}
            Err(e) => log::warn!("Arrêt des serveurs MCP impossible : {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `tdb-mcp` compilé par `cargo test --workspace` (target/<profil>/), à côté du dossier `deps` du test.
    fn executable_compile() -> Option<PathBuf> {
        let dossier = std::env::current_exe()
            .ok()?
            .parent()?
            .parent()?
            .to_path_buf();
        let nom = format!("{EXECUTABLE_MCP}{}", std::env::consts::EXE_SUFFIX);
        Some(dossier.join(nom)).filter(|p| p.is_file())
    }

    #[test]
    fn test_de_connexion_avec_le_vrai_serveur() {
        let Some(executable) = executable_compile() else {
            eprintln!("tdb-mcp non compilé : lance `cargo test --workspace` pour ce test.");
            return;
        };
        let dossier =
            std::env::temp_dir().join(format!("tdb-test-connexion-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        let base = dossier.join("tableau-de-bord.db");
        let db = tdb_core::db::Db::ouvrir(&base).unwrap();
        let essai = || {
            let mut p = lancer(&executable, &base).unwrap();
            let r = dialoguer(
                p.stdin.take().unwrap(),
                BufReader::new(p.stdout.take().unwrap()),
            );
            let _ = p.kill();
            let _ = p.wait();
            r.unwrap()
        };
        let desactive = essai();
        assert!(!desactive.ok);
        assert!(
            desactive.message.contains("désactivée"),
            "{}",
            desactive.message
        );
        assert_eq!(desactive.outils.len(), tdb_shared::OUTILS_MCP.len());
        assert_eq!(
            desactive.version.as_deref(),
            Some(env!("CARGO_PKG_VERSION"))
        );

        db.ecrire_parametre(PARAM_MCP_ACTIF, "1").unwrap();
        let actif = essai();
        assert!(actif.ok, "{}", actif.message);
        assert!(actif.message.contains("0 tâche(s)"));
        drop(db);
        let _ = std::fs::remove_dir_all(&dossier);
    }

    #[test]
    fn configuration_yaml_chemins_windows() {
        let c = configuration(
            Path::new(r"C:\Users\Léa O'Neil\AppData\Local\Tableau de bord étudiant\tdb-mcp.exe"),
            Path::new(
                r"C:\Users\Léa O'Neil\AppData\Roaming\fr.teamfliyou.tableaudebord\tableau-de-bord.db",
            ),
        );
        assert!(c.starts_with("mcp_servers:\n  tableau_de_bord:\n"));
        assert!(c.contains(
            r"command: 'C:\Users\Léa O''Neil\AppData\Local\Tableau de bord étudiant\tdb-mcp.exe'"
        ));
        assert!(c.contains(r"args: ['--base', 'C:\Users\Léa O''Neil\AppData\Roaming\fr.teamfliyou.tableaudebord\tableau-de-bord.db']"));
    }
}
