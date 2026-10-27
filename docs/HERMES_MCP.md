# 🤖 Hermes Agent ↔ Tableau de bord étudiant (serveur MCP)

À partir de la **5.1.0**, l'application Windows installe un serveur **MCP** (*Model Context Protocol*) local, `tdb-mcp.exe`. Il permet à [Hermes Agent](https://github.com/NousResearch/hermes-agent) de consulter tes devoirs, et éventuellement de les modifier, sans que tu aies à les lui recopier :

> « Quelles sont mes tâches aujourd'hui ? » · « Quels projets sont en retard ? » · « Quelle tâche dois-je traiter en priorité ? » · « Résume-moi l'avancement de mes projets. »

Dans ce tableau de bord, un **projet** est une **matière**, et une **tâche** est un **devoir**. Les tâches sans matière forment le projet « Sans matière ».

## 1. Architecture

```
Hermes Agent ──(stdio : JSON-RPC, une ligne par message)──► tdb-mcp.exe
                                                               │ lecture/écriture SQLite (WAL), à chaque appel
TableauDeBordEtudiant.exe ─────────────────────────────────────┴──► %APPDATA%\fr.teamfliyou.tableaudebord\tableau-de-bord.db
```

- **Un exécutable séparé**, installé par l'installateur à côté de l'application, dans le même dossier. Hermes le lance lui-même quand il en a besoin, **même si la fenêtre du tableau de bord est fermée**. Une mise à jour de l'application le remplace en même temps.
- **Pas de second stockage** : le serveur ouvre la **même base SQLite** que l'application. Grâce au mode WAL et au délai d'attente sur verrou, les deux programmes peuvent l'utiliser en même temps. Chaque appel relit la base, donc Hermes voit tes modifications immédiatement. À l'inverse, l'application relit les devoirs quand sa fenêtre reprend le focus.
- **Le serveur ne crée, ne migre ni ne restaure jamais la base.** Si la base est absente, ou si son schéma ne correspond pas à sa version, il refuse avec un message clair. Seule l'application applique les migrations, avec une sauvegarde préalable.
- **Transport stdio uniquement** : aucun port réseau n'est ouvert et aucun accès distant n'existe.

## 2. Activer l'intégration dans l'application

**Configuration → Intégrations IA — Hermes Agent / MCP**

| Réglage | Par défaut | Effet |
|---|---|---|
| Activer l'intégration | **désactivé** | Désactivée : tout appel de Hermes est refusé (« L'intégration Hermes Agent est désactivée… »). |
| Autoriser les modifications | **désactivé** (lecture seule) | Active les 5 outils d'écriture, qui exigent toujours une confirmation (§ 4). Ce réglage se désactive automatiquement si l'intégration est désactivée. |

Les réglages sont enregistrés dans la base (table `parametres`). Le serveur les relit à chaque appel, donc l'effet est immédiat, sans redémarrage.

La même rubrique affiche :
- la disponibilité du serveur et son chemin ;
- la liste des outils ;
- la configuration à copier dans Hermes ;
- un bouton **Tester la connexion**, qui lance réellement `tdb-mcp.exe` comme Hermes le ferait, puis appelle `initialize`, `tools/list` et un outil, et affiche le résultat ou l'erreur.

## 3. Configurer Hermes Agent (Windows natif)

Hermes lit ses serveurs MCP dans la clé `mcp_servers` de son fichier `config.yaml`. Pour une installation native sous Windows, ce fichier est `%LOCALAPPDATA%\hermes\config.yaml`. La commande `hermes config edit` l'ouvre quel que soit le mode d'installation.

1. Dans l'application, clique sur **Copier la configuration**. Le bloc copié contient déjà tes vrais chemins :

   ```yaml
   mcp_servers:
     tableau_de_bord:
       command: 'C:\Users\<toi>\AppData\Local\Tableau de bord étudiant\tdb-mcp.exe'
       args: ['--base', 'C:\Users\<toi>\AppData\Roaming\fr.teamfliyou.tableaudebord\tableau-de-bord.db']
       timeout: 30
       connect_timeout: 15
       idle_timeout_seconds: 600
   ```

   - Les chemins sont entre **apostrophes**, ce qui évite d'échapper les `\` en YAML.
   - `--base` est passé explicitement : Hermes ne transmet au serveur qu'un environnement restreint, le serveur ne doit donc pas dépendre de `%APPDATA%`.
   - `idle_timeout_seconds: 600` laisse Hermes arrêter le serveur après 10 minutes sans appel. Il le relance de façon transparente à l'appel suivant. Cela évite aussi qu'un serveur actif reste longtemps ouvert, ce qui bloquerait son remplacement lors d'une mise à jour.
2. Colle ce bloc dans `config.yaml` (`hermes config edit`). Si une section `mcp_servers:` existe déjà, ajoute seulement la partie `tableau_de_bord:` dessous, avec la même indentation.
3. Vérifie la connexion :
   ```powershell
   hermes mcp test tableau_de_bord
   ```
4. Ouvre une nouvelle session (`hermes chat`), ou tape `/reload-mcp` dans une session déjà ouverte. Hermes préfixe les outils : `mcp_tableau_de_bord_get_today_tasks`, etc.

Si Hermes tourne **dans WSL2**, le même exécutable fonctionne. Remplace alors les chemins par leur équivalent `/mnt/c/Users/<toi>/AppData/...` dans `command` et `args`.

Pour exposer moins d'outils à Hermes, filtre-les côté Hermes avec `tools: { include: [...] }` dans l'entrée `tableau_de_bord`. C'est indépendant des autorisations de l'application, qui restent la protection principale.

## 4. Outils

| Outil | Type | Arguments |
|---|---|---|
| `list_projects` | lecture | — |
| `get_project_details` | lecture | `projet` |
| `list_tasks` | lecture | `projet`, `statut` (`a_faire`/`en_cours`/`termine`), `priorite_min` (0-3), `recherche`, `inclure_terminees`, `limite` (1-200) |
| `get_task_details` | lecture | `id` |
| `get_today_tasks` | lecture | — (échéance = aujourd'hui, heure de Paris) |
| `get_upcoming_tasks` | lecture | `jours` (1-90, 7 par défaut, à partir de demain) |
| `get_overdue_tasks` | lecture | — |
| `get_completed_tasks` | lecture | `projet`, `limite` |
| `get_project_progress` | lecture | `projet` (facultatif : sans argument, avancement global et par projet) |
| `create_project` | écriture | `nom`, `titre` (première tâche), `echeance`, `type`, `priorite`, `description` |
| `update_project` | écriture | `projet`, `nouveau_nom` (renomme la matière sur toutes ses tâches) |
| `create_task` | écriture | `titre`, `projet`, `echeance`, `type`, `statut`, `priorite`, `description`, `sous_taches` |
| `update_task` | écriture | `id` + champs à modifier |
| `complete_task` | écriture | `id` |

Les listes de tâches sont triées par **urgence** : d'abord les non terminées, puis par échéance la plus proche (sans échéance en dernier), puis par priorité haute. Chaque tâche indique `jours_restants`, `en_retard`, la priorité et l'avancement de ses sous-tâches. Hermes a donc de quoi répondre à « que dois-je traiter en priorité ? ».

Il n'existe **aucun outil de suppression**, ni aucun accès aux fichiers, au réseau ou au système.

### Confirmation des modifications

Un outil d'écriture fonctionne toujours en **deux temps** :

1. **Premier appel**, sans `confirmation` : **rien n'est modifié**. Le serveur valide entièrement la demande, puis renvoie un aperçu (« Créer la tâche « Réviser » dans le projet « MATHS » pour le 2026-11-02 ») et un **jeton**.
2. **Second appel**, avec les **mêmes arguments** et `"confirmation": "<jeton>"`, après ton accord : la modification est appliquée.

Le jeton est à usage unique, expire au bout de 5 minutes et est lié exactement à cette action : avec d'autres arguments, il est refusé. Si tu retires l'autorisation d'écriture entre les deux appels, la modification est refusée. Les instructions transmises à Hermes lui demandent de te présenter l'aperçu avant de confirmer.

## 5. Sécurité

- Exécution **locale** uniquement (stdio), sans port ouvert, sans accès distant, sans mot de passe ni jeton dans le code.
- **Lecture seule par défaut.** L'écriture n'est possible qu'après autorisation explicite dans l'application, puis confirmation à chaque opération.
- **Validation stricte des paramètres** : arguments inconnus refusés, types et bornes contrôlés, et mêmes règles de validation que l'application (titre, matière, format de date `AAAA-MM-JJ`, type, statut, priorité, sous-tâches).
- **Intégrité des données** : connexion en lecture seule pour la consultation ; transactions SQLite pour les écritures (renommage d'une matière en une seule transaction) ; délai d'attente de 5 s sur verrou en cas d'accès concurrent avec l'application ; aucune migration par le serveur. Les sauvegardes quotidiennes de l'application (7 conservées) restent actives.
- **Journal** : `%APPDATA%\fr.teamfliyou.tableaudebord\logs\mcp.log` (connexions, outils appelés, erreurs ; rotation à 1 Mo). Rien n'est écrit sur la sortie standard, réservée au protocole.

## 6. Mises à jour et serveur MCP

Le serveur MCP fait partie de l'installateur : une mise à jour de l'application le remplace au même endroit, donc la configuration Hermes reste valable. Juste avant d'installer une mise à jour, l'application arrête les éventuels `tdb-mcp.exe` en cours, sinon Windows verrouillerait le fichier. Hermes relance le serveur à sa prochaine utilisation. Si Hermes signale une erreur juste après une mise à jour, tape `/reload-mcp`.

Pour une **réinstallation manuelle** (installateur lancé à la main), ferme d'abord Hermes, ou attends 10 minutes sans l'utiliser.

La **version portable** (`TableauDeBordEtudiant.exe` seul) ne contient pas le serveur. `tdb-mcp.exe` est publié séparément dans chaque release : place-le où tu veux et indique son chemin dans `command`.

## 7. Dépannage

| Message | Cause / solution |
|---|---|
| « L'intégration Hermes Agent est désactivée » | Active-la dans Configuration → Intégrations IA. |
| « … ne sont pas autorisées … lecture seule » | Active « Autoriser les modifications » si tu veux que Hermes modifie des tâches. |
| « Base de données introuvable » | Lance l'application une première fois, ou corrige le chemin passé à `--base`. |
| « La base n'est pas encore à jour » | Ouvre l'application une fois : elle termine sa migration. |
| « La base provient d'une version plus récente » | Le serveur est plus ancien que la base : mets à jour l'application. |
| `hermes mcp test` échoue au lancement | Vérifie le chemin dans `command` (apostrophes, `tdb-mcp.exe` présent). |

Le serveur peut aussi se tester à la main :

```powershell
& "$env:LOCALAPPDATA\Tableau de bord étudiant\tdb-mcp.exe" --version   # chemin d'installation par défaut
```

Le chemin exact de ton installation est affiché dans Configuration → Intégrations IA.
