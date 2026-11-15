# Vidéo TV – Association Musulmane Audomaroise

Vidéo Remotion (1920×1080, 30 i/s, ~37 s) qui reprend les 3 visuels avec des animations :

1. **Avancement du projet** (14 s) : le titre se dévoile, la frise se dessine étape par étape
   (coches qui se tracent, ligne verte qui progresse), le compteur monte jusqu'à 100 000 €,
   l'étape « Aujourd'hui » pulse en continu.
2. **Permis de construire accordé** (13 s) : le titre monte ligne par ligne, « accordé » est
   surligné, l'arrêté arrive en glissant, l'article 1 s'écrit à la machine, puis un tampon ✓
   vient le valider ; compteur + 378 m².
3. **Téléphone en silencieux** (10 s) : le téléphone sonne et vibre, puis est barré en rouge.

Le bandeau du bas reste fixe, et la vidéo commence et se termine sur un fond vide :
elle peut **tourner en boucle** sur la télé sans coupure visible.

Charte graphique, règles d'affichage TV et données du permis : voir [CONTEXTE.md](CONTEXTE.md).

## Utilisation

```bash
npm install
npm run studio   # prévisualiser et modifier dans le navigateur
npm run build    # génère les vidéos pour la clé USB (voir ci-dessous)
```

`npm run build` produit, au format TV (H.264 high 4.0, yuv420p, 30 i/s, sans son) :
- `out/01_mosquee-tv.mp4` : une boucle de 37 s ;
- `out/01_mosquee-tv_x20.mp4` : la même répétée 20 fois (≈ 12 min), pour les télés qui ne répètent pas les vidéos.

## Modifier le contenu

- Textes, montants : `src/scenes/Avancement.tsx`, `src/scenes/Permis.tsx`, `src/scenes/Telephone.tsx`
- Durée de chaque scène : constantes `*_DURATION` en haut de chaque fichier (en frames, 30 = 1 s)
- Couleurs : `src/theme.ts`
- Polices et logo : `public/` (embarqués, aucun accès internet nécessaire au rendu)

## Sur la télé

Copier `out/01_mosquee-tv.mp4` sur une clé USB (FAT32) et activer la lecture en boucle
(« Répéter ») dans le lecteur de la TV. Si la télé ne sait pas répéter, utiliser `01_mosquee-tv_x20.mp4`.
