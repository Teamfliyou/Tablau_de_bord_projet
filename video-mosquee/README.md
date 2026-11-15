# Vidéo TV – Association Musulmane Audomaroise

Vidéo Remotion (1920×1080, 30 i/s, 49 s) qui enchaîne 4 écrans animés :

1. **Avancement du projet** (14 s) : le titre se dévoile, la frise se dessine étape par étape
   (coches qui se tracent, ligne verte qui progresse), le compteur monte jusqu'à 100 000 €,
   l'étape « Aujourd'hui » pulse en continu.
2. **Collecte du gros œuvre** (12 s) : le montant monte jusqu'à 137 683 €, la barre se remplit
   jusqu'à 46 % des 300 000 €, puis « Il reste 162 317 € à réunir, chaque don compte. »
3. **Permis de construire accordé** (13 s) : le titre monte ligne par ligne, « accordé » est
   surligné, l'arrêté arrive en glissant, l'article 1 s'écrit à la machine, puis un tampon ✓
   vient le valider ; compteur + 378 m².
4. **Téléphone en silencieux** (10 s) : le téléphone sonne et vibre, puis est barré en rouge.

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
- `out/01_mosquee-tv.mp4` : une boucle de 49 s ;
- `out/01_mosquee-tv_x20.mp4` : la même répétée 20 fois (≈ 16 min), pour les télés qui ne répètent pas les vidéos.

## Modifier le contenu

- **Montants de la collecte : `src/donnees.ts`** (la frise et l'écran Collecte se mettent à jour)
- Textes : `src/scenes/Avancement.tsx`, `src/scenes/Permis.tsx`, `src/scenes/Telephone.tsx`
- Durée de chaque scène : constantes `*_DURATION` en haut de chaque fichier (en frames, 30 = 1 s)
- Couleurs : `src/theme.ts`
- Polices et logo : `public/` (embarqués, aucun accès internet nécessaire au rendu)

## Sur la télé

Copier `out/01_mosquee-tv.mp4` sur une clé USB (FAT32) et activer la lecture en boucle
(« Répéter ») dans le lecteur de la TV. Si la télé ne sait pas répéter, utiliser `01_mosquee-tv_x20.mp4`.
