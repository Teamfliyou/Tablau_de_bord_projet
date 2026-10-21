//! Emploi du temps ADE Campus : validation d'URL (anti-SSRF), téléchargement,
//! analyse iCalendar, reconnaissance des matières / groupes / types et filtres.

use crate::error::{validation, Erreur, Resultat};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::{Europe::Paris, Tz};
use regex::Regex;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::OnceLock;
use url::{Host, Url};

pub const TAILLE_MAX_ICS: usize = 10 * 1024 * 1024;
pub const DELAI_REQUETE_SECONDES: u64 = 20;

pub use tdb_shared::{Categories, Cours, CoursDuJour, CoursSemaine, Filtres};

// ------------------------------------------------------------------ SSRF

fn ipv4_interdite(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
        || ip.is_unspecified()
        || ip.is_documentation()
        || o[0] == 0
        || o[0] >= 240
        || (o[0] == 100 && (64..128).contains(&o[1])) // CGNAT 100.64.0.0/10
        || (o[0] == 198 && (o[1] == 18 || o[1] == 19)) // tests de performance
}

pub fn adresse_interdite(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => ipv4_interdite(v4),
        IpAddr::V6(v6) => ipv6_interdite(v6),
    }
}

fn ipv6_interdite(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return ipv4_interdite(v4);
    }
    let s = ip.segments();
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (s[0] & 0xfe00) == 0xfc00 // unique local fc00::/7
        || (s[0] & 0xffc0) == 0xfe80 // link-local fe80::/10
        || (s[0] == 0x2001 && s[1] == 0x0db8) // documentation
        || (s[0] == 0x0064 && s[1] == 0xff9b) // NAT64
}

/// Contrôles purement syntaxiques (sans réseau) : HTTPS, hôte public, pas d'identifiants.
pub fn valider_syntaxe_url(brut: &str) -> Resultat<Url> {
    let brut = brut.trim();
    if brut.is_empty() {
        return validation("Lien ADE vide.");
    }
    let url = Url::parse(brut)
        .map_err(|_| Erreur::Validation("Le lien ADE doit être une URL HTTPS valide.".into()))?;
    if url.scheme() != "https" {
        return validation("Le lien ADE doit être une URL HTTPS valide.");
    }
    if !url.username().is_empty() || url.password().is_some() {
        return validation("Le lien ADE ne doit pas contenir d'identifiants.");
    }
    match url.host() {
        None => return validation("Le lien ADE doit être une URL HTTPS valide."),
        Some(Host::Domain(d)) => {
            let d = d.trim_end_matches('.').to_lowercase();
            if d == "localhost"
                || d.ends_with(".localhost")
                || d.ends_with(".local")
                || d.ends_with(".internal")
                || !d.contains('.')
            {
                return validation("Hôte ADE non autorisé.");
            }
        }
        Some(Host::Ipv4(ip)) => {
            if ipv4_interdite(ip) {
                return validation("Adresse réseau ADE non autorisée.");
            }
        }
        Some(Host::Ipv6(ip)) => {
            if ipv6_interdite(ip) {
                return validation("Adresse réseau ADE non autorisée.");
            }
        }
    }
    Ok(url)
}

/// Validation complète : syntaxe + résolution DNS dont toutes les adresses doivent être publiques.
/// Les adresses retournées sont ensuite imposées au client HTTP (anti DNS-rebinding).
pub async fn valider_url(brut: &str) -> Resultat<(Url, Vec<SocketAddr>)> {
    let url = valider_syntaxe_url(brut)?;
    let port = url.port_or_known_default().unwrap_or(443);
    let adresses: Vec<SocketAddr> = match url.host() {
        Some(Host::Ipv4(ip)) => vec![SocketAddr::new(IpAddr::V4(ip), port)],
        Some(Host::Ipv6(ip)) => vec![SocketAddr::new(IpAddr::V6(ip), port)],
        Some(Host::Domain(d)) => tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::net::lookup_host((d.to_string(), port)),
        )
        .await
        .map_err(|_| Erreur::Reseau("La résolution du domaine ADE a expiré.".into()))?
        .map_err(|_| {
            Erreur::Reseau(
                "Le domaine ADE est introuvable (connexion Internet indisponible ?).".into(),
            )
        })?
        .collect(),
        None => vec![],
    };
    if adresses.is_empty() || adresses.iter().any(|a| adresse_interdite(a.ip())) {
        return validation("Adresse réseau ADE non autorisée.");
    }
    Ok((url, adresses))
}

/// Télécharge le flux ICS : pas de redirection, délais bornés, taille limitée.
pub async fn telecharger_ics(brut: &str) -> Resultat<String> {
    let (url, adresses) = valider_url(brut).await?;
    let mut builder = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(DELAI_REQUETE_SECONDES))
        .connect_timeout(std::time::Duration::from_secs(10))
        .user_agent("TableauDeBordEtudiant/4.0");
    if let Some(Host::Domain(d)) = url.host() {
        builder = builder.resolve_to_addrs(d, &adresses);
    }
    let client = builder
        .build()
        .map_err(|_| Erreur::Interne("Impossible d'initialiser le client réseau.".into()))?;
    let mut reponse = client.get(url).send().await.map_err(erreur_reseau)?;
    let statut = reponse.status();
    if statut.is_redirection() {
        return Err(Erreur::Reseau(
            "Les redirections du flux ADE ne sont pas autorisées.".into(),
        ));
    }
    if !statut.is_success() {
        return Err(Erreur::Reseau(format!(
            "Le serveur ADE a répondu avec l'erreur {}.",
            statut.as_u16()
        )));
    }
    if reponse
        .content_length()
        .is_some_and(|n| n as usize > TAILLE_MAX_ICS)
    {
        return Err(Erreur::Reseau("Le flux ADE est trop volumineux.".into()));
    }
    let mut octets: Vec<u8> = Vec::new();
    while let Some(morceau) = reponse.chunk().await.map_err(erreur_reseau)? {
        octets.extend_from_slice(&morceau);
        if octets.len() > TAILLE_MAX_ICS {
            return Err(Erreur::Reseau("Le flux ADE est trop volumineux.".into()));
        }
    }
    Ok(String::from_utf8_lossy(&octets).into_owned())
}

fn erreur_reseau(e: reqwest::Error) -> Erreur {
    if e.is_timeout() {
        Erreur::Reseau("Le serveur ADE ne répond pas (délai dépassé).".into())
    } else {
        Erreur::Reseau("Impossible de joindre le serveur ADE.".into())
    }
}

// ------------------------------------------------------------ Reconnaissance

fn rx(motif: &str) -> Regex {
    Regex::new(motif).expect("expression régulière valide")
}

struct Motifs {
    groupes: Vec<Regex>,
    types: Vec<Regex>,
    marqueurs_groupe: Regex,
    numero_final: Regex,
    ligne_groupe: Regex,
    ligne_prof: Regex,
    parentheses: Regex,
    espaces: Regex,
}

fn motifs() -> &'static Motifs {
    static M: OnceLock<Motifs> = OnceLock::new();
    M.get_or_init(|| Motifs {
        groupes: vec![
            rx(r"(?i)\b(?:TP|TD)[ ]*(\d+|[A-Z])\b"),
            rx(r"(?i)\bGr(?:oupe)?s?[ ]+(?:\d+|[A-Z])\b"),
            rx(r"(?i)\bParcours[ ]+([A-Z0-9]+)\b"),
            rx(r"(?i)\bPromo[ ]+([A-Z]+)\b"),
            rx(r"(?i)\b(CP[12]|ING[12])\b"),
        ],
        types: vec![
            rx(r"(?i)\b(Kh[oô]lle)\b"),
            rx(r"\b(DS)\b"),
            rx(r"\b(CM)\b"),
            rx(r"\b(TD)(?:[ ]*\d+)?\b"),
            rx(r"\b(TP)(?:[ ]*\d+)?\b"),
            rx(r"\b(IE)\b"),
            rx(r"\b(CC)\b"),
            rx(r"(?i)\b(TRAVAUX[ ]+PRATIQUES)\b"),
            rx(r"(?i)\b(TRAVAUX[ ]+DIRIGES)\b"),
            rx(r"(?i)\b(COURS[ ]+MAGISTRAL)\b"),
            rx(r"(?i)\b(PROJET|ATELIER|AMPHI)\b"),
            rx(r"(?i)\b(Examen|Exam)\b"),
            rx(r"(?i)\b(Contr[oô]le)\b"),
            rx(r"(?i)\b(Oral)\b"),
            rx(r"(?i)\b(PRESENTATION)\b"),
            rx(r"(?i)\b(REUNION)\b"),
            rx(r"(?i)\b(RENTREE)\b"),
            rx(r"(?i)\b(OLYMPIADES)\b"),
            rx(r"(?i)\b(TEDS)\b"),
            rx(r"(?i)\b(WEC)\b"),
        ],
        marqueurs_groupe: rx(
            r"(?i)\((?:TP|TD|CM|DS)[^)]*\)|Gr(?:oupe)?s?[ ]+(?:\d+|[A-Z])|Parcours[ ]+[A-Z0-9]+|Promo[ ]+[A-Z]+|\b(?:CP[12]|ING[12])\b",
        ),
        numero_final: rx(r"\s+\d+(?:\.\d+)*\s*$"),
        ligne_groupe: rx(
            r"(?i)^(?:[A-Z]{2,3}\s*\d|ING\d|CP[12]|TD|TP|CM|PROMO|PARCOURS|FISE|GEE|GI|DK)\b",
        ),
        ligne_prof: rx(r"^[A-ZÀ-ÖØ-Þ][A-ZÀ-ÖØ-Þ'’\- ]+ +[A-ZÀ-ÖØ-Þ][A-ZÀ-ÖØ-Þa-zà-öø-ÿ'’\- ]+$"),
        parentheses: rx(r"\([^)]*\)"),
        espaces: rx(r"\s+"),
    })
}

fn extraire_matches(texte: &str, regexes: &[Regex]) -> Vec<String> {
    let mut resultats: Vec<String> = Vec::new();
    for re in regexes {
        for c in re.captures_iter(texte) {
            let m = c.get(1).or_else(|| c.get(0));
            if let Some(m) = m {
                let v = m.as_str().trim().to_uppercase();
                if !v.is_empty() && !resultats.contains(&v) {
                    resultats.push(v);
                }
            }
        }
    }
    resultats
}

pub fn detecter_groupes(titre: &str, description: &str) -> Vec<String> {
    extraire_matches(&format!("{titre} {description}"), &motifs().groupes)
}

pub fn detecter_types(titre: &str, description: &str) -> Vec<String> {
    extraire_matches(&format!("{titre} {description}"), &motifs().types)
}

pub fn extraire_matiere(titre: &str) -> String {
    let m = motifs();
    let sans_parentheses = m.parentheses.replace_all(titre, " ");
    let avant_tiret = sans_parentheses.split(" - ").next().unwrap_or("");
    let sans_marqueurs = m.marqueurs_groupe.replace_all(avant_tiret, " ");
    let sans_numero = m.numero_final.replace(&sans_marqueurs, "");
    m.espaces
        .replace_all(&sans_numero, " ")
        .trim()
        .to_uppercase()
}

pub fn extraire_enseignants(description: &str) -> Vec<String> {
    let m = motifs();
    let mut v: Vec<String> = Vec::new();
    for ligne in description.lines() {
        let ligne = ligne.trim();
        if ligne.is_empty() || ligne.starts_with('(') || m.ligne_groupe.is_match(ligne) {
            continue;
        }
        if m.ligne_prof.is_match(ligne) && !v.iter().any(|x| x == ligne) {
            v.push(ligne.to_string());
        }
    }
    v
}

// ------------------------------------------------------------------ iCalendar

fn desechapper(valeur: &str) -> String {
    let mut sortie = String::with_capacity(valeur.len());
    let mut it = valeur.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') | Some('N') => sortie.push('\n'),
                Some(autre) => sortie.push(autre),
                None => {}
            }
        } else {
            sortie.push(c);
        }
    }
    sortie
}

/// Convertit une valeur DTSTART/DTEND en date-heure Europe/Paris.
fn parser_date_ical(valeur: &str, tzid: Option<&str>, jour_entier: bool) -> Option<DateTime<Tz>> {
    let v = valeur.trim();
    if jour_entier || v.len() == 8 {
        let d = NaiveDate::parse_from_str(&v[..8.min(v.len())], "%Y%m%d").ok()?;
        return Paris
            .from_local_datetime(&d.and_time(NaiveTime::MIN))
            .earliest();
    }
    if let Some(utc) = v.strip_suffix('Z') {
        let n = NaiveDateTime::parse_from_str(utc, "%Y%m%dT%H%M%S").ok()?;
        return Some(Utc.from_utc_datetime(&n).with_timezone(&Paris));
    }
    let n = NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M%S").ok()?;
    let zone: Tz = tzid
        .and_then(|t| t.trim_matches('"').parse().ok())
        .unwrap_or(Paris);
    zone.from_local_datetime(&n)
        .earliest()
        .map(|d| d.with_timezone(&Paris))
}

pub fn parser_cours(contenu: &str) -> Resultat<Vec<Cours>> {
    let mut cours: Vec<Cours> = Vec::new();
    let lecteur = ical::IcalParser::new(contenu.as_bytes());
    let mut calendriers = 0;
    for calendrier in lecteur {
        let calendrier = calendrier
            .map_err(|e| Erreur::Reseau(format!("Impossible de lire le fichier ICS : {e}")))?;
        calendriers += 1;
        for ev in calendrier.events {
            let prop = |nom: &str| {
                ev.properties
                    .iter()
                    .find(|p| p.name.eq_ignore_ascii_case(nom))
            };
            let texte = |nom: &str| {
                prop(nom)
                    .and_then(|p| p.value.as_deref())
                    .map(desechapper)
                    .unwrap_or_default()
            };
            let param = |p: &ical::property::Property, nom: &str| -> Option<String> {
                p.params
                    .as_ref()?
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(nom))
                    .and_then(|(_, v)| v.first().cloned())
            };
            let Some(p_debut) = prop("DTSTART") else {
                continue;
            };
            let jour_entier =
                param(p_debut, "VALUE").is_some_and(|v| v.eq_ignore_ascii_case("DATE"));
            let Some(debut) = p_debut
                .value
                .as_deref()
                .and_then(|v| parser_date_ical(v, param(p_debut, "TZID").as_deref(), jour_entier))
            else {
                continue;
            };
            let fin = prop("DTEND")
                .and_then(|p| {
                    let entier = param(p, "VALUE").is_some_and(|v| v.eq_ignore_ascii_case("DATE"));
                    p.value
                        .as_deref()
                        .and_then(|v| parser_date_ical(v, param(p, "TZID").as_deref(), entier))
                })
                .unwrap_or_else(|| {
                    if jour_entier {
                        debut + Duration::days(1)
                    } else {
                        debut
                    }
                });

            let titre = {
                let t = texte("SUMMARY");
                let t = t.trim();
                if t.is_empty() {
                    "Cours sans titre".to_string()
                } else {
                    t.to_string()
                }
            };
            let description = texte("DESCRIPTION").trim().to_string();
            let enseignants = extraire_enseignants(&description);
            cours.push(Cours {
                salle: texte("LOCATION").trim().to_string(),
                enseignant: enseignants.join(", "),
                debut: debut.format("%H:%M").to_string(),
                fin: fin.format("%H:%M").to_string(),
                debut_iso: debut.to_rfc3339(),
                fin_iso: fin.to_rfc3339(),
                date: debut.date_naive().to_string(),
                groupes: detecter_groupes(&titre, &description),
                types: detecter_types(&titre, &description),
                matiere: extraire_matiere(&titre),
                enseignants,
                titre,
            });
        }
    }
    if calendriers == 0 && !contenu.trim().is_empty() {
        return Err(Erreur::Reseau(
            "Le flux ADE ne contient aucun calendrier valide.".into(),
        ));
    }
    cours.sort_by(|a, b| a.debut_iso.cmp(&b.debut_iso));
    Ok(cours)
}

// --------------------------------------------------------------------- Filtres

fn normaliser(v: &[String]) -> Vec<String> {
    v.iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Même sémantique que l'API historique : un cours sans valeur pour une dimension n'est jamais exclu.
pub fn appliquer_filtres(cours: Vec<Cours>, f: &Filtres) -> Vec<Cours> {
    let (grp, typ, mat) = (
        normaliser(&f.groupes),
        normaliser(&f.types),
        normaliser(&f.matieres),
    );
    cours
        .into_iter()
        .filter(|c| {
            if !grp.is_empty() {
                let g: Vec<String> = c.groupes.iter().map(|x| x.to_lowercase()).collect();
                if !g.is_empty() && !g.iter().any(|x| grp.contains(x)) {
                    return false;
                }
            }
            if !typ.is_empty() {
                let t: Vec<String> = c.types.iter().map(|x| x.to_lowercase()).collect();
                if !t.is_empty() && !t.iter().any(|x| typ.contains(x)) {
                    return false;
                }
            }
            if !mat.is_empty() {
                let m = c.matiere.to_lowercase();
                if !m.is_empty() && !mat.contains(&m) {
                    return false;
                }
            }
            true
        })
        .collect()
}

pub fn analyser_categories(cours: &[Cours]) -> Categories {
    use std::collections::BTreeSet;
    let mut g = BTreeSet::new();
    let mut t = BTreeSet::new();
    let mut m = BTreeSet::new();
    for c in cours {
        g.extend(c.groupes.iter().filter(|x| !x.is_empty()).cloned());
        t.extend(c.types.iter().filter(|x| !x.is_empty()).cloned());
        if !c.matiere.is_empty() {
            m.insert(c.matiere.clone());
        }
    }
    Categories {
        groupes: g.into_iter().collect(),
        types: t.into_iter().collect(),
        matieres: m.into_iter().collect(),
    }
}

pub fn aujourdhui() -> NaiveDate {
    Utc::now().with_timezone(&Paris).date_naive()
}

pub fn parser_jour(valeur: Option<&str>) -> Resultat<Option<NaiveDate>> {
    match valeur.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(v) => NaiveDate::parse_from_str(v, "%Y-%m-%d")
            .map(Some)
            .map_err(|_| {
                Erreur::Validation(format!("Date invalide : {v} (format attendu AAAA-MM-JJ)"))
            }),
    }
}

pub fn cours_du_jour(tous: Vec<Cours>, jour: Option<NaiveDate>, f: &Filtres) -> CoursDuJour {
    let jour = jour.unwrap_or_else(aujourdhui).to_string();
    let cours = appliquer_filtres(tous.into_iter().filter(|c| c.date == jour).collect(), f);
    CoursDuJour {
        date: jour,
        nb_cours: cours.len(),
        cours,
    }
}

/// Semaine du lundi au vendredi contenant `ref_`.
pub fn cours_semaine(tous: Vec<Cours>, ref_: Option<NaiveDate>, f: &Filtres) -> CoursSemaine {
    let ref_ = ref_.unwrap_or_else(aujourdhui);
    let lundi = ref_ - Duration::days(ref_.weekday().num_days_from_monday() as i64);
    let vendredi = lundi + Duration::days(4);
    let cours = appliquer_filtres(
        tous.into_iter()
            .filter(|c| {
                NaiveDate::parse_from_str(&c.date, "%Y-%m-%d")
                    .is_ok_and(|d| d >= lundi && d <= vendredi)
            })
            .collect(),
        f,
    );
    CoursSemaine {
        date_debut: lundi.to_string(),
        date_fin: vendredi.to_string(),
        cours,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ICS: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Tests//FR\r\n\
BEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20261001T080000Z\r\nDTSTART:20261005T080000Z\r\nDTEND:20261005T100000Z\r\n\
SUMMARY:ALGEBRE (TD2)\r\nLOCATION:Salle A\r\nDESCRIPTION:CP2 PROMO DK\\nDUPONT Jean\\nTD2\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:2\r\nDTSTAMP:20261001T080000Z\r\nDTSTART;TZID=Europe/Paris:20261006T140000\r\nDTEND;TZID=Europe/Paris:20261006T160000\r\n\
SUMMARY:MECANIQUE DU SOLIDE - CINETIQUE (TP1)\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:3\r\nDTSTAMP:20261001T080000Z\r\nDTSTART;VALUE=DATE:20261007\r\nSUMMARY:Journée portes ouvertes\r\nEND:VEVENT\r\n\
END:VCALENDAR\r\n";

    #[test]
    fn urls_locales_et_privees_bloquees() {
        for u in [
            "http://127.0.0.1/calendar.ics",
            "https://localhost/calendar.ics",
            "https://127.0.0.1/x",
            "https://10.0.0.5/x",
            "https://192.168.1.10/x",
            "https://169.254.169.254/latest/meta-data",
            "https://[::1]/x",
            "https://[fd00::1]/x",
            "https://[::ffff:127.0.0.1]/x",
            "https://100.64.0.1/x",
            "https://intranet/x",
            "https://user:pwd@ade.example.com/x",
            "ftp://ade.example.com/x",
            "",
        ] {
            assert!(valider_syntaxe_url(u).is_err(), "{u} devrait être refusée");
        }
        assert!(valider_syntaxe_url("https://ade.example.com/jsp/custom/ics?data=abc").is_ok());
        assert!(valider_syntaxe_url("https://8.8.8.8/x").is_ok());
    }

    #[tokio::test]
    async fn resolution_refuse_les_adresses_privees() {
        assert!(valider_url("https://127.0.0.1/x").await.is_err());
        assert!(valider_url("https://localhost./x").await.is_err());
    }

    #[test]
    fn helpers_de_detection() {
        let titre = "MECANIQUE DU SOLIDE - CINETIQUE (TP2)";
        let description = "CP2 PROMO DK\nDUPONT Jean\nTD2";
        assert!(detecter_groupes(titre, description).contains(&"2".to_string()));
        assert!(detecter_types(titre, description).contains(&"TP".to_string()));
        assert_eq!(extraire_matiere(titre), "MECANIQUE DU SOLIDE");
        assert_eq!(
            extraire_enseignants(description),
            vec!["DUPONT Jean".to_string()]
        );
        assert!(detecter_types("Khôlle de maths", "").contains(&"KHÔLLE".to_string()));
    }

    #[test]
    fn analyse_ics_utc_tzid_et_jour_entier() {
        let cours = parser_cours(ICS).unwrap();
        assert_eq!(cours.len(), 3);
        // 08:00 UTC le 5 octobre 2026 → 10:00 à Paris (heure d'été).
        assert_eq!(
            (cours[0].debut.as_str(), cours[0].fin.as_str()),
            ("10:00", "12:00")
        );
        assert_eq!(cours[0].date, "2026-10-05");
        assert_eq!(cours[0].matiere, "ALGEBRE");
        assert_eq!(cours[0].salle, "Salle A");
        assert_eq!(cours[0].enseignant, "DUPONT Jean");
        assert_eq!(cours[1].debut, "14:00");
        assert!(cours[1].types.contains(&"TP".to_string()));
        assert_eq!(
            (cours[2].debut.as_str(), cours[2].date.as_str()),
            ("00:00", "2026-10-07")
        );
        assert!(cours[0].debut_iso.ends_with("+02:00"));
    }

    #[test]
    fn fuseau_horaire_hiver() {
        let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nDTSTART:20261201T080000Z\r\nDTEND:20261201T090000Z\r\nSUMMARY:X\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let c = parser_cours(ics).unwrap();
        assert_eq!(c[0].debut, "09:00");
    }

    #[test]
    fn contenu_invalide_refuse() {
        assert!(parser_cours("<html>pas un calendrier</html>").is_err());
        assert!(parser_cours("").unwrap().is_empty());
    }

    #[test]
    fn filtres_et_semaine() {
        let cours = parser_cours(ICS).unwrap();
        let f = |t: &[&str]| Filtres {
            types: t.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        };
        assert_eq!(appliquer_filtres(cours.clone(), &f(&["TD"])).len(), 2); // TD + jour entier sans type
        assert_eq!(appliquer_filtres(cours.clone(), &f(&["TP"])).len(), 2);
        assert_eq!(appliquer_filtres(cours.clone(), &f(&["xyz"])).len(), 1);
        let jour = NaiveDate::from_ymd_opt(2026, 10, 7);
        let sem = cours_semaine(cours.clone(), jour, &Filtres::default());
        assert_eq!(
            (sem.date_debut.as_str(), sem.date_fin.as_str()),
            ("2026-10-05", "2026-10-09")
        );
        assert_eq!(sem.cours.len(), 3);
        let j = cours_du_jour(
            cours.clone(),
            NaiveDate::from_ymd_opt(2026, 10, 6),
            &Filtres::default(),
        );
        assert_eq!(j.nb_cours, 1);
        let cat = analyser_categories(&cours);
        assert!(cat.matieres.contains(&"ALGEBRE".to_string()));
    }

    #[test]
    fn parser_jour_valide() {
        assert!(parser_jour(Some("2026-13-40")).is_err());
        assert_eq!(parser_jour(None).unwrap(), None);
        assert!(parser_jour(Some("2026-10-05")).unwrap().is_some());
    }
}
