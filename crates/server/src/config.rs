//! Configuration du serveur, lue dans les variables d'environnement.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Settings {
    pub bind: SocketAddr,
    pub db_path: PathBuf,
    /// Secret de signature des jetons et de chiffrement des clés Gemini (jamais journalisé).
    pub secret: Vec<u8>,
    pub jwt_minutes: i64,
    pub bcrypt_cost: u32,
    /// Origines CORS autorisées ; vide = même origine uniquement.
    pub allowed_origins: Vec<String>,
    pub ade_allowed_hosts: Vec<String>,
    pub gemini_base: String,
    /// Clé Gemini globale facultative (repli quand l'utilisateur n'a pas la sienne).
    pub gemini_api_key: Option<String>,
    pub ics_ttl: Duration,
    pub ics_max_entries: usize,
    pub frontend_dir: Option<PathBuf>,
}

fn env(nom: &str) -> Option<String> {
    std::env::var(nom)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn liste(nom: &str) -> Vec<String> {
    env(nom)
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Accepte un chemin simple ou l'ancienne URL SQLAlchemy (`sqlite:////data/app.db`).
fn chemin_base() -> PathBuf {
    let brut = env("DATABASE_PATH")
        .or_else(|| env("DATABASE_URL"))
        .unwrap_or_else(|| "data/tableau-de-bord.db".to_string());
    PathBuf::from(brut.strip_prefix("sqlite:///").unwrap_or(&brut))
}

/// `SECRET_KEY` si fournie ; sinon un secret aléatoire créé une fois et conservé à côté de la base,
/// afin que les sessions et les clés chiffrées survivent aux redémarrages.
fn charger_secret(db_path: &Path) -> anyhow::Result<Vec<u8>> {
    if let Some(s) = env("SECRET_KEY") {
        if s.len() < 16 {
            anyhow::bail!("SECRET_KEY doit contenir au moins 16 caractères.");
        }
        return Ok(s.into_bytes());
    }
    let dossier = db_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let fichier = dossier.join("secret.key");
    if let Ok(contenu) = std::fs::read_to_string(&fichier) {
        let contenu = contenu.trim();
        if contenu.len() >= 16 {
            return Ok(contenu.as_bytes().to_vec());
        }
    }
    let secret = crate::crypto::secret_aleatoire();
    std::fs::create_dir_all(&dossier)?;
    std::fs::write(&fichier, &secret)?;
    tracing::warn!(
        "SECRET_KEY n'est pas définie : un secret aléatoire a été créé dans {}.",
        fichier.display()
    );
    Ok(secret.into_bytes())
}

impl Settings {
    pub fn depuis_env() -> anyhow::Result<Settings> {
        let port: u16 = env("PORT").and_then(|p| p.parse().ok()).unwrap_or(8000);
        let bind = match env("BIND_ADDR") {
            Some(a) => a.parse()?,
            None => SocketAddr::from(([0, 0, 0, 0], port)),
        };
        let db_path = chemin_base();
        let secret = charger_secret(&db_path)?;
        let frontend_dir = env("FRONTEND_DIR").map(PathBuf::from).or_else(|| {
            ["frontend-dist", "crates/frontend/dist"]
                .iter()
                .map(PathBuf::from)
                .find(|p| p.join("index.html").is_file())
        });
        Ok(Settings {
            bind,
            db_path,
            secret,
            jwt_minutes: env("JWT_EXPIRATION_MINUTES")
                .and_then(|v| v.parse().ok())
                .unwrap_or(60 * 24),
            bcrypt_cost: env("BCRYPT_COST")
                .and_then(|v| v.parse().ok())
                .map_or(12, |c: u32| c.clamp(4, 15)),
            allowed_origins: liste("ALLOWED_ORIGINS"),
            ade_allowed_hosts: liste("ADE_ALLOWED_HOSTS")
                .into_iter()
                .map(|h| h.to_lowercase().trim_start_matches('.').to_string())
                .collect(),
            gemini_base: env("GEMINI_BASE_URL")
                .unwrap_or_else(|| tdb_core::planner::URL_GEMINI.to_string()),
            gemini_api_key: env("GEMINI_API_KEY"),
            ics_ttl: Duration::from_secs(
                env("ICS_CACHE_TTL_SECONDS")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(300),
            ),
            ics_max_entries: env("ICS_CACHE_MAX_ENTRIES")
                .and_then(|v| v.parse().ok())
                .unwrap_or(64),
            frontend_dir,
        })
    }

    /// Réglages minimaux et rapides pour les tests (mot de passe peu coûteux, secret fixe).
    pub fn pour_tests() -> Settings {
        Settings {
            bind: SocketAddr::from(([127, 0, 0, 1], 0)),
            db_path: PathBuf::new(),
            secret: b"secret-de-test-uniquement-0123456789".to_vec(),
            jwt_minutes: 60,
            bcrypt_cost: 4,
            allowed_origins: vec![],
            ade_allowed_hosts: vec![],
            gemini_base: tdb_core::planner::URL_GEMINI.to_string(),
            gemini_api_key: None,
            ics_ttl: Duration::from_secs(300),
            ics_max_entries: 8,
            frontend_dir: None,
        }
    }
}
