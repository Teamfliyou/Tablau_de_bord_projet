import { VIDEO } from "./theme";
import { parseScript, wordsOf } from "./scriptParser";
import type { ParsedScript, Scene, VideoPlan } from "./types";

/** Débit de parole moyen d'une voix off française (mots / seconde). */
const WORDS_PER_SECOND = 2.6;
const MIN_SCENE_SECONDS = 2.4;
const PADDING_SECONDS = 0.7;
export const TRANSITION_FRAMES = 15;

export function naturalSeconds(text: string): number {
  return Math.max(MIN_SCENE_SECONDS, wordsOf(text).length / WORDS_PER_SECOND + PADDING_SECONDS);
}

/** Répartit `total` frames selon des poids, somme exacte (plus grand reste). */
function distribute(weights: number[], total: number): number[] {
  const sum = weights.reduce((a, b) => a + b, 0);
  const raw = weights.map((w) => (w / sum) * total);
  const floors = raw.map(Math.floor);
  let rest = total - floors.reduce((a, b) => a + b, 0);
  const order = raw.map((r, i) => [r - Math.floor(r), i] as const).sort((a, b) => b[0] - a[0]);
  for (const [, i] of order) {
    if (rest-- <= 0) break;
    floors[i]++;
  }
  return floors;
}

/**
 * Construit le plan de montage.
 * Durée totale = voix off (si fournie) > @duration > durée naturelle du texte.
 * Les scènes se chevauchent de `transitionFrames` : total = Σ durées − (n−1)·T.
 */
export function planVideo(parsed: ParsedScript, opts: { audioSeconds?: number } = {}): VideoPlan {
  const { fps, width, height } = VIDEO;
  const n = parsed.scenes.length;
  const T = n > 1 ? TRANSITION_FRAMES : 0;
  const weights = parsed.scenes.map((s) => naturalSeconds(s.text));

  const targetSeconds = opts.audioSeconds ?? parsed.targetSeconds;
  const natural = Math.round(weights.reduce((a, b) => a + b, 0) * fps) - (n - 1) * T;
  const totalFrames = targetSeconds ? Math.round(targetSeconds * fps) : natural;

  const durations = distribute(weights, totalFrames + (n - 1) * T);
  const scenes: Scene[] = parsed.scenes.map((s, i) => ({ ...s, durationInFrames: durations[i] }));
  return {
    title: parsed.title,
    theme: parsed.theme,
    fps, width, height,
    voice: parsed.voice,
    transitionFrames: T,
    scenes,
    totalFrames,
  };
}

export function planFromText(source: string, audioSeconds?: number): VideoPlan {
  return planVideo(parseScript(source), { audioSeconds });
}

/** Mots de sous-titres avec leur fenêtre en frames absolues. */
export type TimedWord = { text: string; from: number; to: number; sceneId: number };

export function buildSubtitleWords(plan: VideoPlan): TimedWord[] {
  const out: TimedWord[] = [];
  let start = 0;
  plan.scenes.forEach((scene, i) => {
    const T = plan.transitionFrames;
    // La narration commence après l'entrée de la scène et finit avant sa sortie.
    const from = start + (i === 0 ? 4 : Math.round(T / 2));
    const to = start + scene.durationInFrames - (i === plan.scenes.length - 1 ? 4 : Math.round(T / 2));
    const words = wordsOf(scene.text);
    const w = words.map((x) => x.length + 1.5);
    const sum = w.reduce((a, b) => a + b, 0);
    let acc = 0;
    words.forEach((text, k) => {
      const a = from + Math.round((acc / sum) * (to - from));
      acc += w[k];
      const b = from + Math.round((acc / sum) * (to - from));
      out.push({ text, from: a, to: b, sceneId: scene.id });
    });
    start += scene.durationInFrames - T;
  });
  return out;
}
