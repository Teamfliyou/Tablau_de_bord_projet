#!/usr/bin/env bash
# Génère les vidéos pour la clé USB de la télé, au format défini dans CONTEXTE.md :
# H.264 yuv420p, profil high, niveau 4.0, 30 fps, sans son.
#   out/01_mosquee-tv.mp4         : une boucle (~37 s)
#   out/01_mosquee-tv_x20.mp4     : la même répétée 20 fois (télés qui ne répètent pas les vidéos)
# Variable optionnelle : CHROME=/chemin/vers/chrome si Remotion ne trouve pas de navigateur.
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p out

BROWSER_ARG=()
if [[ -n "${CHROME:-}" ]]; then BROWSER_ARG=(--browser-executable="$CHROME"); fi

npx remotion render MosqueeTV out/_brut.mp4 --crf=14 --concurrency=2 --timeout=120000 "${BROWSER_ARG[@]}"

ffmpeg -loglevel error -y -i out/_brut.mp4 -an \
	-c:v libx264 -profile:v high -level:v 4.0 -pix_fmt yuv420p -r 30 \
	-crf 18 -preset slow -movflags +faststart out/01_mosquee-tv.mp4

ffmpeg -loglevel error -y -stream_loop 19 -i out/01_mosquee-tv.mp4 -c copy -movflags +faststart \
	out/01_mosquee-tv_x20.mp4

rm out/_brut.mp4
ls -lh out/*.mp4
