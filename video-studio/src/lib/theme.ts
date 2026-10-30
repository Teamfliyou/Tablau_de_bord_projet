import type { ThemeName } from "./types";

export const VIDEO = { width: 1080, height: 1920, fps: 30 } as const;

export type Theme = {
  bg1: string;
  bg2: string;
  accent: string;
  accent2: string;
  text: string;
  muted: string;
};

export const THEMES: Record<ThemeName, Theme> = {
  neon: { bg1: "#07031a", bg2: "#1b0b3d", accent: "#00e5ff", accent2: "#ff2bd6", text: "#ffffff", muted: "#9aa4d6" },
  sunset: { bg1: "#1a0508", bg2: "#3d0b1e", accent: "#ffb703", accent2: "#fb5607", text: "#fff8ee", muted: "#d6a49a" },
  ocean: { bg1: "#02121f", bg2: "#06304a", accent: "#2ee6a6", accent2: "#3a86ff", text: "#f2fbff", muted: "#8fb4c9" },
};

export const FONT_STACK = "'Montserrat','Inter','Helvetica Neue',Arial,sans-serif";
