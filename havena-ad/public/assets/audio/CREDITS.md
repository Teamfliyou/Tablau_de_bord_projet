# Musique

**« Chill lofi inspired »** — isaiah658 (OpenGameArt.org)
Licence **CC0 1.0** (domaine public) : usage commercial et publicitaire autorisé, aucune mention obligatoire.
Source : https://opengameart.org/content/chill-lofi-inspired (fichier `ChillLofi.ogg`, 2 min).

`musique-chill-lofi.mp3` = les 30 premières secondes, normalisées à -16 LUFS, fondu d'entrée 0,6 s et fondu de sortie 3 s
(`ffmpeg -t 30 -i ChillLofi.ogg -af "loudnorm=I=-16:TP=-1.5:LRA=7,afade=t=in:st=0:d=0.6,afade=t=out:st=27:d=3" …`).
Pour changer de musique : remplacer le fichier et ajuster `BRAND.music` dans `src/data/brand.ts`.
