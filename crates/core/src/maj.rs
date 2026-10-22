//! Règles de la mise à jour automatique, indépendantes de Tauri (donc testables partout).

use std::path::Path;

use semver::Version;

/// Une mise à jour n'est proposée que si elle est strictement plus récente et stable.
pub fn doit_proposer(actuelle: &Version, distante: &Version) -> bool {
    distante > actuelle && distante.pre.is_empty()
}

/// L'installateur NSIS dépose `uninstall.exe` à côté de l'exécutable ; l'exécutable portable n'a rien de tel.
/// Lancer l'installateur depuis un exécutable portable créerait une seconde installation : on refuse.
pub fn installe_par_nsis(dossier_exe: &Path) -> bool {
    dossier_exe.join("uninstall.exe").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn propose_une_version_superieure() {
        assert!(doit_proposer(&v("4.2.0"), &v("4.2.1")));
        assert!(doit_proposer(&v("4.2.9"), &v("4.10.0")));
    }

    #[test]
    fn refuse_version_identique_ou_anterieure() {
        assert!(!doit_proposer(&v("4.2.1"), &v("4.2.1")));
        assert!(!doit_proposer(&v("4.2.1"), &v("4.2.0")));
        assert!(!doit_proposer(&v("4.2.0"), &v("4.1.0")));
    }

    #[test]
    fn refuse_les_preversions() {
        assert!(!doit_proposer(&v("4.2.0"), &v("4.3.0-beta.1")));
    }

    #[test]
    fn detecte_installation_nsis_et_portable() {
        let dossier = std::env::temp_dir().join(format!("tdb-test-maj-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).unwrap();
        assert!(!installe_par_nsis(&dossier));
        std::fs::write(dossier.join("uninstall.exe"), b"").unwrap();
        assert!(installe_par_nsis(&dossier));
        std::fs::remove_dir_all(&dossier).unwrap();
    }
}
