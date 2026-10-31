# VOOBY PARIS — Black November 2026 (Remotion)

Publicité verticale 1080×1920, 30 fps, 750 frames (25 s), MP4 H.264, sans voix off.
Rendu final : `out/vooby-black-november.mp4`.

## Commandes
```bash
npm install
npm run studio     # prévisualisation interactive (timeline, scrubbing)
npm run render     # export MP4 -> out/vooby-black-november.mp4
```
Environnement sans Chrome téléchargeable : ajouter `--browser-executable=<chemin chrome-headless-shell>`.
Conversion conseillée pour la compatibilité (plage TV) :
`ffmpeg -i out/vooby-black-november.mp4 -vf "scale=in_range=pc:out_range=tv,format=yuv420p" -c:v libx264 -crf 16 -movflags +faststart -c:a copy final.mp4`

## Structure
- `public/photos/` photos fournies · `public/logo/` logo officiel · `public/music/` musique + effets
- `src/constants.ts` timeline (scènes calées sur le tempo ≈116 BPM), couleurs, zones de sécurité
- `src/scenes/Scene1..6.tsx` les 6 scènes · `src/components/` Photo (Ken Burns), Text, Scene
- Modifier prix / textes : `src/scenes/Scene6.tsx`

## Crédits & licences
**Musique** : « Bass Vibes » — Kevin MacLeod (incompetech.com)
Source : https://incompetech.com/music/royalty-free/mp3-royaltyfree/Bass%20Vibes.mp3
Licence : Creative Commons Attribution (CC BY, usage commercial autorisé, **attribution obligatoire**).
À coller dans la description de la publication :
```
Music: "Bass Vibes" by Kevin MacLeod (incompetech.com)
Licensed under Creative Commons: By Attribution 4.0 License
http://creativecommons.org/licenses/by/4.0/
```
⚠ Vérifiez la version de licence (3.0 ou 4.0) sur la page du morceau avant diffusion payante (Facebook Ads).
**Effets** (`riser.wav`, `impact.wav`) : synthétisés avec ffmpeg, libres de droits.
**Logo** : https://voobyparis.com/img/logo-1792619714.jpg (en réalité un PNG transparent noir ; version blanche = même logo recoloré).
**Polices** : Cormorant Garamond, Montserrat (SIL OFL, via @fontsource).
