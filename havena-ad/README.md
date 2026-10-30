# havena-ad — publicité Havena (cuisines) 16:9

Publicité de 30 s, 1920×1080, 30 fps, 100 % code (Remotion + React + TypeScript). Les cuisines sont dessinées en SVG
(`src/components/KitchenSVG.tsx`) en attendant les vraies photos.

```bash
npm install
npm run studio                       # aperçu
npm run render                       # → output/havena-publicite-16-9.mp4 (1920×1080)
npm run render -- --vertical         # → output/havena-publicite-9-16.mp4 (1080×1920, TikTok/Reels/Shorts)
npm run still -- 45 300 620          # images de contrôle → output/stills/ (ajouter --vertical pour le 9:16)
```
Chromium personnalisé : `BROWSER_EXECUTABLE=/chemin/chrome npm run render`.

## Remplacer par les vraies données (tout est dans `src/data/brand.ts`)
- **Photos de cuisines** : déposer les fichiers dans `public/assets/havena/` puis renseigner `image: "assets/havena/cuisine-1.jpg"`
  sur chaque produit (ratio 3:2 recommandé, recadrage automatique sans déformation).
- **Logo** : `logo: "assets/havena/logo.png"` (le logo actuel est un placeholder dessiné en code).
- **Prix / noms / arguments / couleurs** : `BRAND`, `PRODUCTS`, `ARGUMENTS` et `src/styles/theme.ts`.
  Le prix (1 500 €) vient du client ; les noms de finitions et les 3 arguments décrivent seulement ce que montre l'illustration : à valider.
- **Musique** : déposer un fichier dans `public/assets/audio/` et renseigner `music` (aucune piste pour l'instant).

## Structure
`src/scenes/` (5 scènes) · `src/components/` (KitchenSVG, Logo, RevealText, PanelWipe, Backdrop/FilmFinish, ProductVisual) ·
`src/animations/` (easing, minutage) · `src/data/` · `src/styles/` · `src/compositions/HavenaAd.tsx` (montage).

La version verticale (`src/scenes/vertical/`, `src/compositions/HavenaAdVertical.tsx`) réutilise les mêmes données, composants et le même minutage ; seule la mise en page change.
