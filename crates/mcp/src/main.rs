//! `tdb-mcp` : serveur MCP stdio lancé par Hermes Agent (ou tout client MCP local).
//!
//! Usage : `tdb-mcp [--base <chemin de tableau-de-bord.db>]`
//! Sans argument, la base de l'application est cherchée dans %APPDATA%\fr.teamfliyou.tableaudebord.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use tdb_mcp::{chemin_base_par_defaut, Journal, Serveur, VERSION};

const AIDE: &str = "tdb-mcp — serveur MCP (stdio) du Tableau de bord étudiant

Usage : tdb-mcp [--base <chemin>]

  --base <chemin>  base SQLite à utiliser (par défaut : celle de l'application)
  --version        affiche la version
  --help           affiche cette aide

Ce programme est lancé par un client MCP (Hermes Agent) et dialogue sur l'entrée et la sortie standard.";

fn main() -> ExitCode {
    let mut base: Option<PathBuf> = std::env::var_os("TDB_MCP_BASE").map(PathBuf::from);
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--version") | Some("-V") => {
                println!("tdb-mcp {VERSION}");
                return ExitCode::SUCCESS;
            }
            Some("--help") | Some("-h") => {
                println!("{AIDE}");
                return ExitCode::SUCCESS;
            }
            Some("--base") => match args.next() {
                Some(chemin) => base = Some(PathBuf::from(chemin)),
                None => {
                    eprintln!("--base attend un chemin.");
                    return ExitCode::from(2);
                }
            },
            _ => {
                eprintln!("Argument inconnu : {}\n\n{AIDE}", arg.to_string_lossy());
                return ExitCode::from(2);
            }
        }
    }
    let Some(base) = base.or_else(chemin_base_par_defaut) else {
        eprintln!("Impossible de déterminer l'emplacement de la base : utilise --base <chemin>.");
        return ExitCode::from(2);
    };

    let journal = Journal::pour_base(&base);
    journal.ecrire(
        "INFO",
        &format!("Démarrage de tdb-mcp {VERSION} (base : {})", base.display()),
    );
    let mut serveur = Serveur::nouveau(base, journal);

    let entree = std::io::stdin().lock();
    let mut sortie = std::io::stdout().lock();
    for ligne in entree.lines() {
        let ligne = match ligne {
            Ok(l) => l,
            Err(e) => {
                serveur
                    .journal()
                    .ecrire("ERREUR", &format!("Lecture de l'entrée standard : {e}"));
                return ExitCode::FAILURE;
            }
        };
        if ligne.trim().is_empty() {
            continue;
        }
        if let Some(reponse) = serveur.traiter(&ligne) {
            if writeln!(sortie, "{reponse}")
                .and_then(|_| sortie.flush())
                .is_err()
            {
                // Le client a fermé la connexion.
                return ExitCode::SUCCESS;
            }
        }
    }
    serveur
        .journal()
        .ecrire("INFO", "Fin de session (entrée standard fermée)");
    ExitCode::SUCCESS
}
