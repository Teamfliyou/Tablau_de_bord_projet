//! Stockage SQLite multi-utilisateurs (compatible avec la base `app.db` de l'ancienne version Python).
//!
//! Stratégie de migration : la version du schéma est portée par `PRAGMA user_version`.
//! Une base Python existante (`user_version = 0` avec des tables) est d'abord sauvegardée
//! (`VACUUM INTO`), puis complétée de façon additive : aucune ligne n'est jamais supprimée.

use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tdb_core::{Erreur, Resultat};
use tdb_shared::{Devoir, DevoirEntree, DevoirModification};

#[derive(Debug, Clone)]
pub struct Utilisateur {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub username: String,
    pub ade_url: String,
    /// Clé Gemini telle que stockée (chiffrée ou, pour une ancienne base, en clair).
    pub gemini_key: String,
    pub gemini_model: String,
    pub preferences: String,
    pub created_at: Option<String>,
}

pub struct MajConfig<'a> {
    pub username: &'a str,
    pub ade_url: &'a str,
    pub gemini_model: Option<&'a str>,
    /// `None` : clé inchangée ; `Some("")` : suppression ; `Some(valeur)` : nouvelle valeur chiffrée.
    pub gemini_key: Option<&'a str>,
}

pub struct Store {
    conn: Mutex<Connection>,
    chemin: Option<PathBuf>,
}

const COLONNES_UTILISATEUR: &str = "id, email, password_hash, COALESCE(username,''), \
     COALESCE(ade_ics_url,''), COALESCE(gemini_api_key,''), COALESCE(gemini_model,''), \
     COALESCE(preferences,'{}'), created_at";

fn utilisateur(r: &rusqlite::Row<'_>) -> rusqlite::Result<Utilisateur> {
    Ok(Utilisateur {
        id: r.get(0)?,
        email: r.get(1)?,
        password_hash: r.get(2)?,
        username: r.get(3)?,
        ade_url: r.get(4)?,
        gemini_key: r.get(5)?,
        gemini_model: r.get(6)?,
        preferences: r.get(7)?,
        created_at: r.get(8)?,
    })
}

fn devoir(r: &rusqlite::Row<'_>) -> rusqlite::Result<Devoir> {
    Ok(Devoir {
        id: r.get(0)?,
        titre: r.get(1)?,
        matiere: r.get(2)?,
        echeance: r.get(3)?,
        type_: r.get(4)?,
        statut: r.get(5)?,
    })
}

const COLONNES_DEVOIR: &str = "id, titre, COALESCE(matiere,''), COALESCE(echeance,''), \
     COALESCE(type,'devoir'), COALESCE(statut,'a_faire')";

fn io_err(e: std::io::Error) -> Erreur {
    Erreur::Interne(format!("Erreur de fichier : {e}"))
}

impl Store {
    pub fn ouvrir(chemin: &Path) -> Resultat<Store> {
        if let Some(parent) = chemin.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(io_err)?;
        }
        let conn = Connection::open(chemin)?;
        let store = Store {
            conn: Mutex::new(conn),
            chemin: Some(chemin.to_path_buf()),
        };
        store.initialiser()?;
        Ok(store)
    }

    pub fn en_memoire() -> Resultat<Store> {
        let store = Store {
            conn: Mutex::new(Connection::open_in_memory()?),
            chemin: None,
        };
        store.initialiser()?;
        Ok(store)
    }

    fn verrou(&self) -> Resultat<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| Erreur::Interne("Verrou de base de données indisponible.".into()))
    }

    fn initialiser(&self) -> Resultat<()> {
        {
            let conn = self.verrou()?;
            conn.pragma_update(None, "foreign_keys", "ON")?;
            conn.pragma_update(None, "busy_timeout", 5000)?;
            // Le mode WAL est sans effet (et refusé) sur une base en mémoire : on ignore l'échec.
            let _ = conn.pragma_update(None, "journal_mode", "WAL");
        }
        self.migrer()
    }

    pub fn version(&self) -> Resultat<i64> {
        Ok(self
            .verrou()?
            .query_row("PRAGMA user_version", [], |r| r.get(0))?)
    }

    fn colonnes(conn: &Connection, table: &str) -> Resultat<HashSet<String>> {
        // `table` provient uniquement de constantes internes (jamais d'une entrée utilisateur).
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let noms = stmt
            .query_map([], |r| r.get::<_, String>(1))?
            .collect::<Result<HashSet<_>, _>>()?;
        Ok(noms)
    }

    fn migrer(&self) -> Resultat<()> {
        let version: i64 = self
            .verrou()?
            .query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version >= 2 {
            return Ok(());
        }
        let existante: bool = self.verrou()?.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name IN ('users','devoirs'))",
            [],
            |r| r.get(0),
        )?;
        if existante {
            self.sauvegarder("avant-migration-rust")?;
        }
        let mut conn = self.verrou()?;
        let tx = conn.transaction()?;
        if version < 1 {
            // v1 : schéma de l'ancienne version Python (créé uniquement s'il n'existe pas).
            tx.execute_batch(
                "CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY,
                    email VARCHAR NOT NULL,
                    password_hash VARCHAR NOT NULL,
                    username VARCHAR DEFAULT '',
                    ade_ics_url VARCHAR DEFAULT '',
                    gemini_api_key VARCHAR DEFAULT '',
                    created_at DATETIME
                );
                CREATE TABLE IF NOT EXISTS devoirs (
                    id INTEGER PRIMARY KEY,
                    user_id INTEGER REFERENCES users (id),
                    titre VARCHAR NOT NULL,
                    matiere VARCHAR DEFAULT '',
                    echeance VARCHAR DEFAULT '',
                    type VARCHAR DEFAULT 'devoir',
                    statut VARCHAR DEFAULT 'a_faire'
                );",
            )?;
            // Très anciennes bases : colonnes ajoutées au fil des versions Python.
            let cols = Self::colonnes(&tx, "devoirs")?;
            for (nom, ddl) in [
                ("matiere", "VARCHAR DEFAULT ''"),
                ("echeance", "VARCHAR DEFAULT ''"),
                ("type", "VARCHAR DEFAULT 'devoir'"),
                ("user_id", "INTEGER REFERENCES users (id)"),
            ] {
                if !cols.contains(nom) {
                    tx.execute_batch(&format!("ALTER TABLE devoirs ADD COLUMN {nom} {ddl}"))?;
                }
            }
        }
        // v2 : modèle Gemini et préférences d'affichage par utilisateur, index et unicité de l'e-mail.
        let cols = Self::colonnes(&tx, "users")?;
        if !cols.contains("gemini_model") {
            tx.execute_batch("ALTER TABLE users ADD COLUMN gemini_model VARCHAR DEFAULT ''")?;
        }
        if !cols.contains("preferences") {
            tx.execute_batch("ALTER TABLE users ADD COLUMN preferences VARCHAR DEFAULT '{}'")?;
        }
        tx.execute_batch(
            "CREATE UNIQUE INDEX IF NOT EXISTS ix_users_email ON users (email);
             CREATE INDEX IF NOT EXISTS ix_devoirs_user_id ON devoirs (user_id);",
        )?;
        tx.pragma_update(None, "user_version", 2)?;
        tx.commit()?;
        Ok(())
    }

    /// Copie cohérente de la base (`VACUUM INTO`) à côté du fichier d'origine.
    pub fn sauvegarder(&self, etiquette: &str) -> Resultat<Option<PathBuf>> {
        let Some(chemin) = &self.chemin else {
            return Ok(None);
        };
        let dossier = chemin
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
            .join("sauvegardes");
        fs::create_dir_all(&dossier).map_err(io_err)?;
        let horodatage = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let cible = dossier.join(format!("{etiquette}-{horodatage}.db"));
        if cible.exists() {
            return Ok(Some(cible));
        }
        self.verrou()?
            .execute("VACUUM INTO ?1", [cible.to_string_lossy().as_ref()])?;
        Ok(Some(cible))
    }

    // ------------------------------------------------------------ Utilisateurs

    /// Crée l'utilisateur ; `None` si l'e-mail existe déjà.
    pub fn creer_utilisateur(
        &self,
        email: &str,
        password_hash: &str,
        username: &str,
    ) -> Resultat<Option<Utilisateur>> {
        let conn = self.verrou()?;
        let existe: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = ?1)",
            [email],
            |r| r.get(0),
        )?;
        if existe {
            return Ok(None);
        }
        let maintenant = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.6f")
            .to_string();
        conn.execute(
            "INSERT INTO users (email, password_hash, username, created_at) VALUES (?1,?2,?3,?4)",
            params![email, password_hash, username, maintenant],
        )?;
        let id = conn.last_insert_rowid();
        Ok(conn
            .query_row(
                &format!("SELECT {COLONNES_UTILISATEUR} FROM users WHERE id = ?1"),
                [id],
                utilisateur,
            )
            .optional()?)
    }

    pub fn utilisateur_par_email(&self, email: &str) -> Resultat<Option<Utilisateur>> {
        Ok(self
            .verrou()?
            .query_row(
                &format!("SELECT {COLONNES_UTILISATEUR} FROM users WHERE email = ?1"),
                [email],
                utilisateur,
            )
            .optional()?)
    }

    pub fn utilisateur_par_id(&self, id: i64) -> Resultat<Option<Utilisateur>> {
        Ok(self
            .verrou()?
            .query_row(
                &format!("SELECT {COLONNES_UTILISATEUR} FROM users WHERE id = ?1"),
                [id],
                utilisateur,
            )
            .optional()?)
    }

    pub fn enregistrer_config(&self, id: i64, maj: &MajConfig<'_>) -> Resultat<()> {
        let conn = self.verrou()?;
        conn.execute(
            "UPDATE users SET username = ?2, ade_ics_url = ?3 WHERE id = ?1",
            params![id, maj.username, maj.ade_url],
        )?;
        if let Some(modele) = maj.gemini_model {
            conn.execute(
                "UPDATE users SET gemini_model = ?2 WHERE id = ?1",
                params![id, modele],
            )?;
        }
        if let Some(cle) = maj.gemini_key {
            conn.execute(
                "UPDATE users SET gemini_api_key = ?2 WHERE id = ?1",
                params![id, cle],
            )?;
        }
        Ok(())
    }

    pub fn enregistrer_preferences(&self, id: i64, preferences: &str) -> Resultat<()> {
        self.verrou()?.execute(
            "UPDATE users SET preferences = ?2 WHERE id = ?1",
            params![id, preferences],
        )?;
        Ok(())
    }

    // ----------------------------------------------------------------- Devoirs

    pub fn lister_devoirs(&self, user_id: i64) -> Resultat<Vec<Devoir>> {
        let conn = self.verrou()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {COLONNES_DEVOIR} FROM devoirs WHERE user_id = ?1 ORDER BY id"
        ))?;
        let lignes = stmt
            .query_map([user_id], devoir)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(lignes)
    }

    pub fn creer_devoir(&self, user_id: i64, entree: DevoirEntree) -> Resultat<Devoir> {
        let e = entree.valider()?;
        let conn = self.verrou()?;
        conn.execute(
            "INSERT INTO devoirs (user_id, titre, matiere, echeance, type, statut) \
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![user_id, e.titre, e.matiere, e.echeance, e.type_, e.statut],
        )?;
        Ok(Devoir {
            id: conn.last_insert_rowid(),
            titre: e.titre,
            matiere: e.matiere,
            echeance: e.echeance,
            type_: e.type_,
            statut: e.statut,
        })
    }

    pub fn modifier_devoir(
        &self,
        user_id: i64,
        id: i64,
        m: DevoirModification,
    ) -> Resultat<Devoir> {
        let conn = self.verrou()?;
        let actuel = conn
            .query_row(
                &format!("SELECT {COLONNES_DEVOIR} FROM devoirs WHERE id = ?1 AND user_id = ?2"),
                params![id, user_id],
                devoir,
            )
            .optional()?
            .ok_or_else(|| Erreur::Introuvable("Devoir introuvable".into()))?;
        let d = m.appliquer(actuel)?;
        conn.execute(
            "UPDATE devoirs SET titre=?3, matiere=?4, echeance=?5, type=?6, statut=?7 \
             WHERE id=?1 AND user_id=?2",
            params![d.id, user_id, d.titre, d.matiere, d.echeance, d.type_, d.statut],
        )?;
        Ok(d)
    }

    pub fn supprimer_devoir(&self, user_id: i64, id: i64) -> Resultat<()> {
        let n = self.verrou()?.execute(
            "DELETE FROM devoirs WHERE id = ?1 AND user_id = ?2",
            params![id, user_id],
        )?;
        if n == 0 {
            return Err(Erreur::Introuvable("Devoir introuvable".into()));
        }
        Ok(())
    }

    /// Importe un lot validé : tout ou rien (transaction).
    pub fn importer_devoirs(&self, user_id: i64, entrees: Vec<DevoirEntree>) -> Resultat<usize> {
        let valides = entrees
            .into_iter()
            .enumerate()
            .map(|(i, e)| {
                e.valider()
                    .map_err(|err| Erreur::Validation(format!("Devoir n°{} : {err}", i + 1)))
            })
            .collect::<Resultat<Vec<_>>>()?;
        let mut conn = self.verrou()?;
        let tx = conn.transaction()?;
        for e in &valides {
            tx.execute(
                "INSERT INTO devoirs (user_id, titre, matiere, echeance, type, statut) \
                 VALUES (?1,?2,?3,?4,?5,?6)",
                params![user_id, e.titre, e.matiere, e.echeance, e.type_, e.statut],
            )?;
        }
        tx.commit()?;
        Ok(valides.len())
    }
}
