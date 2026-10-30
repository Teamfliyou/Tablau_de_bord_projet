import { Easing, interpolate, spring } from "remotion";

export const easeOut = Easing.bezier(0.16, 1, 0.3, 1);
export const easeInOut = Easing.bezier(0.76, 0, 0.24, 1);

/** Progression 0→1 entre `from` et `from + duration`. */
export const progress = (frame: number, from: number, duration: number, easing = easeOut) =>
  interpolate(frame, [from, from + duration], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp", easing });

/** Ressort doux, sans rebond excessif (rendu « premium »). */
export const soft = (frame: number, fps: number, delay = 0) =>
  spring({ frame: frame - delay, fps, config: { damping: 22, stiffness: 110, mass: 0.9 } });
