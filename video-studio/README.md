# video-studio — vidéos faceless 100 % code

Vidéos verticales **1080×1920 @ 30 fps** (TikTok / Shorts / Reels) générées à partir d'un simple script texte.
Aucune image ni vidéo générée par IA : tout est du **React + SVG + CSS** animé avec [Remotion](https://remotion.dev).

## Démarrage

```bash
cd video-studio
npm install
npm run studio          # aperçu interactif (Remotion Studio)
npm run plan -- content/demo.txt     # voir le découpage en scènes
npm run render:demo     # → out/demo.mp4 (30 s)
npm run render -- content/ma-video.txt out/ma-video.mp4
npm run render -- content/demo.txt out/demo.mp4 --still=120   # une image PNG
npm test
```

Remotion télécharge un Chrome headless au premier rendu. Hors-ligne / derrière un proxy, pointez vers un Chromium existant :
`BROWSER_EXECUTABLE=/chemin/vers/chrome npm run render:demo`.

## Format du script

```text
@title Mon titre          # en-têtes facultatifs
@theme neon               # neon | sunset | ocean
@duration 30              # durée cible en secondes (sinon : estimée au texte)
@voice audio/voix.mp3     # voix off facultative (fichier dans public/)

[hook icon=rocket title="Titre à l'écran"]
Narration de la scène (sous-titres + durée).

[stat value=87 suffix=% title="Légende"]
[bars title="…" data="Lun:12,Mar:28,Mer:45"]
[line title="…" data="Jan:10,Mar:24,Mai:38"]
[list title="…"]
- Point 1
- Point 2
[question] / [outro cta="Abonne-toi"]
```

- Les scènes sont séparées par une **ligne vide**. Sans balise, le type est **deviné** (1re = hook, dernière = outro,
  `?` = question, nombre + `%` = stat, puces = list) ; une icône est choisie par mots-clés.
  Un texte sans saut de ligne est découpé automatiquement par groupes de 2 phrases.
- `bars`/`line` exigent `data=` : sans données, la scène retombe sur un autre type (rien n'est inventé).
- **Voix off** : si `@voice` existe, sa durée fixe la durée de la vidéo ; les scènes sont réparties proportionnellement
  au nombre de mots, et les sous-titres sont minutés mot à mot sur cette base (approximation, pas d'alignement phonétique).
  Sans voix, la durée vient du texte (~2,6 mots/s) ou de `@duration`.

## Architecture (réutilisable)

| Dossier | Rôle |
|---|---|
| `src/lib/scriptParser.ts` | texte → scènes typées (détection, directives) |
| `src/lib/planner.ts` | durées, chevauchement des transitions, minutage des sous-titres |
| `src/lib/theme.ts`, `anim.ts` | thèmes, easings, springs, aléatoire déterministe |
| `src/components/` | briques : `Background`, `KineticText`, `Icon`/`IconBadge` (16 icônes SVG tracées), `Charts` (`Counter`, `BarChart`, `LineChart`), `CameraRig`, `Subtitles` |
| `src/scenes/scenes.tsx` | une scène = composition de briques ; `SCENE_COMPONENTS` fait le lien type → composant |
| `src/Video.tsx` | montage : `TransitionSeries` (slide/wipe/flip/fade), caméra, sous-titres, audio |
| `src/Root.tsx` | composition Remotion + `calculateMetadata` (durée dynamique) |
| `scripts/render.ts` | CLI de rendu MP4 |

**Ajouter un type de scène** : créer le composant dans `scenes.tsx`, l'ajouter à `SCENE_COMPONENTS` et à `SceneType`
(`types.ts`) ainsi qu'à `SCENE_TYPES` (`scriptParser.ts`). **Ajouter une icône** : un tableau de chemins SVG 24×24 dans `Icon.tsx`
+ le nom dans `IconName`. **Ajouter un thème** : une entrée dans `THEMES`.
