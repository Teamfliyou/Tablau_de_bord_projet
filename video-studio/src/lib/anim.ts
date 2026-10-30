import { interpolate, spring, Easing } from "remotion";

export const easeOut = Easing.bezier(0.16, 1, 0.3, 1);
export const easeInOut = Easing.bezier(0.65, 0, 0.35, 1);

/** Progression 0→1 entre deux frames avec easing. */
export function progress(frame: number, from: number, duration: number, easing = easeOut) {
  return interpolate(frame, [from, from + duration], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing,
  });
}

export function pop(frame: number, fps: number, delay = 0, damping = 14) {
  return spring({ frame: frame - delay, fps, config: { damping, stiffness: 120, mass: 0.7 } });
}

/** Aléatoire déterministe (identique à chaque rendu). */
export function rand(seed: number) {
  const x = Math.sin(seed * 127.1 + 311.7) * 43758.5453;
  return x - Math.floor(x);
}
