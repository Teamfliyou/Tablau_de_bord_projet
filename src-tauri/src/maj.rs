//! Mise à jour automatique via `tauri-plugin-updater` et GitHub Releases.
//!
//! - Le manifeste `latest.json` et l'installateur sont téléchargés en HTTPS depuis GitHub.
//! - La signature minisign de l'installateur est vérifiée par Tauri avec la clé publique
//!   intégrée dans `tauri.conf.json` ; sans signature valide, rien n'est installé.
//! - Aucune version identique, antérieure ou préversion n'est jamais proposée.
//! - La progression exposée au frontend est celle du téléchargement réel (octets reçus).
//! - Le serveur MCP (`tdb-mcp.exe`) fait partie de l'installateur : il est mis à jour avec l'application.

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, State};
use tauri_plugin_updater::{Update, UpdaterExt};
use tdb_core::maj::{doit_proposer, installe_par_nsis};
use tdb_core::{Erreur, Resultat};
use tdb_shared::{InfoMiseAJour, PhaseMiseAJour, ProgressionMiseAJour, VerificationMiseAJour};

/// Mise à jour trouvée (en attente d'installation) et progression courante.
#[derive(Default)]
pub struct EtatMiseAJour {
    trouvee: Mutex<Option<Update>>,
    progression: Mutex<ProgressionMiseAJour>,
}

fn est_installation_nsis() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(installe_par_nsis))
        .unwrap_or(false)
}

fn erreur_updater(e: tauri_plugin_updater::Error) -> Erreur {
    use tauri_plugin_updater::Error as E;
    log::warn!("Mise à jour : {e}");
    match e {
        E::Reqwest(_) => Erreur::Reseau(
            "Impossible de contacter le serveur de mises à jour. Vérifie ta connexion Internet puis réessaie."
                .into(),
        ),
        E::Minisign(_) | E::Base64(_) | E::SignatureUtf8(_) => Erreur::Interne(
            "La signature de la mise à jour est invalide : l'installation a été refusée par sécurité."
                .into(),
        ),
        _ => Erreur::Interne(
            "La mise à jour a échoué. Réessaie dans quelques instants ; tes données ne sont pas modifiées."
                .into(),
        ),
    }
}

fn definir(maj: &EtatMiseAJour, phase: PhaseMiseAJour, telecharge: u64, total: Option<u64>) {
    if let Ok(mut p) = maj.progression.lock() {
        *p = ProgressionMiseAJour {
            phase,
            telecharge,
            total,
        };
    }
}

/// Dernière version publiée (numéro, date RFC 3339), relevée pendant la comparaison des versions.
type VersionPubliee = Arc<Mutex<Option<(String, Option<String>)>>>;

/// Date RFC 3339 annoncée par `latest.json`.
fn date_rfc3339(date: Option<time::OffsetDateTime>) -> Option<String> {
    date.and_then(|d| {
        d.format(&time::format_description::well_known::Rfc3339)
            .ok()
    })
}

/// Interroge GitHub Releases (réseau asynchrone : l'interface n'est jamais bloquée).
/// La dernière version publiée est renvoyée même lorsqu'elle n'est pas plus récente.
#[tauri::command]
pub async fn verifier_mise_a_jour(
    app: AppHandle,
    maj: State<'_, EtatMiseAJour>,
) -> Resultat<VerificationMiseAJour> {
    let publiee: VersionPubliee = Arc::default();
    let capture = Arc::clone(&publiee);
    let updater = app
        .updater_builder()
        .version_comparator(move |actuelle, distante| {
            if let Ok(mut p) = capture.lock() {
                *p = Some((
                    distante.version.to_string(),
                    date_rfc3339(distante.pub_date),
                ));
            }
            doit_proposer(&actuelle, &distante.version)
        })
        .build()
        .map_err(erreur_updater)?;
    let trouvee = updater.check().await.map_err(erreur_updater)?;
    let (derniere_version, date) = publiee
        .lock()
        .ok()
        .and_then(|p| p.clone())
        .map_or((None, None), |(v, d)| (Some(v), d));
    let mise_a_jour = trouvee.as_ref().map(|u| InfoMiseAJour {
        version_actuelle: u.current_version.clone(),
        version: u.version.clone(),
        notes: u.body.clone().filter(|n| !n.trim().is_empty()),
        installable: est_installation_nsis(),
        date: date_rfc3339(u.date),
    });
    if let Ok(mut t) = maj.trouvee.lock() {
        *t = trouvee;
    }
    Ok(VerificationMiseAJour {
        version_actuelle: app.package_info().version.to_string(),
        derniere_version,
        date,
        mise_a_jour,
    })
}

/// Télécharge, vérifie la signature puis lance l'installateur (l'application se ferme et redémarre).
#[tauri::command]
pub async fn installer_mise_a_jour(maj: State<'_, EtatMiseAJour>) -> Resultat<()> {
    if !est_installation_nsis() {
        return Err(Erreur::Validation(
            "La mise à jour automatique n'est disponible que pour la version installée. Télécharge le nouvel installateur depuis GitHub Releases."
                .into(),
        ));
    }
    let update = maj
        .trouvee
        .lock()
        .ok()
        .and_then(|t| t.clone())
        .ok_or_else(|| {
            Erreur::Validation("Aucune mise à jour à installer : relance la recherche.".into())
        })?;

    definir(&maj, PhaseMiseAJour::Telechargement, 0, None);
    let mut recu: u64 = 0;
    let octets = update
        .download(
            |morceau, total| {
                recu += morceau as u64;
                definir(&maj, PhaseMiseAJour::Telechargement, recu, total);
            },
            || {},
        )
        .await
        .map_err(|e| {
            definir(&maj, PhaseMiseAJour::Inactif, 0, None);
            erreur_updater(e)
        })?;

    // `install` revérifie la signature avant de lancer l'installateur NSIS (mode passif).
    definir(&maj, PhaseMiseAJour::Installation, recu, Some(recu));
    // Un serveur MCP lancé par Hermes verrouillerait tdb-mcp.exe pendant l'installation.
    crate::mcp::arreter_serveurs_mcp();
    update.install(octets).map_err(|e| {
        definir(&maj, PhaseMiseAJour::Inactif, 0, None);
        erreur_updater(e)
    })
}

#[tauri::command]
pub fn progression_mise_a_jour(maj: State<'_, EtatMiseAJour>) -> ProgressionMiseAJour {
    maj.progression
        .lock()
        .map(|p| p.clone())
        .unwrap_or_default()
}
