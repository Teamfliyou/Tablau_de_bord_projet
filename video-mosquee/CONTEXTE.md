> Copie du README du projet « Affichage numérique » (conversation Claude classique, novembre 2026).
> Les sources HTML/JPG mentionnées ci-dessous ne sont pas dans ce dépôt ; la vidéo animée Remotion de ce dossier suit cette charte.

# Affichage numérique — Association Musulmane Audomaroise

Écrans pour la télé de la mosquée, lus depuis une clé USB, sous forme d'images JPG ou d'une vidéo MP4 en boucle.
Ce dossier contient tout ce qui a été fait dans la conversation Claude « Affichage numérique à la mosquée » (novembre 2026).

## Contenu

```
src/            sources HTML des écrans (1 fichier = 1 écran 1920x1080)
src/base.css    gabarit commun : couleurs, polices, bandeau de pied de page
src/assets/     logo (complet, icône seule, original), extraits du permis de construire
src/fonts/      Lora, Poppins, DejaVu Sans (fichiers locaux, aucun accès réseau nécessaire)
build.sh        génère out/*.jpg et les vidéos MP4
out/            rendus actuels (JPG + vidéo boucle 33 s)
```

Prérequis : Chrome ou Chromium (headless), ffmpeg, Python 3 avec Pillow.
`./build.sh` régénère tout. `HOLD=12 FADE=1 ./build.sh` change la durée par écran et celle du fondu.
L'ordre des écrans dans la vidéo est défini par le tableau `VIDEO=(...)` dans build.sh.

## Charte graphique

| Rôle | Couleur |
|---|---|
| Vert profond (titres, texte fort) | #0F3A2E |
| Doré (accents, filets, sur-titres) | #D89020 |
| Blanc cassé (fond par défaut) | #FAF8F5 |
| Blanc pur (cartes, bandeau) | #FFFFFF |
| Gris anthracite (texte courant) | #4A5568 |
| Gris clair (bordures) | #EAEAEA |

- **Fond clair obligatoire.** Pas de grands fonds vert foncé : le vert et le doré servent uniquement d'accents.
- **Polices :** Lora (titres, en gras 700), Poppins (texte courant, sur-titres en majuscules espacées), DejaVu Sans (texte arabe).
- **Logo :** on garde les couleurs d'origine du logo (vert-bleu #0C5953, doré #DEAD46). On ne le recolore pas.
- **Bandeau de pied de page commun :** fond blanc, filet doré en haut, icône du logo et « Association Musulmane Audomaroise » centrés. Pas d'adresse e-mail.

## Règles d'affichage TV

- Format 1920×1080 en paysage, exporté en JPG (qualité 92). Pas de PDF ni de PPTX, que les télés ne lisent pas.
- Une annonce par écran, 30 à 40 mots au maximum. Titres d'environ 90 à 120 px, texte courant d'au moins 40 px.
- Garder une marge d'environ 5 % sur les bords, parce que certaines télés rognent l'image (voir la mire 01).
- Nommer les fichiers 01_, 02_… : la télé les affiche par ordre alphabétique. Clé USB en FAT32.
- Vidéo : H.264 yuv420p, profil high, niveau 4.0, 30 fps, sans son. 10 s par écran avec un fondu de 1 s. Le dernier fondu revient sur le premier écran pour que la boucle ne saute pas. Une version répétée 20 fois sert pour les télés qui ne répètent pas les vidéos.

## Écrans existants

| Fichier | Contenu |
|---|---|
| 01_mire | Mire de test : cadre rouge au bord, zone de sécurité dorée, tailles de texte 36/48/64/88 px |
| 02_bienvenue | Logo complet à gauche ; « السلام عليكم », « Bienvenue à la mosquée » à droite ; fond blanc, sans bandeau |
| 03_telephone | Rappel « Téléphone en silencieux » ; pictogramme téléphone barré dans un cercle doré |
| 04_permis | 2 colonnes. À gauche : « Bonne nouvelle · الحمد لله », « Le permis de construire est accordé », aménagement du lieu de culte (nouvel étage + extension), 4 novembre 2026 (arrêté du maire), + 378 m² de surface créée. À droite : extraits du vrai arrêté (en-tête de la mairie de Longuenesse et article 1) sur une feuille inclinée, avec une coche verte |
| 05_avancement | Frise « Notre mosquée avance · ما شاء الله » : janvier 2024 achat du local 380 000 € → 4 novembre 2026 permis accordé → aujourd'hui 100 000 € récoltés pour le gros œuvre → prochaine étape : travaux de gros œuvre |

La vidéo actuelle enchaîne les écrans 03, 04 et 05.

## Données du projet de travaux (source : arrêté de permis de construire)

- Dossier PC 062525 26 00010, déposé complet le 24/07/2026, accordé par le maire de Longuenesse le 04/11/2026.
- Projet : aménagement d'un lieu de culte avec création d'un étage et extension, 9063 impasse Guy Mollet, 62219 Longuenesse.
- Surface de plancher : 888 m² existants et 378 m² créés. Avis favorables de la commission de sécurité incendie et de la commission d'accessibilité.
- Recours des tiers possible pendant 2 mois à partir du début de l'affichage sur le terrain. Sur les écrans, écrire « accordé » et non « définitif ».
- Ne pas afficher la signature du maire ni le nom du représentant de l'association.

## Pistes ouvertes

- Horaires de prière : ils changent chaque jour, donc prévoir une image par semaine ou par mois générée à partir d'un fichier de données. Plus tard, éventuellement, un affichage en direct (box Android ou Raspberry Pi avec une page web hébergée sur le VPS).
- Barre de progression de la collecte pour le gros œuvre (« 100 000 € sur X € »), à faire quand l'objectif chiffré sera connu.
- Animations : faire apparaître les éléments un par un (frise, coche, chiffres) dans la vidéo MP4.
- Test sur la télé (modèle inconnu) : vérifier le rognage avec la mire, la durée réglable du diaporama et la lecture en boucle.
