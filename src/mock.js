// Simulation du backend Rust pour l'aperçu navigateur (`npm run dev`). Chargé uniquement en développement.
const iso = (d) => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
const aujourdhui = new Date();
const lundi = new Date(aujourdhui);
lundi.setDate(lundi.getDate() - ((lundi.getDay() + 6) % 7));

const jourDeSemaine = (decalage, debut, fin, extra) => {
  const d = new Date(lundi);
  d.setDate(d.getDate() + decalage);
  return { salle: 'B204', enseignant: 'DUPONT Jean', enseignants: ['DUPONT Jean'], groupes: [], types: [], date: iso(d), debut, fin, ...extra };
};

let cours = [
  jourDeSemaine(0, '08:00', '10:00', { titre: 'ALGEBRE (TD2)', matiere: 'ALGEBRE', types: ['TD'], groupes: ['2'] }),
  jourDeSemaine(1, '10:15', '12:15', { titre: 'MECANIQUE DU SOLIDE (CM)', matiere: 'MECANIQUE DU SOLIDE', types: ['CM'] }),
  jourDeSemaine(2, '14:00', '16:00', { titre: 'PHYSIQUE (TP1)', matiere: 'PHYSIQUE', types: ['TP'], groupes: ['1'] }),
  jourDeSemaine(3, '08:00', '10:00', { titre: 'ANGLAIS', matiere: 'ANGLAIS' }),
  jourDeSemaine(4, '13:30', '15:30', { titre: 'DS MATHS', matiere: 'MATHS', types: ['DS'] }),
  { ...jourDeSemaine(0, '00:00', '23:59', { titre: 'ANALYSE (CM)', matiere: 'ANALYSE', types: ['CM'] }), date: iso(aujourdhui), debut: '08:00', fin: '23:00' },
  { ...jourDeSemaine(0, '00:00', '23:59', { titre: 'INFORMATIQUE (TP2)', matiere: 'INFORMATIQUE', types: ['TP'], groupes: ['2'] }), date: iso(aujourdhui), debut: '14:00', fin: '17:00' },
];
cours.sort((a, b) => (a.date + a.debut).localeCompare(b.date + b.debut));

const dans = (n) => { const d = new Date(aujourdhui); d.setDate(d.getDate() + n); return iso(d); };
let devoirs = [
  { id: 1, titre: 'Rapport de TP physique', matiere: 'PHYSIQUE', echeance: dans(2), type: 'devoir', statut: 'en_cours' },
  { id: 2, titre: 'DS de mathématiques', matiere: 'MATHS', echeance: dans(5), type: 'ds', statut: 'a_faire' },
  { id: 3, titre: 'Exercices algèbre', matiere: 'ALGEBRE', echeance: dans(-1), type: 'ie', statut: 'a_faire' },
  { id: 4, titre: 'Exposé anglais', matiere: 'ANGLAIS', echeance: dans(12), type: 'devoir', statut: 'termine' },
];
let prochainId = 5;
let config = { username: 'Étudiant', ade_url: 'https://ade.example.com/ics?data=demo', gemini_model: 'gemini-3.8-flash', gemini_configured: false };
let preferences = {};

const filtrer = (liste, f = {}) => liste;

export default async function moquette(commande, args) {
  switch (commande) {
    case 'obtenir_config': return { ...config };
    case 'enregistrer_config': {
      const m = args.maj;
      config = { ...config, username: m.username, ade_url: m.ade_url, gemini_model: m.gemini_model || config.gemini_model };
      if (m.remove_gemini_key) config.gemini_configured = false;
      else if (m.gemini_key) config.gemini_configured = true;
      return { ...config };
    }
    case 'obtenir_preferences': return preferences;
    case 'enregistrer_preferences': preferences = args.preferences; return null;
    case 'cours_du_jour': {
      const liste = filtrer(cours.filter((c) => c.date === iso(aujourdhui)), args.filtres);
      return { date: iso(aujourdhui), nb_cours: liste.length, cours: liste, hors_ligne: false, maj: new Date().toISOString() };
    }
    case 'cours_semaine': {
      const ref = args.date ? new Date(args.date + 'T00:00:00') : aujourdhui;
      const l = new Date(ref);
      l.setDate(l.getDate() - ((l.getDay() + 6) % 7));
      const v = new Date(l);
      v.setDate(v.getDate() + 4);
      const liste = cours.filter((c) => c.date >= iso(l) && c.date <= iso(v));
      return { date_debut: iso(l), date_fin: iso(v), cours: liste, hors_ligne: true, maj: new Date().toISOString() };
    }
    case 'analyser_categories': return { groupes: ['1', '2'], types: ['CM', 'DS', 'TD', 'TP'], matieres: [...new Set(cours.map((c) => c.matiere))].sort() };
    case 'lister_devoirs': return devoirs.map((d) => ({ ...d }));
    case 'creer_devoir': {
      const d = { id: prochainId++, matiere: '', echeance: '', type: 'devoir', statut: 'a_faire', ...args.devoir };
      devoirs.push(d);
      return d;
    }
    case 'modifier_devoir': {
      const d = devoirs.find((x) => x.id === args.id);
      if (!d) throw new Error('Devoir introuvable.');
      Object.assign(d, args.modification);
      return { ...d };
    }
    case 'supprimer_devoir': devoirs = devoirs.filter((d) => d.id !== args.id); return null;
    case 'exporter_devoirs': return 'C:\\Utilisateurs\\demo\\devoirs.json';
    case 'importer_devoirs': return 3;
    case 'migrer_ancienne_base': return { devoirs_importes: 4, devoirs_ignores: 1, parametres_importes: true, cle_gemini_trouvee: false };
    case 'generer_plan':
      return {
        conseil: 'Commence par le DS de mathématiques, puis alterne révisions et pauses courtes.',
        planning: [
          { heure: '18:00 - 18:45', matiere: 'DS de mathématiques', action: 'Refaire les exercices du dernier TD' },
          { heure: '18:55 - 19:40', matiere: 'Rapport de TP physique', action: 'Rédiger la partie résultats' },
        ],
        mode: 'simulation',
        avertissement: config.gemini_configured ? 'Quota Gemini dépassé : réessaie dans quelques minutes. Plan généré en mode simulation.' : null,
      };
    case 'version_application': return '4.0.0';
    default: throw new Error('Commande inconnue : ' + commande);
  }
}
