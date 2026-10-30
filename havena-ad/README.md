# havena-ad — publicité Havena (cuisines) 16:9

Publicité de 30 s, 30 fps, 100 % code (Remotion + React + TypeScript), en 16:9 et en 9:16. Photos, logo et prix viennent de
https://www.havena.fr/ (relevé d'octobre 2026). `KitchenSVG.tsx` n'est plus qu'une illustration de secours si une photo manque.

```bash
npm install
npm run studio                       # aperçu
npm run render                       # → output/havena-publicite-16-9.mp4 (1920×1080)
npm run render -- --vertical         # → output/havena-publicite-9-16.mp4 (1080×1920, TikTok/Reels/Shorts)
npm run still -- 45 300 620          # images de contrôle → output/stills/ (ajouter --vertical pour le 9:16)
```
Chromium personnalisé : `BROWSER_EXECUTABLE=/chemin/chrome npm run render`.

## Données (tout est dans `src/data/brand.ts`)
- **Photos** : `public/assets/havena/` ; à remplacer par des fichiers plus nets si besoin (la photo Scarlet ne fait que 752 px de large).
- **Prix** : 799 € TTC pour les deux cuisines Atelier (fiches du site). Aucun prix pour Scarlet : sa fiche contient des données de test.
- **Arguments** : tirés de la page d'accueil (5 collections, devis gratuit sous 72 h, conseiller).
- **Logo** : `public/assets/havena/logo.png` (officiel, recadré dans `Logo.tsx`).
- **Musique** : déposer un fichier dans `public/assets/audio/` et renseigner `music` (aucune piste pour l'instant).

## Structure
`src/scenes/` (5 scènes) · `src/components/` (KitchenSVG, Logo, RevealText, PanelWipe, Backdrop/FilmFinish, ProductVisual) ·
`src/animations/` (easing, minutage) · `src/data/` · `src/styles/` · `src/compositions/HavenaAd.tsx` (montage).

La version verticale (`src/scenes/vertical/`, `src/compositions/HavenaAdVertical.tsx`) réutilise les mêmes données, composants et le même minutage ; seule la mise en page change.
