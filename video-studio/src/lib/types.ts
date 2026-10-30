export type SceneType =
  | "hook"
  | "stat"
  | "bars"
  | "line"
  | "list"
  | "question"
  | "outro";

export type IconName =
  | "rocket" | "bulb" | "chart" | "clock" | "target" | "shield"
  | "heart" | "star" | "bolt" | "check" | "users" | "globe"
  | "code" | "play" | "arrow" | "money";

export type DataPoint = { label: string; value: number };

export type Scene = {
  id: number;
  type: SceneType;
  /** Narration complète (sert aux sous-titres et à la durée). */
  text: string;
  /** Titre affiché à l'écran (par défaut : dérivé du texte). */
  title: string;
  icon: IconName;
  /** outro : texte du bouton */
  cta?: string;
  /** stat */
  value?: number;
  prefix?: string;
  suffix?: string;
  /** bars / line */
  data?: DataPoint[];
  /** list */
  items?: string[];
  /** Durée en frames, hors chevauchement de transition. */
  durationInFrames: number;
};

export type ThemeName = "neon" | "sunset" | "ocean";

export type ParsedScript = {
  title: string;
  theme: ThemeName;
  voice?: string;
  targetSeconds?: number;
  scenes: Omit<Scene, "durationInFrames">[];
};

export type VideoPlan = {
  title: string;
  theme: ThemeName;
  fps: number;
  width: number;
  height: number;
  voice?: string;
  transitionFrames: number;
  scenes: Scene[];
  totalFrames: number;
};
