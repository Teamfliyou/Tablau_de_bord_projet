import {loadFont} from '@remotion/fonts';
import {interpolate, spring, Easing, staticFile} from 'remotion';

// Polices embarquées dans public/fonts : la vidéo se rend sans connexion internet.
// Lora et Arimo sont des polices variables : un seul fichier couvre plusieurs graisses.
const fonts: [string, string, string][] = [
	['Lora', '600 700', 'lora-600.woff2'],
	['Poppins', '400', 'poppins-400.woff2'],
	['Poppins', '600 700', 'poppins-700.woff2'],
	['Poppins', '800', 'poppins-800.woff2'],
	['Amiri', '400', 'amiri-400.woff2'],
	['Arimo', '400 700', 'arimo-400.woff2'],
];
for (const [family, weight, file] of fonts) {
	loadFont({family, weight, url: staticFile(`fonts/${file}`)});
}

export const serif = 'Lora, serif';
export const sans = 'Poppins, sans-serif';
export const arabic = 'Amiri, serif';
// Police proche d'Arial, pour imiter le document officiel de la mairie.
export const docFont = 'Arimo, Arial, sans-serif';

export const C = {
	bg: '#f8f6f2',
	green: '#0f3d2e',
	gold: '#d4922a',
	goldSoft: '#ebc88f',
	text: '#4a5068',
	label: '#3d4255',
	border: '#ebe7e1',
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
