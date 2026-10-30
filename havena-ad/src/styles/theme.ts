/** Charte graphique. La palette est une base premium « chaleureuse » : à ajuster avec les couleurs officielles Havena. */
export const COLORS = {
  ink: "#12100e",
  ink2: "#1d1915",
  cream: "#f4efe6",
  brass: "#c9a46a",
  brassLight: "#ecd5a5",
  sage: "#7d9a86",
  muted: "#b9b0a2",
} as const;

export const FONTS = {
  serif: "'Cormorant Garamond', Georgia, serif",
  sans: "'Montserrat', 'Helvetica Neue', Arial, sans-serif",
} as const;

export const VIDEO = { width: 1920, height: 1080, fps: 30, durationInFrames: 900 } as const;
