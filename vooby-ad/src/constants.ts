export const FPS = 30;
export const TOTAL_FRAMES = 750; // 25 s

// Musique : "Bass Vibes" ≈ 116 BPM, premier temps à ~0,07 s.
export const BEAT = 15.47; // frames par temps
export const BEAT_OFFSET = 2;
/** Arrondit une frame au temps musical le plus proche. */
export const snap = (frame: number) =>
	Math.round(Math.round((frame - BEAT_OFFSET) / BEAT) * BEAT + BEAT_OFFSET);

// Début de chaque scène (calé sur la grille rythmique). Cibles : 0/4/8/12/15/19/25 s.
export const S1 = 0;
export const S2 = snap(120);
export const S3 = snap(240);
export const S4 = snap(360);
export const S5 = snap(450);
export const S6 = snap(570);
export const OVERLAP = 14; // fondu enchaîné entre scènes (frames)

export const COLORS = {
	black: '#0b0b0c',
	white: '#f7f4ee',
	gold: '#d8bd8a',
	goldDeep: '#b8975a',
};

export const SERIF = "'Cormorant Garamond', Georgia, serif";
export const SANS = "'Montserrat', 'Inter', Arial, sans-serif";

// Zones de sécurité Reels / TikTok (px sur 1080x1920)
export const SAFE = {top: 250, bottom: 380, side: 70};
