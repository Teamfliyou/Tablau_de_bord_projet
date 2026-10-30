/** Découpage de la pub (30 fps, 900 frames = 30 s). Les volets de transition sont centrés sur les frontières. */
export const SCENES = {
  hook: { from: 0, duration: 90 },
  discovery: { from: 90, duration: 150 },
  products: { from: 240, duration: 240 },
  arguments: { from: 480, duration: 210 },
  cta: { from: 690, duration: 210 },
} as const;

export const WIPE_HALF = 10;
