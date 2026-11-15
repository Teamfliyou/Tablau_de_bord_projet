import {loadFont} from '@remotion/fonts';
import {interpolate, spring, Easing, staticFile} from 'remotion';

// Polices embarquées dans public/fonts : la vidéo se rend sans connexion internet.
// Lora et Arimo sont des polices variables : un seul fichier couvre plusieurs graisses.
const fonts: [string, string, string][] = [
	['Lora', '600 700', 'lora-600.woff2'],
	['Poppins', '400', 'poppins-400.woff2'],
	['Poppins', '600 700', 'poppins-700.woff2'],
	['Poppins', '800', 'poppins-800.woff2'],
	['DejaVu Sans', '400', 'dejavu-sans.ttf'],
	['Arimo', '400 700', 'arimo-400.woff2'],
];
for (const [family, weight, file] of fonts) {
	loadFont({family, weight, url: staticFile(`fonts/${file}`)});
}

export const serif = 'Lora, serif';
export const sans = 'Poppins, sans-serif';
export const arabic = "'DejaVu Sans', sans-serif";
// Police proche d'Arial, pour imiter le document officiel de la mairie.
export const docFont = 'Arimo, Arial, sans-serif';

// Charte graphique de l'affichage TV (voir CONTEXTE.md).
export const C = {
	bg: '#faf8f5', // blanc cassé, fond par défaut
	green: '#0f3a2e', // vert profond : titres, texte fort
	gold: '#d89020', // doré : accents, filets, sur-titres
	goldSoft: '#efcf98', // doré clair : surlignage
	goldHalo: 'rgba(216,144,32,0.22)',
	text: '#4a5568', // gris anthracite : texte courant
	label: '#2f3a4a',
	border: '#eaeaea', // gris clair : bordures
	grey: '#c9c9c9',
	red: '#c0392b',
	white: '#ffffff',
};

export const FPS = 30;

/** Apparition douce : fondu + montée. */
export const rise = (
	frame: number,
	fps: number,
	delay: number,
	distance = 40,
): React.CSSProperties => {
	const s = spring({frame: frame - delay, fps, config: {damping: 200}});
	return {
		opacity: s,
		transform: `translateY(${(1 - s) * distance}px)`,
	};
};

/** Apparition avec rebond (pour les pastilles / icônes). */
export const pop = (frame: number, fps: number, delay: number) =>
	spring({frame: frame - delay, fps, config: {damping: 11, stiffness: 140}});

/** Progression 0 → 1 entre deux frames, avec easing. */
export const progress = (frame: number, from: number, to: number) =>
	interpolate(frame, [from, to], [0, 1], {
		extrapolateLeft: 'clamp',
		extrapolateRight: 'clamp',
		easing: Easing.inOut(Easing.cubic),
	});

/** 100000 → "100 000" */
export const formatNumber = (n: number) =>
	Math.round(n)
		.toString()
		.replace(/\B(?=(\d{3})+(?!\d))/g, ' ');
