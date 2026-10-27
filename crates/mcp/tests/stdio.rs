//! Test de bout en bout : le binaire `tdb-mcp` est lancé comme le ferait Hermes Agent
//! (stdio, JSON-RPC ligne par ligne) sur une base créée par le cœur de l'application.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};
use tdb_core::db::Db;
use tdb_shared::{
    DevoirEntree, DevoirModification, TodoEntree, PARAM_MCP_ACTIF, PARAM_MCP_ECRITURE,
};

struct Client {
    processus: Child,
    entree: ChildStdin,
    sortie: BufReader<ChildStdout>,
    id: u64,
}

impl Client {
    fn lancer(base: &PathBuf) -> Self {
        let mut processus = Command::new(env!("CARGO_BIN_EXE_tdb-mcp"))
            .arg("--base")
            .arg(base)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("lancement de tdb-mcp");
        let entree = processus.stdin.take().unwrap();
        let sortie = BufReader::new(processus.stdout.take().unwrap());
        Client {
            processus,
            entree,
            sortie,
            id: 0,
        }
    }

    fn envoyer(&mut self, message: Value) {
        writeln!(self.entree, "{message}").unwrap();
        self.entree.flush().unwrap();
    }

    fn requete(&mut self, methode: &str, params: Value) -> Value {
        self.id += 1;
        self.envoyer(
            json!({ "jsonrpc": "2.0", "id": self.id, "method": methode, "params": params }),
        );
        let mut ligne = String::new();
        self.sortie.read_line(&mut ligne).unwrap();
        let reponse: Value = serde_json::from_str(&ligne).expect("réponse JSON valide");
        assert_eq!(reponse["id"], json!(self.id), "identifiant de réponse");
        reponse
    }

    /// Appelle un outil ; renvoie (est une erreur, contenu structuré ou texte).
    fn outil(&mut self, nom: &str, arguments: Value) -> (bool, Value) {
        let r = self.requete("tools/call", json!({ "name": nom, "arguments": arguments }));
        let resultat = &r["result"];
        let erreur = resultat["isError"].as_bool().unwrap();
        let texte = resultat["content"][0]["text"].as_str().unwrap().to_string();
        if erreur {
            (true, Value::String(texte))
        } else {
            // Le texte et le contenu structuré décrivent la même chose.
            assert_eq!(
                serde_json::from_str::<Value>(&texte).unwrap(),
                resultat["structuredContent"]
            );
            (false, resultat["structuredContent"].clone())
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.processus.kill();
        let _ = self.processus.wait();
    }
}

fn jour(decalage: i64) -> String {
    (tdb_core::ade::aujourdhui() + chrono::Duration::days(decalage))
        .format("%Y-%m-%d")
        .to_string()
}

fn devoir(
    titre: &str,
    matiere: &str,
    echeance: String,
    statut: &str,
    priorite: u8,
) -> DevoirEntree {
    DevoirEntree {
        titre: titre.into(),
        matiere: matiere.into(),
        echeance,
        type_: "devoir".into(),
        statut: statut.into(),
        tags: Vec::new(),
        todos: vec![
            TodoEntree {
                texte: "Lire le cours".into(),
                fait: true,
            },
            TodoEntree {
                texte: "Exercices".into(),
                fait: false,
            },
        ],
        description: "Chapitre 3".into(),
        priorite,
    }
}

fn base_temporaire(nom: &str) -> (PathBuf, Db) {
    let dossier = std::env::temp_dir().join(format!("tdb-mcp-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dossier);
    std::fs::create_dir_all(&dossier).unwrap();
    let chemin = dossier.join("tableau-de-bord.db");
    let app = Db::ouvrir(&chemin).unwrap();
    (chemin, app)
}

#[test]
fn protocole_et_consultation() {
    let (base, app) = base_temporaire("lecture");
    let retard = app
        .creer_devoir(devoir("TP noté", "PHYSIQUE", jour(-2), "en_cours", 2))
        .unwrap();
    let today = app
        .creer_devoir(devoir("Exercices 4 à 9", "MATHS", jour(0), "a_faire", 3))
        .unwrap();
    app.creer_devoir(devoir("DS d'algèbre", "MATHS", jour(3), "a_faire", 1))
        .unwrap();
    app.creer_devoir(devoir("Exposé", "", jour(20), "a_faire", 0))
        .unwrap();
    app.creer_devoir(devoir("Fiche de lecture", "MATHS", jour(-5), "termine", 0))
        .unwrap();

    let mut c = Client::lancer(&base);
    let init = c.requete(
        "initialize",
        json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "1" } }),
    );
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(init["result"]["serverInfo"]["name"], "tableau-de-bord");
    assert!(init["result"]["capabilities"]["tools"].is_object());
    // Une notification ne reçoit pas de réponse : la requête suivante obtient bien la sienne.
    c.envoyer(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
    assert!(c.requete("ping", json!({}))["result"].is_object());

    let outils = c.requete("tools/list", json!({}));
    let noms: Vec<&str> = outils["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(noms.len(), 14);
    for attendu in [
        "list_projects",
        "get_today_tasks",
        "get_overdue_tasks",
        "create_task",
        "complete_task",
    ] {
        assert!(noms.contains(&attendu), "{attendu} absent");
    }

    // Désactivée par défaut : tout appel est refusé avec un message clair.
    let (erreur, message) = c.outil("list_projects", json!({}));
    assert!(erreur);
    assert!(message.as_str().unwrap().contains("désactivée"));

    app.ecrire_parametre(PARAM_MCP_ACTIF, "1").unwrap();
    let (erreur, projets) = c.outil("list_projects", json!({}));
    assert!(!erreur, "{projets}");
    assert_eq!(projets["nombre"], 3);
    let maths = projets["projets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["projet"] == "MATHS")
        .unwrap();
    assert_eq!(maths["taches"], 3);
    assert_eq!(maths["terminees"], 1);
    assert_eq!(maths["progression_pourcent"], 33);
    assert_eq!(maths["prochaine_echeance"]["id"], today.id);
    assert!(projets["projets"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["projet"] == "Sans matière"));

    let (_, aujourd_hui) = c.outil("get_today_tasks", json!({}));
    assert_eq!(aujourd_hui["date_du_jour"], jour(0));
    assert_eq!(aujourd_hui["nombre"], 1);
    assert_eq!(aujourd_hui["taches"][0]["titre"], "Exercices 4 à 9");
    assert_eq!(aujourd_hui["en_retard"], 1);

    let (_, retards) = c.outil("get_overdue_tasks", json!({}));
    assert_eq!(
        retards["nombre"], 1,
        "une tâche terminée n'est jamais en retard"
    );
    assert_eq!(retards["taches"][0]["id"], retard.id);
    assert_eq!(retards["taches"][0]["jours_restants"], -2);

    let (_, prochains) = c.outil("get_upcoming_tasks", json!({ "jours": 7 }));
    assert_eq!(prochains["nombre"], 1);
    let (_, prochains) = c.outil("get_upcoming_tasks", json!({ "jours": "30" }));
    assert_eq!(prochains["nombre"], 2, "nombre transmis en texte accepté");

    // Tri par urgence : retard d'abord, puis aujourd'hui.
    let (_, taches) = c.outil("list_tasks", json!({}));
    assert_eq!(taches["nombre"], 4);
    assert_eq!(taches["taches"][0]["id"], retard.id);
    assert_eq!(taches["taches"][1]["id"], today.id);
    let (_, filtrees) = c.outil(
        "list_tasks",
        json!({ "projet": "maths", "priorite_min": 2 }),
    );
    assert_eq!(filtrees["nombre"], 1, "projet insensible à la casse");
    let (_, trouvees) = c.outil("list_tasks", json!({ "recherche": "algèbre" }));
    assert_eq!(trouvees["nombre"], 1);

    let (_, detail) = c.outil("get_task_details", json!({ "id": today.id }));
    assert_eq!(detail["description"], "Chapitre 3");
    assert_eq!(detail["sous_taches"]["faites"], 1);
    assert_eq!(detail["sous_taches_liste"][1]["texte"], "Exercices");

    let (_, projet) = c.outil("get_project_details", json!({ "projet": "Sans matière" }));
    assert_eq!(projet["taches"], 1);
    let (_, faites) = c.outil("get_completed_tasks", json!({}));
    assert_eq!(faites["nombre"], 1);
    let (_, avancement) = c.outil("get_project_progress", json!({}));
    assert_eq!(avancement["global"]["taches"], 5);
    assert_eq!(avancement["global"]["sous_taches"]["total"], 10);

    // Données actualisées : une modification faite dans l'application est vue au prochain appel.
    app.modifier_devoir(
        retard.id,
        DevoirModification {
            statut: Some("termine".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let (_, retards) = c.outil("get_overdue_tasks", json!({}));
    assert_eq!(retards["nombre"], 0);

    // Validation des paramètres.
    let (erreur, message) = c.outil("get_task_details", json!({ "id": 99999 }));
    assert!(erreur && message.as_str().unwrap().contains("introuvable"));
    let (erreur, message) = c.outil("list_tasks", json!({ "chemin": "C:\\Windows" }));
    assert!(erreur && message.as_str().unwrap().contains("Argument inconnu"));
    let (erreur, _) = c.outil("get_upcoming_tasks", json!({ "jours": 1000 }));
    assert!(erreur);
    let (erreur, message) = c.outil("get_project_details", json!({ "projet": "CHIMIE" }));
    assert!(erreur && message.as_str().unwrap().contains("MATHS"));
    let inconnu = c.requete(
        "tools/call",
        json!({ "name": "lire_fichier", "arguments": {} }),
    );
    assert_eq!(inconnu["error"]["code"], -32602);
    assert_eq!(
        c.requete("resources/read", json!({}))["error"]["code"],
        -32601
    );

    drop(c);
    let _ = std::fs::remove_dir_all(base.parent().unwrap());
}

#[test]
fn modifications_autorisees_et_confirmees() {
    let (base, app) = base_temporaire("ecriture");
    let tache = app
        .creer_devoir(devoir("Exercices", "MATHS", jour(1), "a_faire", 0))
        .unwrap();
    app.ecrire_parametre(PARAM_MCP_ACTIF, "1").unwrap();
    let mut c = Client::lancer(&base);

    // Lecture seule par défaut.
    let creation = json!({ "titre": "Réviser le chapitre 2", "projet": "maths", "echeance": jour(4), "priorite": 2, "sous_taches": ["Fiche", "Annales"] });
    let (erreur, message) = c.outil("create_task", creation.clone());
    assert!(erreur && message.as_str().unwrap().contains("lecture seule"));

    app.ecrire_parametre(PARAM_MCP_ECRITURE, "1").unwrap();
    // Premier appel : aperçu et jeton, rien n'est écrit.
    let (erreur, apercu) = c.outil("create_task", creation.clone());
    assert!(!erreur, "{apercu}");
    assert_eq!(apercu["confirmation_requise"], true);
    assert!(
        apercu["action"].as_str().unwrap().contains("« MATHS »"),
        "orthographe du projet existant reprise"
    );
    let jeton = apercu["confirmation"].as_str().unwrap().to_string();
    assert_eq!(app.lister_devoirs().unwrap().len(), 1);

    // Jeton présenté avec d'autres arguments : refusé.
    let mut autre = creation.clone();
    autre["titre"] = json!("Autre chose");
    autre["confirmation"] = json!(jeton);
    let (erreur, _) = c.outil("create_task", autre);
    assert!(erreur);
    assert_eq!(app.lister_devoirs().unwrap().len(), 1);

    // Confirmation : la tâche est créée, visible immédiatement dans l'application.
    let mut confirme = creation.clone();
    confirme["confirmation"] = json!(jeton);
    let (erreur, cree) = c.outil("create_task", confirme.clone());
    assert!(!erreur, "{cree}");
    assert_eq!(cree["tache"]["projet"], "MATHS");
    assert_eq!(cree["tache"]["sous_taches"]["total"], 2);
    assert_eq!(app.lister_devoirs().unwrap().len(), 2);
    // Jeton à usage unique.
    let (erreur, message) = c.outil("create_task", confirme);
    assert!(erreur && message.as_str().unwrap().contains("invalide"));
    assert_eq!(app.lister_devoirs().unwrap().len(), 2);

    // Données invalides : refusées avant toute demande de confirmation.
    let (erreur, message) = c.outil(
        "create_task",
        json!({ "titre": "X", "projet": "MATHS", "echeance": "demain" }),
    );
    assert!(erreur && message.as_str().unwrap().contains("AAAA-MM-JJ"));

    // complete_task.
    let (_, apercu) = c.outil("complete_task", json!({ "id": tache.id }));
    let jeton = apercu["confirmation"].as_str().unwrap();
    let (erreur, fait) = c.outil(
        "complete_task",
        json!({ "id": tache.id, "confirmation": jeton }),
    );
    assert!(!erreur, "{fait}");
    assert_eq!(fait["tache"]["statut"], "termine");
    let (erreur, _) = c.outil("complete_task", json!({ "id": tache.id }));
    assert!(erreur, "déjà terminée");

    // update_task.
    let (_, apercu) = c.outil(
        "update_task",
        json!({ "id": tache.id, "priorite": 3, "statut": "en_cours" }),
    );
    assert!(apercu["action"]
        .as_str()
        .unwrap()
        .contains("statut « termine » → « en_cours »"));
    let jeton = apercu["confirmation"].as_str().unwrap();
    let (erreur, _) = c.outil(
        "update_task",
        json!({ "id": tache.id, "priorite": 3, "statut": "en_cours", "confirmation": jeton }),
    );
    assert!(!erreur);
    let modifie = app
        .lister_devoirs()
        .unwrap()
        .into_iter()
        .find(|d| d.id == tache.id)
        .unwrap();
    assert_eq!((modifie.priorite, modifie.statut.as_str()), (3, "en_cours"));

    // update_project : renommage sur toutes les tâches de la matière.
    let (_, apercu) = c.outil(
        "update_project",
        json!({ "projet": "MATHS", "nouveau_nom": "Mathématiques" }),
    );
    assert!(apercu["action"].as_str().unwrap().contains("2 tâche(s)"));
    let jeton = apercu["confirmation"].as_str().unwrap();
    let (erreur, renomme) = c.outil(
        "update_project",
        json!({ "projet": "MATHS", "nouveau_nom": "Mathématiques", "confirmation": jeton }),
    );
    assert!(!erreur, "{renomme}");
    assert_eq!(renomme["taches_modifiees"], 2);
    assert!(app
        .lister_devoirs()
        .unwrap()
        .iter()
        .all(|d| d.matiere == "Mathématiques"));

    // create_project : refusé si le projet existe, sinon créé avec sa première tâche.
    let (erreur, _) = c.outil(
        "create_project",
        json!({ "nom": "mathématiques", "titre": "X" }),
    );
    assert!(erreur);
    let (_, apercu) = c.outil(
        "create_project",
        json!({ "nom": "Histoire", "titre": "Frise chronologique" }),
    );
    let jeton = apercu["confirmation"].as_str().unwrap();
    let (erreur, _) = c.outil(
        "create_project",
        json!({ "nom": "Histoire", "titre": "Frise chronologique", "confirmation": jeton }),
    );
    assert!(!erreur);
    assert!(app
        .lister_devoirs()
        .unwrap()
        .iter()
        .any(|d| d.matiere == "Histoire"));

    // Écriture retirée dans l'application : un jeton déjà émis ne suffit plus.
    let (_, apercu) = c.outil("complete_task", json!({ "id": tache.id }));
    let jeton = apercu["confirmation"].as_str().unwrap().to_string();
    app.ecrire_parametre(PARAM_MCP_ECRITURE, "0").unwrap();
    let (erreur, _) = c.outil(
        "complete_task",
        json!({ "id": tache.id, "confirmation": jeton }),
    );
    assert!(erreur);
    assert_ne!(
        app.lister_devoirs()
            .unwrap()
            .into_iter()
            .find(|d| d.id == tache.id)
            .unwrap()
            .statut,
        "termine"
    );

    // Journal écrit à côté de la base, jamais sur la sortie standard.
    drop(c);
    let journal = std::fs::read_to_string(base.parent().unwrap().join("logs/mcp.log")).unwrap();
    assert!(journal.contains("create_task : ok"));
    let _ = std::fs::remove_dir_all(base.parent().unwrap());
}

#[test]
fn base_absente_message_clair() {
    let dossier = std::env::temp_dir().join(format!("tdb-mcp-absente-{}", std::process::id()));
    let base = dossier.join("tableau-de-bord.db");
    let mut c = Client::lancer(&base);
    let (erreur, message) = c.outil("list_projects", json!({}));
    assert!(erreur);
    assert!(message.as_str().unwrap().contains("lance l'application"));
    assert!(!base.exists(), "le serveur MCP ne crée jamais de base");
}
