import React from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import { pop } from "../lib/anim";
import { FONT_STACK, type Theme } from "../lib/theme";
import type { TimedWord } from "../lib/planner";

const WORDS_PER_PAGE = 4;
const MAX_CHARS_PER_PAGE = 24;

/** Sous-titres animés façon TikTok : pages de 4 mots, mot actif surligné. */
export const Subtitles: React.FC<{ words: TimedWord[]; theme: Theme }> = ({ words, theme }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Page = groupe de mots consécutifs d'une même scène.
  const pages: TimedWord[][] = [];
  for (const w of words) {
    const last = pages[pages.length - 1];
    if (
      last && last.length < WORDS_PER_PAGE && last[0].sceneId === w.sceneId &&
      last.reduce((n, x) => n + x.text.length + 1, 0) + w.text.length <= MAX_CHARS_PER_PAGE
    ) last.push(w);
    else pages.push([w]);
  }
  const idx = pages.findIndex((p) => frame >= p[0].from && frame < p[p.length - 1].to);
  if (idx < 0) return null;
  const page = pages[idx];
  const pageIn = pop(frame, fps, page[0].from, 18);

  return (
    <div
      style={{
        position: "absolute", left: 60, right: 60, bottom: 250, display: "flex", justifyContent: "center",
        flexWrap: "wrap", gap: "6px 22px", fontFamily: FONT_STACK, fontWeight: 900, fontSize: 80,
        textTransform: "uppercase", textAlign: "center",
        transform: `translateY(${(1 - pageIn) * 40}px) scale(${0.9 + 0.1 * pageIn})`, opacity: Math.min(1, pageIn * 2),
      }}
    >
      {page.map((w, i) => {
        const active = frame >= w.from && frame < w.to;
        const done = frame >= w.to;
        const bump = active ? pop(frame, fps, w.from, 10) : 1;
        return (
          <span
            key={i}
            style={{
              display: "inline-block", color: active ? theme.accent : "#fff",
              opacity: active || done ? 1 : 0.45,
              transform: `scale(${active ? 1 + 0.12 * Math.min(1, bump) : 1})`,
              WebkitTextStroke: "10px #000000cc", paintOrder: "stroke fill",
              textShadow: active ? `0 0 36px ${theme.accent}` : "none",
            }}
          >
            {w.text}
          </span>
        );
      })}
    </div>
  );
};
