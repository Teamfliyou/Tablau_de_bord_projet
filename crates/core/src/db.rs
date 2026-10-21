//! Base SQLite locale : migrations versionnées, sauvegardes, devoirs, paramètres, cache ICS.

use crate::error::{validation, Erreur, Resultat};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

pub use tdb_shared::{
    valider_echeance, valider_matiere, valider_statut, valider_titre, valider_type, Devoir,
    DevoirEntree, DevoirModification, RapportMigration, Tag, TagEntree, IMPORT_MAX,
    IMPORT_MAX_OCTETS, STATUTS, TYPES,
};
const SAUVEGARDES_CONSERVEES: usize = 7;

/// Migrations appliquées dans l'ordre ; la version courante est stockée dans `PRAGMA user_version`.
const MIGRATIONS: &[&str] = &[
    // v1 : schéma initial (équivalent monoposte du schéma SQLAlchemy historique)
    "CREATE TABLE devoirs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        titre TEXT NOT NULL,
        matiere TEXT NOT NULL DEFAULT '',
        echeance TEXT NOT NULL DEFAULT '',
        type TEXT NOT NULL DEFAULT 'devoir',
        statut TEXT NOT NULL DEFAULT 'a_faire'
    );
    CREATE TABLE parametres (cle TEXT PRIMARY KEY, valeur TEXT NOT NULL);
    CREATE TABLE cache_ics (
        url TEXT PRIMARY KEY,
        contenu TEXT NOT NULL,
        recupere_le TEXT NOT NULL
    );",
    // v2 : tags personnalisés (ajout seul, aucune donnée existante modifiée).
    "CREATE TABLE tags (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        nom TEXT NOT NULL UNIQUE COLLATE NOCASE,
        couleur TEXT NOT NULL DEFAULT '#6366f1'
    );
    CREATE TABLE devoir_tags (
        devoir_id INTEGER NOT NULL REFERENCES devoirs (id) ON DELETE CASCADE,
        tag_id INTEGER NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
        PRIMARY KEY (devoir_id, tag_id)
    );",
];

/// Paramètres de l'ancienne base trouvés dans `users` (le secret est renvoyé séparément).
#[derive(Debug, Default)]
pub struct AncienneConfig {
    pub username: String,
    pub ade_url: String,
    pub gemini_key: String,
}

pub struct Db {
    conn: Mutex<Connection>,
    chemin: Option<PathBuf>,
}

impl Db {
    /// Ouvre (ou crée) la base. Une base illisible est mise de côté puis restaurée
    /// depuis la dernière sauvegarde ; aucune donnée n'est jamais supprimée.
    pub fn ouvrir(chemin: &Path) -> Resultat<Db> {
        if let Some(parent) = chemin.parent() {
            fs::create_dir_all(parent).map_err(io_err)?;
        }
        if chemin.exists() && !Self::base_saine(chemin) {
            Self::mettre_de_cote(chemin)?;
        }
        let existait = chemin.exists();
        let conn = Connection::open(chemin)?;
        configurer(&conn)?;
        let db = Db {
            conn: Mutex::new(conn),
            chemin: Some(chemin.to_path_buf()),
        };
        let version = db.version()?;
        if existait && version > 0 && (version as usize) < MIGRATIONS.len() {
            db.sauvegarder(&format!("avant-migration-v{version}"))?;
        }
        db.migrer()?;
        Ok(db)
    }

    pub fn en_memoire() -> Resultat<Db> {
        let conn = Connection::open_in_memory()?;
        configurer(&conn)?;
        let db = Db {
            conn: Mutex::new(conn),
            chemin: None,
        };
        db.migrer()?;
        Ok(db)
    }

    fn base_saine(chemin: &Path) -> bool {
        let Ok(conn) = Connection::open_with_flags(chemin, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
            return false;
        };
        conn.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .map(|r| r == "ok")
            .unwrap_or(false)
    }

    fn mettre_de_cote(chemin: &Path) -> Resultat<()> {
        let horodatage = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let corrompue = chemin.with_file_name(format!("corrompue-{horodatage}.db"));
        fs::rename(chemin, &corrompue).map_err(io_err)?;
        for ext in ["-wal", "-shm"] {
            let mut nom = chemin.as_os_str().to_owned();
            nom.push(ext);
            let _ = fs::remove_file(PathBuf::from(nom));
        }
        log::error!("Base illisible : conservée dans {}", corrompue.display());
        if let Some(derniere) = dernieres_sauvegardes(&dossier_sauvegardes(chemin))
            .into_iter()
            .next()
        {
            fs::copy(&derniere, chemin).map_err(io_err)?;
            log::warn!("Restauration depuis {}", derniere.display());
        }
        Ok(())
    }

    fn verrou(&self) -> Resultat<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| Erreur::Interne("Verrou de base de données indisponible.".into()))
    }

    pub fn version(&self) -> Resultat<i64> {
        Ok(self
            .verrou()?
            .query_row("PRAGMA user_version", [], |r| r.get(0))?)
    }

    fn migrer(&self) -> Resultat<()> {
        let mut conn = self.verrou()?;
        let courante: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if courante as usize > MIGRATIONS.len() {
            return Err(Erreur::Interne(
                "La base a été créée par une version plus récente de l'application.".into(),
            ));
        }
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(courante as usize) {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", (i + 1) as i64)?;
            tx.commit()?;
        }
        Ok(())
    }

    /// Copie cohérente de la base (`VACUUM INTO`), avec rotation des anciennes sauvegardes.
    pub fn sauvegarder(&self, etiquette: &str) -> Resultat<Option<PathBuf>> {
        let Some(chemin) = &self.chemin else {
            return Ok(None);
        };
        let dossier = dossier_sauvegardes(chemin);
        fs::create_dir_all(&dossier).map_err(io_err)?;
        let horodatage = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let cible = dossier.join(format!("{etiquette}-{horodatage}.db"));
        if cible.exists() {
            return Ok(Some(cible));
        }
        self.verrou()?
            .execute("VACUUM INTO ?1", [cible.to_string_lossy().as_ref()])?;
        for ancienne in dernieres_sauvegardes(&dossier)
            .into_iter()
            .skip(SAUVEGARDES_CONSERVEES)
        {
            let _ = fs::remove_file(ancienne);
        }
        Ok(Some(cible))
    }

    /// Une sauvegarde par jour au plus (`auto-AAAAMMJJ.db`), créée au démarrage.
    pub fn sauvegarde_quotidienne(&self) -> Resultat<()> {
        let Some(chemin) = &self.chemin else {
            return Ok(());
        };
        let dossier = dossier_sauvegardes(chemin);
        let cible = dossier.join(format!("auto-{}.db", chrono::Local::now().format("%Y%m%d")));
        if cible.exists() {
            return Ok(());
        }
        fs::create_dir_all(&dossier).map_err(io_err)?;
        self.verrou()?
            .execute("VACUUM INTO ?1", [cible.to_string_lossy().as_ref()])?;
        for ancienne in dernieres_sauvegardes(&dossier)
            .into_iter()
            .skip(SAUVEGARDES_CONSERVEES)
        {
            let _ = fs::remove_file(ancienne);
        }
        Ok(())
    }

    // ---------------------------------------------------------------- Devoirs

    pub fn lister_devoirs(&self) -> Resultat<Vec<Devoir>> {
        let conn = self.verrou()?;
        let mut stmt = conn.prepare(
            "SELECT id, titre, matiere, echeance, type, statut FROM devoirs ORDER BY id",
        )?;
        let mut lignes = stmt
            .query_map([], ligne_devoir)?
            .collect::<Result<Vec<_>, _>>()?;
        for d in &mut lignes {
            d.tags = tags_du_devoir(&conn, d.id)?;
        }
        Ok(lignes)
    }

    pub fn creer_devoir(&self, entree: DevoirEntree) -> Resultat<Devoir> {
        let e = entree.valider()?;
        let mut conn = self.verrou()?;
        let tx = conn.transaction()?;
        let id = inserer_devoir(&tx, &e)?;
        let tags = tags_du_devoir(&tx, id)?;
        tx.commit()?;
        Ok(Devoir {
            id,
            titre: e.titre,
            matiere: e.matiere,
            echeance: e.echeance,
            type_: e.type_,
            statut: e.statut,
            tags,
        })
    }

    pub fn modifier_devoir(&self, id: i64, m: DevoirModification) -> Resultat<Devoir> {
        let conn = self.verrou()?;
        let d = conn
            .query_row(
                "SELECT id, titre, matiere, echeance, type, statut FROM devoirs WHERE id = ?1",
                [id],
                ligne_devoir,
            )
            .optional()?
            .ok_or_else(|| Erreur::Introuvable("Devoir introuvable.".into()))?;
        let mut d = m.appliquer(d)?;
        conn.execute(
            "UPDATE devoirs SET titre=?2, matiere=?3, echeance=?4, type=?5, statut=?6 WHERE id=?1",
            params![d.id, d.titre, d.matiere, d.echeance, d.type_, d.statut],
        )?;
        d.tags = tags_du_devoir(&conn, id)?;
        Ok(d)
    }

    pub fn supprimer_devoir(&self, id: i64) -> Resultat<()> {
        let n = self
            .verrou()?
            .execute("DELETE FROM devoirs WHERE id = ?1", [id])?;
        if n == 0 {
            return Err(Erreur::Introuvable("Devoir introuvable.".into()));
        }
        Ok(())
    }

    // ------------------------------------------------------------------ Tags

    pub fn lister_tags(&self) -> Resultat<Vec<Tag>> {
        let conn = self.verrou()?;
        let mut stmt = conn.prepare("SELECT id, nom, couleur FROM tags ORDER BY nom")?;
        let lignes = stmt
            .query_map([], ligne_tag)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(lignes)
    }

    pub fn creer_tag(&self, entree: TagEntree) -> Resultat<Tag> {
        let e = entree.valider()?;
        let conn = self.verrou()?;
        if nom_tag_pris(&conn, &e.nom, None)? {
            return validation("Un tag porte déjà ce nom.");
        }
        conn.execute(
            "INSERT INTO tags (nom, couleur) VALUES (?1, ?2)",
            params![e.nom, e.couleur],
        )?;
        Ok(Tag {
            id: conn.last_insert_rowid(),
            nom: e.nom,
            couleur: e.couleur,
        })
    }

    pub fn modifier_tag(&self, id: i64, entree: TagEntree) -> Resultat<Tag> {
        let e = entree.valider()?;
        let conn = self.verrou()?;
        if nom_tag_pris(&conn, &e.nom, Some(id))? {
            return validation("Un tag porte déjà ce nom.");
        }
        let n = conn.execute(
            "UPDATE tags SET nom = ?2, couleur = ?3 WHERE id = ?1",
            params![id, e.nom, e.couleur],
        )?;
        if n == 0 {
            return Err(Erreur::Introuvable("Tag introuvable.".into()));
        }
        Ok(Tag {
            id,
            nom: e.nom,
            couleur: e.couleur,
        })
    }

    /// Supprime le tag et ses associations ; les devoirs restent intacts.
    pub fn supprimer_tag(&self, id: i64) -> Resultat<()> {
        let n = self
            .verrou()?
            .execute("DELETE FROM tags WHERE id = ?1", [id])?;
        if n == 0 {
            return Err(Erreur::Introuvable("Tag introuvable.".into()));
        }
        Ok(())
    }

    /// Remplace l'ensemble des tags d'un devoir (ajout et retrait en une transaction).
    pub fn definir_tags_devoir(&self, id: i64, tags: Vec<i64>) -> Resultat<Devoir> {
        if tags.len() > tdb_shared::TAGS_PAR_DEVOIR_MAX {
            return validation("Un devoir ne peut pas avoir plus de 20 tags.");
        }
        let mut conn = self.verrou()?;
        let tx = conn.transaction()?;
        let mut devoir = tx
            .query_row(
                "SELECT id, titre, matiere, echeance, type, statut FROM devoirs WHERE id = ?1",
                [id],
                ligne_devoir,
            )
            .optional()?
            .ok_or_else(|| Erreur::Introuvable("Devoir introuvable.".into()))?;
        tx.execute("DELETE FROM devoir_tags WHERE devoir_id = ?1", [id])?;
        for tag_id in tags {
            let existe: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM tags WHERE id = ?1)",
                [tag_id],
                |r| r.get(0),
            )?;
            if !existe {
                return Err(Erreur::Introuvable("Tag introuvable.".into()));
            }
            tx.execute(
                "INSERT OR IGNORE INTO devoir_tags (devoir_id, tag_id) VALUES (?1, ?2)",
                params![id, tag_id],
            )?;
        }
        devoir.tags = tags_du_devoir(&tx, id)?;
        tx.commit()?;
        Ok(devoir)
    }

    /// Export au format historique (sans identifiant) ; les tags sont exportés par nom.
    pub fn exporter_devoirs(&self) -> Resultat<Vec<DevoirEntree>> {
        Ok(self
            .lister_devoirs()?
            .into_iter()
            .map(|d| DevoirEntree {
                titre: d.titre,
                matiere: d.matiere,
                echeance: d.echeance,
                type_: d.type_,
                statut: d.statut,
                tags: d
                    .tags
                    .into_iter()
                    .map(|t| TagEntree {
                        nom: t.nom,
                        couleur: t.couleur,
                    })
                    .collect(),
            })
            .collect())
    }

    /// Importe un JSON de devoirs : tout est validé avant la moindre écriture (transaction).
    pub fn importer_devoirs_json(&self, contenu: &str) -> Resultat<usize> {
        if contenu.len() as u64 > IMPORT_MAX_OCTETS {
            return validation("Fichier trop volumineux (2 Mo maximum).");
        }
        let brut: Vec<serde_json::Value> = serde_json::from_str(contenu).map_err(|_| {
            Erreur::Validation("Fichier JSON invalide : une liste de devoirs est attendue.".into())
        })?;
        if brut.len() > IMPORT_MAX {
            return validation("Import limité à 500 devoirs par fichier.");
        }
        let mut entrees = Vec::with_capacity(brut.len());
        for (i, v) in brut.into_iter().enumerate() {
            let e: DevoirEntree = serde_json::from_value(v).map_err(|_| {
                Erreur::Validation(format!("Devoir n°{} invalide dans le fichier.", i + 1))
            })?;
            entrees.push(
                e.valider()
                    .map_err(|err| Erreur::Validation(format!("Devoir n°{} : {err}", i + 1)))?,
            );
        }
        self.inserer_lot(&entrees)
    }

    fn inserer_lot(&self, entrees: &[DevoirEntree]) -> Resultat<usize> {
        let mut conn = self.verrou()?;
        let tx = conn.transaction()?;
        for e in entrees {
            inserer_devoir(&tx, e)?;
        }
        tx.commit()?;
        Ok(entrees.len())
    }

    // ------------------------------------------------------------- Paramètres

    pub fn lire_parametre(&self, cle: &str) -> Resultat<Option<String>> {
        Ok(self
            .verrou()?
            .query_row("SELECT valeur FROM parametres WHERE cle = ?1", [cle], |r| {
                r.get(0)
            })
            .optional()?)
    }

    pub fn ecrire_parametre(&self, cle: &str, valeur: &str) -> Resultat<()> {
        self.verrou()?.execute(
            "INSERT INTO parametres (cle, valeur) VALUES (?1, ?2)
             ON CONFLICT(cle) DO UPDATE SET valeur = excluded.valeur",
            params![cle, valeur],
        )?;
        Ok(())
    }

    // -------------------------------------------------------------- Cache ICS

    pub fn enregistrer_cache_ics(
        &self,
        url: &str,
        contenu: &str,
        recupere_le: &str,
    ) -> Resultat<()> {
        let conn = self.verrou()?;
        // Un seul flux conservé : le dernier emploi du temps récupéré (consultation hors ligne).
        conn.execute("DELETE FROM cache_ics WHERE url <> ?1", [url])?;
        conn.execute(
            "INSERT INTO cache_ics (url, contenu, recupere_le) VALUES (?1,?2,?3)
             ON CONFLICT(url) DO UPDATE SET contenu = excluded.contenu, recupere_le = excluded.recupere_le",
            params![url, contenu, recupere_le],
        )?;
        Ok(())
    }

    pub fn lire_cache_ics(&self, url: &str) -> Resultat<Option<(String, String)>> {
        Ok(self
            .verrou()?
            .query_row(
                "SELECT contenu, recupere_le FROM cache_ics WHERE url = ?1",
                [url],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?)
    }

    // -------------------------------------------- Migration de l'ancienne base

    /// Importe les devoirs (et paramètres) d'une ancienne base `app.db` ouverte en lecture seule.
    /// Les doublons exacts sont ignorés ; l'ancienne base n'est jamais modifiée.
    pub fn migrer_ancienne_base(
        &self,
        ancienne: &Path,
    ) -> Resultat<(RapportMigration, AncienneConfig)> {
        let src = Connection::open_with_flags(ancienne, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(
            |_| Erreur::Validation("Impossible d'ouvrir l'ancienne base SQLite.".into()),
        )?;
        let a_table = |nom: &str| -> bool {
            src.query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
                [nom],
                |_| Ok(()),
            )
            .is_ok()
        };
        if !a_table("devoirs") {
            return validation(
                "Ce fichier ne ressemble pas à une base du Tableau de bord (table « devoirs » absente).",
            );
        }
        let colonnes_de = |table: &str| -> Resultat<Vec<String>> {
            Ok(src
                .prepare(&format!("PRAGMA table_info({table})"))?
                .query_map([], |r| r.get::<_, String>(1))?
                .collect::<Result<_, _>>()?)
        };
        let colonnes = colonnes_de("devoirs")?;
        let col = |nom: &str, defaut: &str| {
            if colonnes.iter().any(|c| c == nom) {
                nom.to_string()
            } else {
                format!("'{defaut}'")
            }
        };
        let sql = format!(
            "SELECT {}, {}, {}, {}, {} FROM devoirs",
            col("titre", ""),
            col("matiere", ""),
            col("echeance", ""),
            col("type", "devoir"),
            col("statut", "a_faire")
        );
        let anciens: Vec<[String; 5]> = src
            .prepare(&sql)?
            .query_map([], |r| {
                let mut v: [String; 5] = Default::default();
                for (i, cellule) in v.iter_mut().enumerate() {
                    *cellule = r.get::<_, Option<String>>(i)?.unwrap_or_default();
                }
                Ok(v)
            })?
            .collect::<Result<_, _>>()?;

        // Les tags ne font pas partie de la comparaison de doublons (anciennes bases sans tags).
        let existants: Vec<DevoirEntree> = self
            .exporter_devoirs()?
            .into_iter()
            .map(|mut e| {
                e.tags.clear();
                e
            })
            .collect();
        let (mut nouveaux, mut ignores) = (Vec::<DevoirEntree>::new(), 0usize);
        for [titre, matiere, echeance, type_, statut] in anciens {
            let e = DevoirEntree {
                titre,
                matiere,
                echeance,
                type_: if TYPES.contains(&type_.as_str()) {
                    type_
                } else {
                    "devoir".into()
                },
                statut: if STATUTS.contains(&statut.as_str()) {
                    statut
                } else {
                    "a_faire".into()
                },
                tags: Vec::new(),
            };
            match e.valider() {
                Ok(e) if !existants.contains(&e) && !nouveaux.contains(&e) => nouveaux.push(e),
                _ => ignores += 1,
            }
        }
        let devoirs_importes = self.inserer_lot(&nouveaux)?;

        let mut config = AncienneConfig::default();
        if a_table("users") {
            let cu = colonnes_de("users")?;
            let c = |nom: &str| {
                if cu.iter().any(|x| x == nom) {
                    nom.to_string()
                } else {
                    "''".to_string()
                }
            };
            let sql = format!(
                "SELECT {}, {}, {} FROM users ORDER BY id LIMIT 1",
                c("username"),
                c("ade_ics_url"),
                c("gemini_api_key")
            );
            let ligne = src
                .query_row(&sql, [], |r| {
                    Ok((
                        r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                        r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                        r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    ))
                })
                .optional()?;
            if let Some((username, ade_url, gemini_key)) = ligne {
                config = AncienneConfig {
                    username,
                    ade_url,
                    gemini_key,
                };
            }
        }
        let mut parametres_importes = false;
        for (cle, valeur) in [("username", &config.username), ("ade_url", &config.ade_url)] {
            if !valeur.is_empty() && self.lire_parametre(cle)?.is_none() {
                self.ecrire_parametre(cle, valeur)?;
                parametres_importes = true;
            }
        }
        let rapport = RapportMigration {
            devoirs_importes,
            devoirs_ignores: ignores,
            parametres_importes,
            cle_gemini_trouvee: !config.gemini_key.is_empty(),
        };
        Ok((rapport, config))
    }
}

fn configurer(conn: &Connection) -> Resultat<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(())
}

fn ligne_devoir(r: &rusqlite::Row<'_>) -> rusqlite::Result<Devoir> {
    Ok(Devoir {
        id: r.get(0)?,
        titre: r.get(1)?,
        matiere: r.get(2)?,
        echeance: r.get(3)?,
        type_: r.get(4)?,
        statut: r.get(5)?,
        tags: Vec::new(),
    })
}

fn ligne_tag(r: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: r.get(0)?,
        nom: r.get(1)?,
        couleur: r.get(2)?,
    })
}

/// Insère un devoir et ses tags (créés au besoin, par nom sans tenir compte de la casse).
fn inserer_devoir(conn: &Connection, e: &DevoirEntree) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO devoirs (titre, matiere, echeance, type, statut) VALUES (?1,?2,?3,?4,?5)",
        params![e.titre, e.matiere, e.echeance, e.type_, e.statut],
    )?;
    let id = conn.last_insert_rowid();
    for t in &e.tags {
        conn.execute(
            "INSERT OR IGNORE INTO tags (nom, couleur) VALUES (?1, ?2)",
            params![t.nom, t.couleur],
        )?;
        let tag_id: i64 = conn.query_row("SELECT id FROM tags WHERE nom = ?1", [&t.nom], |r| {
            r.get(0)
        })?;
        conn.execute(
            "INSERT OR IGNORE INTO devoir_tags (devoir_id, tag_id) VALUES (?1, ?2)",
            params![id, tag_id],
        )?;
    }
    Ok(id)
}

fn tags_du_devoir(conn: &Connection, devoir_id: i64) -> rusqlite::Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.nom, t.couleur FROM devoir_tags dt
         JOIN tags t ON t.id = dt.tag_id
         WHERE dt.devoir_id = ?1 ORDER BY t.nom",
    )?;
    let lignes = stmt
        .query_map([devoir_id], ligne_tag)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(lignes)
}

fn nom_tag_pris(conn: &Connection, nom: &str, exclu: Option<i64>) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM tags WHERE nom = ?1 AND id IS NOT ?2)",
        params![nom, exclu],
        |r| r.get(0),
    )
}

fn io_err(e: std::io::Error) -> Erreur {
    Erreur::Interne(format!("Erreur d'accès aux fichiers : {e}"))
}

fn dossier_sauvegardes(chemin_base: &Path) -> PathBuf {
    chemin_base
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("sauvegardes")
}

/// Sauvegardes triées de la plus récente à la plus ancienne.
fn dernieres_sauvegardes(dossier: &Path) -> Vec<PathBuf> {
    let mut v: Vec<(std::time::SystemTime, PathBuf)> = fs::read_dir(dossier)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "db"))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    v.sort_by_key(|a| std::cmp::Reverse(a.0));
    v.into_iter().map(|(_, p)| p).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entree(titre: &str) -> DevoirEntree {
        DevoirEntree {
            titre: titre.into(),
            matiere: "MATHS".into(),
            echeance: "2026-10-05".into(),
            type_: "ds".into(),
            statut: "a_faire".into(),
            tags: Vec::new(),
        }
    }

    fn dossier_temp(nom: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("tdb-test-{nom}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn crud_devoirs() {
        let db = Db::en_memoire().unwrap();
        let d = db.creer_devoir(entree("  Algèbre  ")).unwrap();
        assert_eq!(d.titre, "Algèbre");
        let m = db
            .modifier_devoir(
                d.id,
                DevoirModification {
                    statut: Some("termine".into()),
                    titre: Some("Algèbre 2".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            (m.statut.as_str(), m.titre.as_str()),
            ("termine", "Algèbre 2")
        );
        assert_eq!(db.lister_devoirs().unwrap().len(), 1);
        db.supprimer_devoir(d.id).unwrap();
        assert!(matches!(
            db.supprimer_devoir(d.id),
            Err(Erreur::Introuvable(_))
        ));
    }

    #[test]
    fn validation_rejette_les_valeurs_invalides() {
        let db = Db::en_memoire().unwrap();
        assert!(db.creer_devoir(entree("")).is_err());
        let mut e = entree("x");
        e.echeance = "05/10/2026".into();
        assert!(db.creer_devoir(e).is_err());
        let mut e = entree("x");
        e.statut = "fini".into();
        assert!(db.creer_devoir(e).is_err());
        assert!(db
            .modifier_devoir(999, DevoirModification::default())
            .is_err());
    }

    #[test]
    fn import_export_aller_retour() {
        let db = Db::en_memoire().unwrap();
        db.creer_devoir(entree("A")).unwrap();
        db.creer_devoir(entree("B")).unwrap();
        let json = serde_json::to_string(&db.exporter_devoirs().unwrap()).unwrap();
        let autre = Db::en_memoire().unwrap();
        assert_eq!(autre.importer_devoirs_json(&json).unwrap(), 2);
        assert_eq!(
            autre.exporter_devoirs().unwrap(),
            db.exporter_devoirs().unwrap()
        );
    }

    #[test]
    fn import_atomique_et_borne() {
        let db = Db::en_memoire().unwrap();
        let json = r#"[{"titre":"ok"},{"titre":"","statut":"a_faire"}]"#;
        assert!(db.importer_devoirs_json(json).is_err());
        assert!(
            db.lister_devoirs().unwrap().is_empty(),
            "aucun devoir ne doit être écrit"
        );
        assert!(db.importer_devoirs_json("{\"a\":1}").is_err());
        let trop: Vec<_> = (0..501)
            .map(|i| serde_json::json!({"titre": format!("d{i}")}))
            .collect();
        assert!(db
            .importer_devoirs_json(&serde_json::to_string(&trop).unwrap())
            .is_err());
        assert_eq!(
            db.importer_devoirs_json(r#"[{"titre":"défaut"}]"#).unwrap(),
            1
        );
        assert_eq!(db.lister_devoirs().unwrap()[0].statut, "a_faire");
    }

    #[test]
    fn donnees_conservees_entre_redemarrages() {
        let dir = dossier_temp("persist");
        let chemin = dir.join("tableau.db");
        {
            let db = Db::ouvrir(&chemin).unwrap();
            db.creer_devoir(entree("Persistant")).unwrap();
            db.ecrire_parametre("username", "Zayd").unwrap();
        }
        let db = Db::ouvrir(&chemin).unwrap();
        assert_eq!(db.lister_devoirs().unwrap()[0].titre, "Persistant");
        assert_eq!(
            db.lire_parametre("username").unwrap().as_deref(),
            Some("Zayd")
        );
        assert_eq!(db.version().unwrap() as usize, MIGRATIONS.len());
    }

    #[test]
    fn sauvegarde_et_restauration_apres_corruption() {
        let dir = dossier_temp("corrupt");
        let chemin = dir.join("tableau.db");
        {
            let db = Db::ouvrir(&chemin).unwrap();
            db.creer_devoir(entree("Sauvé")).unwrap();
            assert!(db.sauvegarder("auto").unwrap().is_some());
        }
        fs::write(
            &chemin,
            b"ceci n'est pas une base sqlite du tout, juste du bruit",
        )
        .unwrap();
        let db = Db::ouvrir(&chemin).unwrap();
        assert_eq!(db.lister_devoirs().unwrap()[0].titre, "Sauvé");
        let conservee = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .any(|e| e.file_name().to_string_lossy().starts_with("corrompue-"));
        assert!(
            conservee,
            "la base corrompue doit être conservée, pas supprimée"
        );
    }

    #[test]
    fn migration_ancienne_base_sans_modifier_la_source() {
        let dir = dossier_temp("legacy");
        let ancienne = dir.join("app.db");
        {
            let c = Connection::open(&ancienne).unwrap();
            c.execute_batch(
                "CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT, password_hash TEXT, username TEXT, ade_ics_url TEXT, gemini_api_key TEXT);
                 INSERT INTO users VALUES (1,'a@b.fr','x','Léa','https://ade.example.com/x.ics','AIzaSECRET');
                 CREATE TABLE devoirs (id INTEGER PRIMARY KEY, user_id INTEGER, titre TEXT, matiere TEXT, echeance TEXT, type TEXT, statut TEXT);
                 INSERT INTO devoirs VALUES (1,1,'TP1','PHYS','2026-10-10','devoir','en_cours');
                 INSERT INTO devoirs VALUES (2,1,'TP1','PHYS','2026-10-10','devoir','en_cours');
                 INSERT INTO devoirs VALUES (3,1,'DS1','MATHS','','bizarre','nimporte');",
            )
            .unwrap();
        }
        let avant = fs::read(&ancienne).unwrap();
        let db = Db::en_memoire().unwrap();
        let (rapport, cfg) = db.migrer_ancienne_base(&ancienne).unwrap();
        assert_eq!((rapport.devoirs_importes, rapport.devoirs_ignores), (2, 1));
        assert!(rapport.parametres_importes && rapport.cle_gemini_trouvee);
        assert_eq!(cfg.gemini_key, "AIzaSECRET");
        assert_eq!(
            db.lire_parametre("username").unwrap().as_deref(),
            Some("Léa")
        );
        let ds = db
            .lister_devoirs()
            .unwrap()
            .into_iter()
            .find(|d| d.titre == "DS1")
            .unwrap();
        assert_eq!(
            (ds.type_.as_str(), ds.statut.as_str()),
            ("devoir", "a_faire")
        );
        assert_eq!(
            fs::read(&ancienne).unwrap(),
            avant,
            "l'ancienne base ne doit pas être modifiée"
        );
        // Relancer la migration ne duplique rien.
        let (r2, _) = db.migrer_ancienne_base(&ancienne).unwrap();
        assert_eq!(r2.devoirs_importes, 0);
    }

    #[test]
    fn tags_crud_association_et_import_export() {
        let db = Db::en_memoire().unwrap();
        let d = db.creer_devoir(entree("Projet")).unwrap();
        let cm = db
            .creer_tag(TagEntree {
                nom: "CM".into(),
                couleur: "#FF0000".into(),
            })
            .unwrap();
        assert!(db
            .creer_tag(TagEntree {
                nom: "cm".into(),
                couleur: "#00ff00".into(),
            })
            .is_err());
        let urgent = db
            .creer_tag(TagEntree {
                nom: "Urgent".into(),
                couleur: "#123456".into(),
            })
            .unwrap();
        let m = db.definir_tags_devoir(d.id, vec![cm.id, urgent.id]).unwrap();
        assert_eq!(m.tags.len(), 2);
        let m = db.definir_tags_devoir(d.id, vec![urgent.id]).unwrap();
        assert_eq!(m.tags, vec![urgent.clone()]);
        assert!(db.definir_tags_devoir(d.id, vec![999]).is_err());
        db.definir_tags_devoir(d.id, vec![cm.id]).unwrap();
        let renomme = db
            .modifier_tag(
                cm.id,
                TagEntree {
                    nom: "Cours".into(),
                    couleur: "#00ff00".into(),
                },
            )
            .unwrap();
        assert_eq!(renomme.nom, "Cours");
        // Export avec tags, puis import dans une base vierge (tags recréés).
        let json = serde_json::to_string(&db.exporter_devoirs().unwrap()).unwrap();
        let autre = Db::en_memoire().unwrap();
        autre.importer_devoirs_json(&json).unwrap();
        assert_eq!(autre.lister_devoirs().unwrap()[0].tags[0].nom, "Cours");
        // Ancien format sans tags : accepté.
        assert_eq!(
            autre
                .importer_devoirs_json(r#"[{"titre":"sans tags"}]"#)
                .unwrap(),
            1
        );
        // Supprimer un tag ne supprime pas le devoir.
        db.supprimer_tag(cm.id).unwrap();
        let apres = db.lister_devoirs().unwrap();
        assert_eq!(apres.len(), 1);
        assert!(apres[0].tags.is_empty());
    }

    #[test]
    fn cache_ics_conserve_le_dernier_flux() {
        let db = Db::en_memoire().unwrap();
        db.enregistrer_cache_ics("https://a", "AAA", "t1").unwrap();
        db.enregistrer_cache_ics("https://b", "BBB", "t2").unwrap();
        assert!(db.lire_cache_ics("https://a").unwrap().is_none());
        assert_eq!(db.lire_cache_ics("https://b").unwrap().unwrap().0, "BBB");
    }
}
