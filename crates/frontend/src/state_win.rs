//! Actions propres à l'interface Windows : annuler / rétablir, ajout rapide, sélection, recherche.

use crate::api;
use crate::state::Etat;
use leptos::prelude::*;
use leptos::task::spawn_local;
use tdb_shared::{Devoir, DevoirEntree, DevoirModification, TagEntree, TodoEntree};

/// Action réversible (pile annuler / rétablir). `appliquer` exécute l'action et renvoie son inverse.
#[derive(Clone)]
pub enum Annul {
    /// Remet le devoir dans ce statut.
    Statut { id: i64, vers: String },
    /// Recrée un devoir supprimé.
    Recreer(DevoirEntree),
    /// Supprime un devoir créé.
    Supprimer(i64),
}

const PILE_MAX: usize = 50;

pub fn en_entree(d: &Devoir) -> DevoirEntree {
    DevoirEntree {
        titre: d.titre.clone(),
        matiere: d.matiere.clone(),
        echeance: d.echeance.clone(),
        type_: d.type_.clone(),
        statut: d.statut.clone(),
        description: d.description.clone(),
        priorite: d.priorite,
        tags: d
            .tags
            .iter()
            .map(|t| TagEntree {
                nom: t.nom.clone(),
                couleur: t.couleur.clone(),
            })
            .collect(),
        todos: d
            .todos
            .iter()
            .map(|t| TodoEntree {
                texte: t.texte.clone(),
                fait: t.fait,
            })
            .collect(),
    }
}

fn empiler(pile: RwSignal<Vec<Annul>>, a: Annul) {
    pile.update(|p| {
        p.push(a);
        if p.len() > PILE_MAX {
            p.remove(0);
        }
    });
}

impl Etat {
    pub fn devoir_par_id(self, id: i64) -> Option<Devoir> {
        self.devoirs
            .with(|l| l.iter().find(|d| d.id == id).cloned())
    }

    /// Devoirs d'un statut, filtrés par la recherche et le tag actif.
    pub fn devoirs_visibles(self, statut: &str) -> Vec<Devoir> {
        let filtre = self.filtre_tag.get();
        let q = self.recherche.get().trim().to_lowercase();
        self.devoirs.with(|l| {
            l.iter()
                .filter(|d| d.statut == statut)
                .filter(|d| filtre.is_none_or(|t| d.tags.iter().any(|g| g.id == t)))
                .filter(|d| {
                    q.is_empty()
                        || d.titre.to_lowercase().contains(&q)
                        || d.matiere.to_lowercase().contains(&q)
                        || d.tags.iter().any(|t| t.nom.to_lowercase().contains(&q))
                })
                .cloned()
                .collect()
        })
    }

    /// Change le statut en mémorisant l'ancien pour pouvoir annuler.
    pub fn deplacer_vers(self, id: i64, statut: &'static str) {
        let Some(d) = self.devoir_par_id(id) else {
            return;
        };
        if d.statut == statut {
            return;
        }
        empiler(self.annulables, Annul::Statut { id, vers: d.statut });
        self.refaisables.set(vec![]);
        self.changer_statut(id, statut);
    }

    pub fn supprimer_annulable(self, id: i64) {
        let Some(d) = self.devoir_par_id(id) else {
            return;
        };
        empiler(self.annulables, Annul::Recreer(en_entree(&d)));
        self.refaisables.set(vec![]);
        if self.edition.get_untracked().is_some_and(|e| e.id == id) {
            self.edition.set(None);
        }
        self.supprimer_devoir(id);
        self.notifier(format!("« {} » supprimé. ⌘Z pour annuler.", d.titre));
    }

    /// Création rapide (titre seul) dans une colonne.
    pub fn ajout_rapide(self, statut: &str, titre: &str) {
        let titre = titre.trim().to_string();
        if titre.is_empty() {
            return;
        }
        let entree = DevoirEntree {
            titre,
            matiere: String::new(),
            echeance: String::new(),
            type_: "devoir".into(),
            statut: statut.to_string(),
            tags: vec![],
            todos: vec![],
            description: String::new(),
            priorite: 0,
        };
        spawn_local(async move {
            match api::creer_devoir(&entree).await {
                Ok(d) => {
                    empiler(self.annulables, Annul::Supprimer(d.id));
                    self.refaisables.set(vec![]);
                    self.charger_devoirs_async().await;
                }
                Err(e) => {
                    let m = self.message(e);
                    self.erreur_devoirs.set(Some(m));
                }
            }
        });
    }

    fn appliquer_annul(self, a: Annul, vers_pile: RwSignal<Vec<Annul>>) {
        match a {
            Annul::Statut { id, vers } => {
                let Some(d) = self.devoir_par_id(id) else {
                    return;
                };
                empiler(vers_pile, Annul::Statut { id, vers: d.statut });
                let m = DevoirModification {
                    statut: Some(vers.clone()),
                    ..Default::default()
                };
                spawn_local(async move {
                    if api::modifier_devoir(id, &m).await.is_ok() {
                        self.charger_devoirs_async().await;
                    }
                });
            }
            Annul::Recreer(entree) => {
                spawn_local(async move {
                    if let Ok(d) = api::creer_devoir(&entree).await {
                        empiler(vers_pile, Annul::Supprimer(d.id));
                        self.charger_devoirs_async().await;
                    }
                });
            }
            Annul::Supprimer(id) => {
                let Some(d) = self.devoir_par_id(id) else {
                    return;
                };
                empiler(vers_pile, Annul::Recreer(en_entree(&d)));
                if self.edition.get_untracked().is_some_and(|e| e.id == id) {
                    self.edition.set(None);
                }
                spawn_local(async move {
                    if api::supprimer_devoir(id).await.is_ok() {
                        self.charger_devoirs_async().await;
                    }
                });
            }
        }
    }

    pub fn annuler(self) {
        let mut derniere = None;
        self.annulables.update(|p| derniere = p.pop());
        if let Some(a) = derniere {
            self.appliquer_annul(a, self.refaisables);
        }
    }

    pub fn retablir(self) {
        let mut derniere = None;
        self.refaisables.update(|p| derniere = p.pop());
        if let Some(a) = derniere {
            self.appliquer_annul(a, self.annulables);
        }
    }
}
