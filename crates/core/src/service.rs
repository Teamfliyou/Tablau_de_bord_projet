//! Orchestration : configuration, flux ADE avec cache (mémoire + SQLite), plan de révision.

use crate::ade::{self, Categories, Cours, Filtres};
use crate::db::{Db, RapportMigration};
use crate::error::{validation, Erreur, Resultat};
use crate::planner::{self, Plan};
use crate::secrets::{StockageSecrets, SECRET_GEMINI};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const CACHE_TTL: Duration = Duration::from_secs(300);
const PREFERENCES_MAX_OCTETS: usize = 16 * 1024;

pub use tdb_shared::{Config, ConfigUpdate, CoursDuJourEtat, CoursSemaineEtat, EtatFlux};

struct EntreeCache {
    url: String,
    contenu: String,
    instant: Instant,
    maj: String,
}

pub struct Service {
    pub db: Db,
    secrets: Arc<dyn StockageSecrets>,
    cache: Mutex<Option<EntreeCache>>,
    gemini_base: String,
}

impl Service {
    pub fn nouveau(db: Db, secrets: Arc<dyn StockageSecrets>) -> Service {
        Self::avec_base_gemini(db, secrets, planner::URL_GEMINI)
    }

    pub fn avec_base_gemini(db: Db, secrets: Arc<dyn StockageSecrets>, base: &str) -> Service {
        Service {
            db,
            secrets,
            cache: Mutex::new(None),
            gemini_base: base.to_string(),
        }
    }

    // ------------------------------------------------------------ Configuration

    pub fn obtenir_config(&self) -> Resultat<Config> {
        Ok(Config {
            username: self.db.lire_parametre("username")?.unwrap_or_default(),
            ade_url: self.db.lire_parametre("ade_url")?.unwrap_or_default(),
            gemini_model: self
                .db
                .lire_parametre("gemini_model")?
                .unwrap_or_else(|| planner::MODELE_PAR_DEFAUT.to_string()),
            gemini_configured: self.cle_gemini()?.is_some(),
        })
    }

    fn cle_gemini(&self) -> Resultat<Option<String>> {
        Ok(self
            .secrets
            .lire(SECRET_GEMINI)?
            .filter(|c| !c.trim().is_empty()))
    }

    pub fn enregistrer_config(&self, maj: ConfigUpdate) -> Resultat<Config> {
        let username = maj.username.trim();
        if username.chars().count() > 80 {
            return validation("Le pseudo est limité à 80 caractères.");
        }
        let ade_url = maj.ade_url.trim();
        if ade_url.len() > 2048 {
            return validation("Le lien ADE est trop long.");
        }
        if !ade_url.is_empty() {
            ade::valider_syntaxe_url(ade_url)?;
        }
        let modele = match maj
            .gemini_model
            .as_deref()
            .map(str::trim)
            .filter(|m| !m.is_empty())
        {
            Some(m) => Some(planner::valider_modele(m)?.to_string()),
            None => None,
        };
        let nouvelle_cle = maj
            .gemini_key
            .as_deref()
            .map(str::trim)
            .filter(|k| !k.is_empty());
        if nouvelle_cle.is_some_and(|k| k.len() > 512) {
            return validation("La clé API est trop longue.");
        }

        self.db.ecrire_parametre("username", username)?;
        self.db.ecrire_parametre("ade_url", ade_url)?;
        if let Some(m) = modele {
            self.db.ecrire_parametre("gemini_model", &m)?;
        }
        if maj.remove_gemini_key {
            self.secrets.supprimer(SECRET_GEMINI)?;
        } else if let Some(k) = nouvelle_cle {
            self.secrets.ecrire(SECRET_GEMINI, k)?;
        }
        self.obtenir_config()
    }

    /// Préférences d'affichage (style de navigation, thème, filtres enregistrés) : objet JSON opaque.
    pub fn obtenir_preferences(&self) -> Resultat<serde_json::Value> {
        Ok(self
            .db
            .lire_parametre("preferences")?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({})))
    }

    pub fn enregistrer_preferences(&self, prefs: serde_json::Value) -> Resultat<()> {
        if !prefs.is_object() {
            return validation("Préférences invalides.");
        }
        let texte = prefs.to_string();
        if texte.len() > PREFERENCES_MAX_OCTETS {
            return validation("Préférences trop volumineuses.");
        }
        self.db.ecrire_parametre("preferences", &texte)
    }

    // ---------------------------------------------------------------- Flux ADE

    fn url_ade(&self) -> Resultat<Option<String>> {
        Ok(self
            .db
            .lire_parametre("ade_url")?
            .filter(|u| !u.trim().is_empty()))
    }

    /// Contenu ICS : cache mémoire (5 min) → réseau → dernier cache SQLite en cas de panne réseau.
    async fn contenu_ics(&self, url: &str, forcer: bool) -> Resultat<(String, EtatFlux)> {
        if !forcer {
            if let Some(e) = self
                .cache
                .lock()
                .map_err(|_| Erreur::Interne("verrou".into()))?
                .as_ref()
            {
                if e.url == url && e.instant.elapsed() < CACHE_TTL {
                    return Ok((
                        e.contenu.clone(),
                        EtatFlux {
                            hors_ligne: false,
                            maj: Some(e.maj.clone()),
                            avertissement: None,
                        },
                    ));
                }
            }
        }
        match ade::telecharger_ics(url)
            .await
            .and_then(|c| ade::parser_cours(&c).map(|_| c))
        {
            Ok(contenu) => {
                let maj = chrono::Utc::now().to_rfc3339();
                self.db.enregistrer_cache_ics(url, &contenu, &maj)?;
                *self
                    .cache
                    .lock()
                    .map_err(|_| Erreur::Interne("verrou".into()))? = Some(EntreeCache {
                    url: url.into(),
                    contenu: contenu.clone(),
                    instant: Instant::now(),
                    maj: maj.clone(),
                });
                Ok((
                    contenu,
                    EtatFlux {
                        hors_ligne: false,
                        maj: Some(maj),
                        avertissement: None,
                    },
                ))
            }
            Err(Erreur::Reseau(msg)) => {
                // Panne réseau / flux illisible : on affiche le dernier emploi du temps connu.
                if let Some((contenu, maj)) = self.db.lire_cache_ics(url)? {
                    Ok((
                        contenu,
                        EtatFlux {
                            hors_ligne: true,
                            maj: Some(maj),
                            avertissement: Some(msg),
                        },
                    ))
                } else {
                    Err(Erreur::Reseau(format!(
                        "{msg} Aucun emploi du temps en cache n'est disponible."
                    )))
                }
            }
            Err(autre) => Err(autre),
        }
    }

    pub async fn charger_cours(&self, forcer: bool) -> Resultat<(Vec<Cours>, EtatFlux)> {
        let Some(url) = self.url_ade()? else {
            return Ok((vec![], EtatFlux::default()));
        };
        let (contenu, etat) = self.contenu_ics(&url, forcer).await?;
        Ok((ade::parser_cours(&contenu)?, etat))
    }

    pub async fn cours_du_jour(
        &self,
        date: Option<&str>,
        f: &Filtres,
    ) -> Resultat<CoursDuJourEtat> {
        let jour = ade::parser_jour(date)?;
        let (tous, etat) = self.charger_cours(false).await?;
        Ok(CoursDuJourEtat {
            jour: ade::cours_du_jour(tous, jour, f),
            etat,
        })
    }

    pub async fn cours_semaine(
        &self,
        date: Option<&str>,
        f: &Filtres,
    ) -> Resultat<CoursSemaineEtat> {
        let jour = ade::parser_jour(date)?;
        let (tous, etat) = self.charger_cours(false).await?;
        Ok(CoursSemaineEtat {
            semaine: ade::cours_semaine(tous, jour, f),
            etat,
        })
    }

    pub async fn analyser_categories(&self) -> Resultat<Categories> {
        let (tous, _) = self.charger_cours(true).await?;
        Ok(ade::analyser_categories(&tous))
    }

    // ------------------------------------------------------------------- Plan IA

    pub async fn generer_plan(&self, f: &Filtres) -> Resultat<Plan> {
        self.generer_plan_options(f, &planner::OptionsPlan::default()).await
    }

    pub async fn generer_plan_options(&self, f: &Filtres, options: &planner::OptionsPlan) -> Resultat<Plan> {
        // Sans emploi du temps disponible, le plan reste utile à partir des devoirs seuls.
        let cours = match self.charger_cours(false).await {
            Ok((tous, _)) => ade::cours_du_jour(tous, None, f).cours,
            Err(e) => {
                log::warn!("Plan généré sans emploi du temps : {e}");
                vec![]
            }
        };
        let devoirs = self.db.lister_devoirs()?;
        let cle = self.cle_gemini()?;
        let modele = self
            .db
            .lire_parametre("gemini_model")?
            .unwrap_or_else(|| planner::MODELE_PAR_DEFAUT.to_string());
        Ok(
            planner::generer_plan_options(&self.gemini_base, cle.as_deref(), &modele, &cours, &devoirs, options)
                .await,
        )
    }

    // ----------------------------------------------------------------- Migration

    pub fn migrer_ancienne_base(&self, chemin: &Path) -> Resultat<RapportMigration> {
        self.db.sauvegarder("avant-import-ancienne-base")?;
        let (rapport, ancienne) = self.db.migrer_ancienne_base(chemin)?;
        if !ancienne.gemini_key.trim().is_empty() && self.cle_gemini()?.is_none() {
            self.secrets
                .ecrire(SECRET_GEMINI, ancienne.gemini_key.trim())?;
        }
        Ok(rapport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::SecretsMemoire;

    fn service() -> Service {
        Service::nouveau(
            Db::en_memoire().unwrap(),
            Arc::new(SecretsMemoire::default()),
        )
    }

    fn maj(url: &str) -> ConfigUpdate {
        ConfigUpdate {
            username: " Léa ".into(),
            ade_url: url.into(),
            gemini_model: None,
            gemini_key: None,
            remove_gemini_key: false,
        }
    }

    #[test]
    fn configuration_et_secrets_separes() {
        let s = service();
        assert!(!s.obtenir_config().unwrap().gemini_configured);
        let mut m = maj("https://ade.example.com/x.ics");
        m.gemini_key = Some("  AIzaSECRET  ".into());
        m.gemini_model = Some("gemini-2.5-pro".into());
        let c = s.enregistrer_config(m).unwrap();
        assert_eq!(
            (
                c.username.as_str(),
                c.gemini_model.as_str(),
                c.gemini_configured
            ),
            ("Léa", "gemini-2.5-pro", true)
        );
        // La clé est dans le coffre, jamais dans SQLite ni dans la config renvoyée.
        assert!(!serde_json::to_string(&c).unwrap().contains("AIzaSECRET"));
        assert!(s.db.lire_parametre("gemini_key").unwrap().is_none());
        assert_eq!(s.cle_gemini().unwrap().as_deref(), Some("AIzaSECRET"));
        // Une sauvegarde sans clé la conserve ; la suppression explicite l'efface.
        s.enregistrer_config(maj("")).unwrap();
        assert!(s.obtenir_config().unwrap().gemini_configured);
        let mut m = maj("");
        m.remove_gemini_key = true;
        assert!(!s.enregistrer_config(m).unwrap().gemini_configured);
    }

    #[test]
    fn configuration_refuse_url_dangereuse_et_modele_invalide() {
        let s = service();
        assert!(s
            .enregistrer_config(maj("http://ade.example.com/x.ics"))
            .is_err());
        assert!(s
            .enregistrer_config(maj("https://127.0.0.1/x.ics"))
            .is_err());
        let mut m = maj("");
        m.gemini_model = Some("../evil".into());
        assert!(s.enregistrer_config(m).is_err());
    }

    #[test]
    fn preferences_aller_retour() {
        let s = service();
        assert_eq!(s.obtenir_preferences().unwrap(), serde_json::json!({}));
        s.enregistrer_preferences(serde_json::json!({"theme":"dark","interface":"dock"}))
            .unwrap();
        assert_eq!(s.obtenir_preferences().unwrap()["theme"], "dark");
        assert!(s
            .enregistrer_preferences(serde_json::json!([1, 2]))
            .is_err());
    }

    #[tokio::test]
    async fn sans_url_aucun_cours() {
        let s = service();
        let r = s.cours_du_jour(None, &Filtres::default()).await.unwrap();
        assert_eq!(r.jour.nb_cours, 0);
        assert!(s
            .cours_du_jour(Some("n'importe quoi"), &Filtres::default())
            .await
            .is_err());
    }

    #[tokio::test]
    async fn hors_ligne_utilise_le_dernier_cache() {
        let s = service();
        let url = "https://127.0.0.1/ics"; // refusée avant tout accès réseau → erreur de validation
        s.db.ecrire_parametre("ade_url", url).unwrap();
        assert!(matches!(
            s.charger_cours(false).await,
            Err(Erreur::Validation(_))
        ));
        // Simule une panne réseau : le cache SQLite d'une URL publique est utilisé.
        let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nDTSTART:20261005T080000Z\r\nDTEND:20261005T090000Z\r\nSUMMARY:ALGEBRE\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let url = "https://ade.invalid/flux.ics"; // domaine inexistant : la résolution échoue
        s.db.ecrire_parametre("ade_url", url).unwrap();
        assert!(
            s.charger_cours(false).await.is_err(),
            "pas de cache : erreur claire"
        );
        s.db.enregistrer_cache_ics(url, ics, "2026-10-01T08:00:00+00:00")
            .unwrap();
        let (cours, etat) = s.charger_cours(false).await.unwrap();
        assert_eq!(cours.len(), 1);
        assert!(
            etat.hors_ligne,
            "le flux injoignable doit être signalé comme hors ligne"
        );
        assert_eq!(etat.maj.as_deref(), Some("2026-10-01T08:00:00+00:00"));
    }
}
