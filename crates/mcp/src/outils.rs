//! Outils MCP : consultation des devoirs (« tâches ») regroupés par matière (« projets »),
//! et modifications contrôlées (autorisation dans l'application + confirmation à usage unique).

use std::collections::hash_map::RandomState;
use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hasher};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chrono::NaiveDate;
use serde_json::{json, Map, Value};
use tdb_core::db::Db;
use tdb_shared::{
    Devoir, DevoirEntree, DevoirModification, TodoEntree, OUTILS_MCP, PARAM_MCP_ACTIF,
    PARAM_MCP_ECRITURE, STATUTS, TYPES,
};

type R<T> = Result<T, String>;

const SANS_MATIERE: &str = "Sans matière";
const DUREE_CONFIRMATION: Duration = Duration::from_secs(300);
const CONFIRMATIONS_MAX: usize = 32;
const LIMITE_DEFAUT: i64 = 50;
const LIMITE_MAX: i64 = 200;

const DESACTIVEE: &str = "L'intégration Hermes Agent est désactivée : active-la dans l'application (Configuration → Intégrations IA → Hermes Agent / MCP).";
const LECTURE_SEULE: &str = "Les modifications par Hermes Agent ne sont pas autorisées : active « Autoriser les modifications » dans l'application (Configuration → Intégrations IA). L'intégration reste en lecture seule.";

/// Modification validée, prête à être exécutée après confirmation.
enum Action {
    Creer(DevoirEntree),
    Renommer {
        ancienne: String,
        nouvelle: String,
    },
    Modifier {
        id: i64,
        modification: DevoirModification,
    },
}

struct Attente {
    outil: String,
    arguments: Value,
    expire: Instant,
}

pub struct Outils {
    base: PathBuf,
    attentes: HashMap<String, Attente>,
    alea: RandomState,
    compteur: u64,
}

impl Outils {
    pub fn nouveau(base: PathBuf) -> Self {
        Outils {
            base,
            attentes: HashMap::new(),
            alea: RandomState::new(),
            compteur: 0,
        }
    }

    pub fn existe(nom: &str) -> bool {
        OUTILS_MCP.iter().any(|o| o.nom == nom)
    }

    /// Réponse de `tools/list` : description, schéma d'entrée et annotations de chaque outil.
    pub fn catalogue() -> Vec<Value> {
        OUTILS_MCP
            .iter()
            .map(|o| {
                json!({
                    "name": o.nom,
                    "description": o.description,
                    "inputSchema": schema(o.nom),
                    "annotations": {
                        "readOnlyHint": !o.ecriture,
                        "destructiveHint": false,
                        "idempotentHint": !o.ecriture || o.nom == "complete_task",
                        "openWorldHint": false
                    }
                })
            })
            .collect()
    }

    pub fn executer(&mut self, nom: &str, arguments: &Value) -> R<Value> {
        let outil = OUTILS_MCP
            .iter()
            .find(|o| o.nom == nom)
            .ok_or_else(|| format!("Outil inconnu : {nom}"))?;
        let args = arguments
            .as_object()
            .ok_or("Les arguments doivent former un objet.")?;
        verifier_arguments(nom, args)?;

        let db = self.ouvrir(false)?;
        if !parametre_actif(&db, PARAM_MCP_ACTIF) {
            return Err(DESACTIVEE.into());
        }
        let devoirs = db.lister_devoirs().map_err(|e| e.to_string())?;
        let jour = tdb_core::ade::aujourdhui();
        if !outil.ecriture {
            return consulter(nom, args, &devoirs, jour);
        }
        if !parametre_actif(&db, PARAM_MCP_ECRITURE) {
            return Err(LECTURE_SEULE.into());
        }
        // Validation complète (et aperçu) avant d'émettre ou d'accepter un jeton.
        let (action, resume) = preparer(nom, args, &devoirs, jour)?;
        drop(db);

        let mut sans_jeton = args.clone();
        let jeton = sans_jeton.remove("confirmation");
        let sans_jeton = Value::Object(sans_jeton);
        match jeton {
            None | Some(Value::Null) => {
                let jeton = self.emettre(nom, sans_jeton);
                return Ok(json!({
                    "confirmation_requise": true,
                    "action": resume,
                    "confirmation": jeton,
                    "expire_dans_secondes": DUREE_CONFIRMATION.as_secs(),
                    "consigne": format!("Rien n'a encore été modifié. Présente cette action à l'utilisateur et, s'il l'accepte, rappelle « {nom} » avec exactement les mêmes arguments et \"confirmation\": \"{jeton}\".")
                }));
            }
            Some(Value::String(j)) => self.consommer(j.trim(), nom, &sans_jeton)?,
            Some(_) => return Err("« confirmation » doit être une chaîne.".into()),
        }

        let db = self.ouvrir(true)?;
        if !parametre_actif(&db, PARAM_MCP_ECRITURE) {
            return Err(LECTURE_SEULE.into());
        }
        appliquer(&db, action, jour)
    }

    fn ouvrir(&self, ecriture: bool) -> R<Db> {
        Db::ouvrir_existante(&self.base, ecriture).map_err(|e| e.to_string())
    }

    fn emettre(&mut self, outil: &str, arguments: Value) -> String {
        let maintenant = Instant::now();
        self.attentes.retain(|_, a| a.expire > maintenant);
        while self.attentes.len() >= CONFIRMATIONS_MAX {
            let plus_ancien = self
                .attentes
                .iter()
                .min_by_key(|(_, a)| a.expire)
                .map(|(j, _)| j.clone());
            match plus_ancien {
                Some(j) => self.attentes.remove(&j),
                None => break,
            };
        }
        self.compteur += 1;
        let mut h = self.alea.build_hasher();
        h.write_u64(self.compteur);
        h.write_u128(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default(),
        );
        let jeton = format!("{:016x}", h.finish());
        self.attentes.insert(
            jeton.clone(),
            Attente {
                outil: outil.to_string(),
                arguments,
                expire: maintenant + DUREE_CONFIRMATION,
            },
        );
        jeton
    }

    fn consommer(&mut self, jeton: &str, outil: &str, arguments: &Value) -> R<()> {
        let valide = self
            .attentes
            .get(jeton)
            .is_some_and(|a| a.expire > Instant::now());
        if !valide {
            self.attentes.remove(jeton);
            return Err("Jeton de confirmation invalide ou expiré : rappelle l'outil sans « confirmation » pour obtenir un nouvel aperçu.".into());
        }
        let attente = &self.attentes[jeton];
        if attente.outil != outil || &attente.arguments != arguments {
            return Err("Ce jeton de confirmation correspond à une autre action : rappelle l'outil sans « confirmation » pour obtenir un nouvel aperçu.".into());
        }
        self.attentes.remove(jeton);
        Ok(())
    }
}

fn parametre_actif(db: &Db, cle: &str) -> bool {
    matches!(
        db.lire_parametre(cle).ok().flatten().as_deref(),
        Some("1") | Some("true")
    )
}

// ------------------------------------------------------------------ Schémas

fn chaine(description: &str) -> Value {
    json!({ "type": "string", "description": description })
}

fn entier(description: &str, min: i64, max: i64) -> Value {
    json!({ "type": "integer", "description": description, "minimum": min, "maximum": max })
}

fn objet(proprietes: Value, requis: &[&str]) -> Value {
    json!({ "type": "object", "properties": proprietes, "required": requis, "additionalProperties": false })
}

fn schema(nom: &str) -> Value {
    let projet = chaine("Nom du projet (matière), tel que renvoyé par list_projects. « Sans matière » désigne les tâches sans matière.");
    let id = entier("Identifiant de la tâche", 1, i64::MAX);
    let echeance = chaine("Échéance au format AAAA-MM-JJ (vide : aucune)");
    let type_ = json!({ "type": "string", "enum": TYPES, "description": "devoir, ie (interrogation écrite), ds (devoir surveillé) ou exam" });
    let statut =
        json!({ "type": "string", "enum": STATUTS, "description": "a_faire, en_cours ou termine" });
    let priorite = entier("0 = aucune, 1 = basse, 2 = moyenne, 3 = haute", 0, 3);
    let description = chaine("Notes libres (2000 caractères maximum)");
    let confirmation = chaine("Jeton renvoyé par le premier appel, à fournir seulement après l'accord explicite de l'utilisateur");
    let limite = entier(
        "Nombre maximal de tâches renvoyées (50 par défaut)",
        1,
        LIMITE_MAX,
    );
    match nom {
        "get_project_details" => objet(json!({ "projet": projet }), &["projet"]),
        "list_tasks" => objet(
            json!({
                "projet": projet,
                "statut": statut,
                "priorite_min": entier("Priorité minimale (0 à 3)", 0, 3),
                "recherche": chaine("Texte recherché dans le titre, la description et les sous-tâches"),
                "inclure_terminees": { "type": "boolean", "description": "Inclure les tâches terminées (non par défaut)" },
                "limite": limite
            }),
            &[],
        ),
        "get_task_details" | "complete_task" => {
            let mut p = json!({ "id": id });
            if nom == "complete_task" {
                p["confirmation"] = confirmation;
            }
            objet(p, &["id"])
        }
        "get_upcoming_tasks" => objet(
            json!({ "jours": entier("Nombre de jours à partir de demain (7 par défaut)", 1, 90) }),
            &[],
        ),
        "get_completed_tasks" => objet(json!({ "projet": projet, "limite": limite }), &[]),
        "get_project_progress" => objet(json!({ "projet": projet }), &[]),
        "create_project" => objet(
            json!({
                "nom": chaine("Nom du nouveau projet (matière), 120 caractères maximum"),
                "titre": chaine("Titre de la première tâche du projet (un projet existe à travers ses tâches)"),
                "echeance": echeance, "type": type_, "priorite": priorite,
                "description": description, "confirmation": confirmation
            }),
            &["nom", "titre"],
        ),
        "update_project" => objet(
            json!({
                "projet": projet,
                "nouveau_nom": chaine("Nouveau nom du projet (matière)"),
                "confirmation": confirmation
            }),
            &["projet", "nouveau_nom"],
        ),
        "create_task" => objet(
            json!({
                "titre": chaine("Titre de la tâche (200 caractères maximum)"),
                "projet": projet, "echeance": echeance, "type": type_, "statut": statut,
                "priorite": priorite, "description": description,
                "sous_taches": { "type": "array", "items": { "type": "string" }, "description": "Sous-tâches (to-do), 100 maximum" },
                "confirmation": confirmation
            }),
            &["titre", "projet"],
        ),
        "update_task" => objet(
            json!({
                "id": id, "titre": chaine("Nouveau titre"), "projet": projet,
                "echeance": echeance, "type": type_, "statut": statut, "priorite": priorite,
                "description": description, "confirmation": confirmation
            }),
            &["id"],
        ),
        // list_projects, get_today_tasks, get_overdue_tasks : aucun argument.
        _ => objet(json!({}), &[]),
    }
}

/// Refuse les arguments inconnus ou manquants (le schéma publié fait foi).
fn verifier_arguments(nom: &str, args: &Map<String, Value>) -> R<()> {
    let schema = schema(nom);
    let proprietes = schema["properties"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    if let Some(inconnu) = args.keys().find(|k| !proprietes.contains_key(*k)) {
        let attendus: Vec<&str> = proprietes.keys().map(String::as_str).collect();
        return Err(format!(
            "Argument inconnu « {inconnu} » pour {nom}. Arguments acceptés : {}.",
            if attendus.is_empty() {
                "aucun".to_string()
            } else {
                attendus.join(", ")
            }
        ));
    }
    for requis in schema["required"].as_array().into_iter().flatten() {
        let cle = requis.as_str().unwrap_or_default();
        if args.get(cle).is_none_or(Value::is_null) {
            return Err(format!("Argument obligatoire manquant : « {cle} »."));
        }
    }
    Ok(())
}

// ---------------------------------------------------- Lecture des arguments

fn arg_chaine(args: &Map<String, Value>, cle: &str) -> R<Option<String>> {
    match args.get(cle) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.trim().to_string())),
        Some(_) => Err(format!("« {cle} » doit être une chaîne de caractères.")),
    }
}

fn arg_entier(args: &Map<String, Value>, cle: &str, min: i64, max: i64) -> R<Option<i64>> {
    let valeur = match args.get(cle) {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::Number(n)) => n.as_i64(),
        // Tolérance : certains modèles envoient les nombres sous forme de texte.
        Some(Value::String(s)) => s.trim().parse().ok(),
        Some(_) => None,
    };
    match valeur {
        Some(v) if (min..=max).contains(&v) => Ok(Some(v)),
        _ => Err(format!(
            "« {cle} » doit être un entier compris entre {min} et {max}."
        )),
    }
}

fn arg_booleen(args: &Map<String, Value>, cle: &str) -> R<bool> {
    match args.get(cle) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(format!("« {cle} » doit valoir true ou false.")),
    }
}

fn arg_statut(args: &Map<String, Value>) -> R<Option<String>> {
    let s = arg_chaine(args, "statut")?;
    if let Some(v) = &s {
        tdb_shared::valider_statut(v)?;
    }
    Ok(s)
}

fn arg_type(args: &Map<String, Value>) -> R<Option<String>> {
    let t = arg_chaine(args, "type")?;
    if let Some(v) = &t {
        tdb_shared::valider_type(v)?;
    }
    Ok(t)
}

fn arg_id(args: &Map<String, Value>) -> R<i64> {
    arg_entier(args, "id", 1, i64::MAX)?
        .ok_or_else(|| "Argument obligatoire manquant : « id ».".into())
}

fn arg_priorite(args: &Map<String, Value>) -> R<Option<u8>> {
    Ok(arg_entier(args, "priorite", 0, 3)?.map(|p| p as u8))
}

fn arg_limite(args: &Map<String, Value>) -> R<usize> {
    Ok(arg_entier(args, "limite", 1, LIMITE_MAX)?.unwrap_or(LIMITE_DEFAUT) as usize)
}

// ------------------------------------------------------------------ Vues

fn nom_projet(matiere: &str) -> &str {
    if matiere.is_empty() {
        SANS_MATIERE
    } else {
        matiere
    }
}

fn date_de(d: &Devoir) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&d.echeance, "%Y-%m-%d").ok()
}

fn termine(d: &Devoir) -> bool {
    d.statut == "termine"
}

fn en_retard(d: &Devoir, jour: NaiveDate) -> bool {
    !termine(d) && date_de(d).is_some_and(|e| e < jour)
}

fn libelle_priorite(p: u8) -> &'static str {
    match p {
        1 => "basse",
        2 => "moyenne",
        3 => "haute",
        _ => "aucune",
    }
}

fn vue_tache(d: &Devoir, jour: NaiveDate, detail: bool) -> Value {
    let faites = d.todos.iter().filter(|t| t.fait).count();
    let mut v = json!({
        "id": d.id,
        "titre": d.titre,
        "projet": nom_projet(&d.matiere),
        "echeance": (!d.echeance.is_empty()).then_some(&d.echeance),
        "jours_restants": date_de(d).map(|e| (e - jour).num_days()),
        "en_retard": en_retard(d, jour),
        "type": d.type_,
        "statut": d.statut,
        "priorite": d.priorite,
        "priorite_libelle": libelle_priorite(d.priorite),
        "tags": d.tags.iter().map(|t| t.nom.as_str()).collect::<Vec<_>>(),
        "sous_taches": { "faites": faites, "total": d.todos.len() },
    });
    if detail {
        v["description"] = json!(d.description);
        v["sous_taches_liste"] = d
            .todos
            .iter()
            .map(|t| json!({ "texte": t.texte, "fait": t.fait }))
            .collect();
    }
    v
}

/// Ordre d'urgence : non terminées, échéance la plus proche (sans échéance en dernier), priorité haute d'abord.
fn trier_par_urgence(taches: &mut [&Devoir]) {
    taches.sort_by_key(|d| {
        (
            termine(d),
            date_de(d).is_none(),
            date_de(d),
            std::cmp::Reverse(d.priorite),
            d.id,
        )
    });
}

fn vues(mut taches: Vec<&Devoir>, jour: NaiveDate, limite: usize) -> Value {
    trier_par_urgence(&mut taches);
    let total = taches.len();
    json!({
        "nombre": total,
        "tronque": total > limite,
        "taches": taches.iter().take(limite).map(|d| vue_tache(d, jour, false)).collect::<Vec<_>>(),
    })
}

fn par_projet(devoirs: &[Devoir]) -> BTreeMap<&str, Vec<&Devoir>> {
    let mut groupes: BTreeMap<&str, Vec<&Devoir>> = BTreeMap::new();
    for d in devoirs {
        groupes.entry(d.matiere.as_str()).or_default().push(d);
    }
    groupes
}

fn statistiques(nom: &str, taches: &[&Devoir], jour: NaiveDate) -> Value {
    let compte = |s: &str| taches.iter().filter(|d| d.statut == s).count();
    let total = taches.len();
    let terminees = compte("termine");
    let sous_total: usize = taches.iter().map(|d| d.todos.len()).sum();
    let sous_faites: usize = taches
        .iter()
        .map(|d| d.todos.iter().filter(|t| t.fait).count())
        .sum();
    let prochaine = taches
        .iter()
        .filter(|d| !termine(d))
        .filter_map(|d| date_de(d).map(|e| (e, *d)))
        .min_by_key(|(e, d)| (*e, d.id))
        .map(|(_, d)| json!({ "id": d.id, "titre": d.titre, "echeance": d.echeance }));
    json!({
        "projet": nom_projet(nom),
        "taches": total,
        "a_faire": compte("a_faire"),
        "en_cours": compte("en_cours"),
        "terminees": terminees,
        "en_retard": taches.iter().filter(|d| en_retard(d, jour)).count(),
        "progression_pourcent": (terminees * 100 + total / 2).checked_div(total).unwrap_or(0),
        "sous_taches": { "faites": sous_faites, "total": sous_total },
        "prochaine_echeance": prochaine,
    })
}

/// Trouve un projet existant (nom exact, puis sans tenir compte de la casse).
fn resoudre_projet(devoirs: &[Devoir], nom: &str) -> R<String> {
    if let Some(m) = trouver_projet(devoirs, nom) {
        return Ok(m);
    }
    let mut existants: Vec<&str> = devoirs.iter().map(|d| nom_projet(&d.matiere)).collect();
    existants.sort_unstable();
    existants.dedup();
    Err(format!(
        "Projet introuvable : « {} ». Projets existants : {}.",
        nom.trim(),
        if existants.is_empty() {
            "aucun".to_string()
        } else {
            existants.join(", ")
        }
    ))
}

fn trouver_projet(devoirs: &[Devoir], nom: &str) -> Option<String> {
    let nom = nom.trim();
    let cible = if nom.eq_ignore_ascii_case(SANS_MATIERE)
        || nom.to_lowercase() == SANS_MATIERE.to_lowercase()
    {
        ""
    } else {
        nom
    };
    devoirs
        .iter()
        .find(|d| d.matiere == cible)
        .or_else(|| {
            let bas = cible.to_lowercase();
            devoirs.iter().find(|d| d.matiere.to_lowercase() == bas)
        })
        .map(|d| d.matiere.clone())
}

fn trouver_tache(devoirs: &[Devoir], id: i64) -> R<&Devoir> {
    devoirs
        .iter()
        .find(|d| d.id == id)
        .ok_or_else(|| format!("Tâche introuvable : aucune tâche n'a l'identifiant {id}."))
}

// ------------------------------------------------------------ Consultation

fn consulter(
    nom: &str,
    args: &Map<String, Value>,
    devoirs: &[Devoir],
    jour: NaiveDate,
) -> R<Value> {
    let date = jour.format("%Y-%m-%d").to_string();
    let non_terminees = || devoirs.iter().filter(|d| !termine(d));
    let mut reponse = match nom {
        "list_projects" => {
            let projets: Vec<Value> = par_projet(devoirs)
                .iter()
                .map(|(n, t)| statistiques(n, t, jour))
                .collect();
            json!({ "nombre": projets.len(), "projets": projets })
        }
        "get_project_details" => {
            let projet =
                resoudre_projet(devoirs, &arg_chaine(args, "projet")?.unwrap_or_default())?;
            let taches: Vec<&Devoir> = devoirs.iter().filter(|d| d.matiere == projet).collect();
            let mut v = statistiques(&projet, &taches, jour);
            v["liste"] = vues(taches, jour, LIMITE_MAX as usize);
            v
        }
        "list_tasks" => {
            let projet = arg_chaine(args, "projet")?
                .map(|p| resoudre_projet(devoirs, &p))
                .transpose()?;
            let statut = arg_statut(args)?;
            let priorite_min = arg_entier(args, "priorite_min", 0, 3)?.unwrap_or(0) as u8;
            let recherche = arg_chaine(args, "recherche")?
                .filter(|r| !r.is_empty())
                .map(|r| r.to_lowercase());
            let inclure_terminees =
                arg_booleen(args, "inclure_terminees")? || statut.as_deref() == Some("termine");
            let taches = devoirs
                .iter()
                .filter(|d| projet.as_ref().is_none_or(|p| &d.matiere == p))
                .filter(|d| statut.as_ref().is_none_or(|s| &d.statut == s))
                .filter(|d| inclure_terminees || !termine(d))
                .filter(|d| d.priorite >= priorite_min)
                .filter(|d| {
                    recherche.as_ref().is_none_or(|r| {
                        d.titre.to_lowercase().contains(r)
                            || d.description.to_lowercase().contains(r)
                            || d.todos.iter().any(|t| t.texte.to_lowercase().contains(r))
                    })
                })
                .collect();
            vues(taches, jour, arg_limite(args)?)
        }
        "get_task_details" => vue_tache(trouver_tache(devoirs, arg_id(args)?)?, jour, true),
        "get_today_tasks" => {
            let mut v = vues(
                non_terminees()
                    .filter(|d| date_de(d) == Some(jour))
                    .collect(),
                jour,
                LIMITE_MAX as usize,
            );
            v["en_retard"] = json!(non_terminees().filter(|d| en_retard(d, jour)).count());
            v
        }
        "get_upcoming_tasks" => {
            let jours = arg_entier(args, "jours", 1, 90)?.unwrap_or(7);
            let fin = jour + chrono::Duration::days(jours);
            let mut v = vues(
                non_terminees()
                    .filter(|d| date_de(d).is_some_and(|e| e > jour && e <= fin))
                    .collect(),
                jour,
                LIMITE_MAX as usize,
            );
            v["jusqu_au"] = json!(fin.format("%Y-%m-%d").to_string());
            v
        }
        "get_overdue_tasks" => vues(
            non_terminees().filter(|d| en_retard(d, jour)).collect(),
            jour,
            LIMITE_MAX as usize,
        ),
        "get_completed_tasks" => {
            let projet = arg_chaine(args, "projet")?
                .map(|p| resoudre_projet(devoirs, &p))
                .transpose()?;
            vues(
                devoirs
                    .iter()
                    .filter(|d| termine(d))
                    .filter(|d| projet.as_ref().is_none_or(|p| &d.matiere == p))
                    .collect(),
                jour,
                arg_limite(args)?,
            )
        }
        "get_project_progress" => match arg_chaine(args, "projet")? {
            Some(p) => {
                let projet = resoudre_projet(devoirs, &p)?;
                let taches: Vec<&Devoir> = devoirs.iter().filter(|d| d.matiere == projet).collect();
                statistiques(&projet, &taches, jour)
            }
            None => {
                let toutes: Vec<&Devoir> = devoirs.iter().collect();
                let mut global = statistiques("", &toutes, jour);
                if let Some(o) = global.as_object_mut() {
                    o.remove("projet");
                }
                json!({
                    "global": global,
                    "projets": par_projet(devoirs).iter().map(|(n, t)| statistiques(n, t, jour)).collect::<Vec<_>>(),
                })
            }
        },
        _ => return Err(format!("Outil inconnu : {nom}")),
    };
    reponse["date_du_jour"] = json!(date);
    Ok(reponse)
}

// ------------------------------------------------------------ Modification

fn preparer(
    nom: &str,
    args: &Map<String, Value>,
    devoirs: &[Devoir],
    jour: NaiveDate,
) -> R<(Action, String)> {
    match nom {
        "create_project" => {
            let projet = arg_chaine(args, "nom")?.unwrap_or_default();
            if projet.is_empty() {
                return Err("Le nom du projet ne peut pas être vide.".into());
            }
            if let Some(existant) = trouver_projet(devoirs, &projet) {
                return Err(format!(
                    "Le projet « {} » existe déjà : utilise create_task pour y ajouter une tâche.",
                    nom_projet(&existant)
                ));
            }
            let entree = nouvelle_entree(args, projet)?;
            let resume = format!(
                "Créer le projet « {} » avec la tâche « {} »{}.",
                entree.matiere,
                entree.titre,
                suffixe_echeance(&entree.echeance)
            );
            Ok((Action::Creer(entree), resume))
        }
        "update_project" => {
            let ancienne =
                resoudre_projet(devoirs, &arg_chaine(args, "projet")?.unwrap_or_default())?;
            let nouvelle = arg_chaine(args, "nouveau_nom")?.unwrap_or_default();
            tdb_shared::valider_matiere(&nouvelle)?;
            if nouvelle.is_empty() {
                return Err("Le nouveau nom du projet ne peut pas être vide.".into());
            }
            if nouvelle == ancienne {
                return Err("Le nouveau nom est identique à l'actuel.".into());
            }
            let n = devoirs.iter().filter(|d| d.matiere == ancienne).count();
            let fusion = trouver_projet(devoirs, &nouvelle)
                .filter(|m| *m != ancienne)
                .map(|m| format!(" Attention : ses tâches rejoindront le projet existant « {m} »."))
                .unwrap_or_default();
            let resume = format!(
                "Renommer le projet « {} » en « {nouvelle} » ({n} tâche(s) concernée(s)).{fusion}",
                nom_projet(&ancienne)
            );
            Ok((Action::Renommer { ancienne, nouvelle }, resume))
        }
        "create_task" => {
            let demande = arg_chaine(args, "projet")?.unwrap_or_default();
            // Réutilise l'orthographe d'un projet existant (« maths » → « MATHS ») ; sinon nouveau projet.
            let projet = trouver_projet(devoirs, &demande).unwrap_or(demande);
            let entree = nouvelle_entree(args, projet)?;
            let resume = format!(
                "Créer la tâche « {} » dans le projet « {} »{}.",
                entree.titre,
                nom_projet(&entree.matiere),
                suffixe_echeance(&entree.echeance)
            );
            Ok((Action::Creer(entree), resume))
        }
        "update_task" => {
            let id = arg_id(args)?;
            let actuelle = trouver_tache(devoirs, id)?;
            let modification = DevoirModification {
                titre: arg_chaine(args, "titre")?,
                matiere: arg_chaine(args, "projet")?
                    .map(|p| trouver_projet(devoirs, &p).unwrap_or(p)),
                echeance: arg_chaine(args, "echeance")?,
                type_: arg_type(args)?,
                statut: arg_statut(args)?,
                description: arg_chaine(args, "description")?,
                priorite: arg_priorite(args)?,
            };
            let apres = modification.clone().appliquer(actuelle.clone())?;
            let changements = differences(actuelle, &apres);
            if changements.is_empty() {
                return Err(
                    "Aucune modification : indique au moins un champ différent de l'actuel.".into(),
                );
            }
            let resume = format!(
                "Modifier la tâche n°{id} « {} » : {}.",
                actuelle.titre,
                changements.join(", ")
            );
            let _ = jour;
            Ok((Action::Modifier { id, modification }, resume))
        }
        "complete_task" => {
            let id = arg_id(args)?;
            let tache = trouver_tache(devoirs, id)?;
            if termine(tache) {
                return Err(format!(
                    "La tâche n°{id} « {} » est déjà terminée.",
                    tache.titre
                ));
            }
            let modification = DevoirModification {
                statut: Some("termine".into()),
                ..Default::default()
            };
            let resume = format!(
                "Marquer comme terminée la tâche n°{id} « {} » (projet « {} »).",
                tache.titre,
                nom_projet(&tache.matiere)
            );
            Ok((Action::Modifier { id, modification }, resume))
        }
        _ => Err(format!("Outil inconnu : {nom}")),
    }
}

fn nouvelle_entree(args: &Map<String, Value>, matiere: String) -> R<DevoirEntree> {
    let sous_taches = match args.get("sous_taches") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(liste)) => liste
            .iter()
            .map(|v| {
                v.as_str()
                    .map(|t| TodoEntree {
                        texte: t.to_string(),
                        fait: false,
                    })
                    .ok_or_else(|| "« sous_taches » doit être une liste de chaînes.".to_string())
            })
            .collect::<R<Vec<_>>>()?,
        Some(_) => return Err("« sous_taches » doit être une liste de chaînes.".into()),
    };
    let entree = DevoirEntree {
        titre: arg_chaine(args, "titre")?.unwrap_or_default(),
        matiere: if matiere.eq_ignore_ascii_case(SANS_MATIERE) {
            String::new()
        } else {
            matiere
        },
        echeance: arg_chaine(args, "echeance")?.unwrap_or_default(),
        type_: arg_type(args)?.unwrap_or_else(|| "devoir".into()),
        statut: arg_statut(args)?.unwrap_or_else(|| "a_faire".into()),
        tags: Vec::new(),
        todos: sous_taches,
        description: arg_chaine(args, "description")?.unwrap_or_default(),
        priorite: arg_priorite(args)?.unwrap_or(0),
    };
    entree.valider()
}

fn suffixe_echeance(echeance: &str) -> String {
    if echeance.is_empty() {
        " (sans échéance)".into()
    } else {
        format!(" pour le {echeance}")
    }
}

fn differences(avant: &Devoir, apres: &Devoir) -> Vec<String> {
    let mut d = Vec::new();
    let mut champ = |nom: &str, a: &str, b: &str| {
        if a != b {
            d.push(format!("{nom} « {a} » → « {b} »"));
        }
    };
    champ("titre", &avant.titre, &apres.titre);
    champ(
        "projet",
        nom_projet(&avant.matiere),
        nom_projet(&apres.matiere),
    );
    champ("échéance", &avant.echeance, &apres.echeance);
    champ("type", &avant.type_, &apres.type_);
    champ("statut", &avant.statut, &apres.statut);
    champ(
        "priorité",
        libelle_priorite(avant.priorite),
        libelle_priorite(apres.priorite),
    );
    if avant.description != apres.description {
        d.push("description modifiée".into());
    }
    d
}

fn appliquer(db: &Db, action: Action, jour: NaiveDate) -> R<Value> {
    let erreur = |e: tdb_core::Erreur| e.to_string();
    match action {
        Action::Creer(entree) => {
            let d = db.creer_devoir(entree).map_err(erreur)?;
            Ok(json!({ "effectue": true, "tache": vue_tache(&d, jour, true) }))
        }
        Action::Renommer { ancienne, nouvelle } => {
            let n = db.renommer_matiere(&ancienne, &nouvelle).map_err(erreur)?;
            Ok(json!({ "effectue": true, "projet": nouvelle, "taches_modifiees": n }))
        }
        Action::Modifier { id, modification } => {
            let d = db.modifier_devoir(id, modification).map_err(erreur)?;
            Ok(json!({ "effectue": true, "tache": vue_tache(&d, jour, true) }))
        }
    }
}
