import React from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import { pop, progress } from "../lib/anim";
import { FONT_STACK } from "../lib/theme";

/**
 * Typographie cinématique : mots qui montent avec flou, mise à l'échelle et
 * léger décalage. Les `highlight` (index de mots) prennent la couleur d'accent.
 */
export const KineticText: React.FC<{
  text: string;
  color: string;
  accent: string;
  fontSize?: number;
  delay?: number;
  stagger?: number;
  highlightLast?: number;
  align?: "center" | "left";
  weight?: number;
}> = ({ text, color, accent, fontSize = 120, delay = 0, stagger = 4, highlightLast = 1, align = "center", weight = 900 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const words = text.split(/\s+/).filter(Boolean);
  return (
    <div
      style={{
        display: "flex", flexWrap: "wrap", justifyContent: align === "center" ? "center" : "flex-start",
        gap: `${fontSize * 0.1}px ${fontSize * 0.26}px`, fontFamily: FONT_STACK, fontWeight: weight,
        fontSize, lineHeight: 1.05, textAlign: align, textTransform: "uppercase", letterSpacing: -2,
      }}
    >
      {words.map((w, i) => {
        const d = delay + i * stagger;
        const s = pop(frame, fps, d, 16);
        const blur = (1 - progress(frame, d, 14)) * 16;
        const isAccent = i >= words.length - highlightLast;
        return (
          <span
            key={i}
            style={{
              display: "inline-block",
              opacity: Math.min(1, s * 1.6),
              transform: `translateY(${(1 - s) * 90}px) scale(${0.85 + 0.15 * s}) rotate(${(1 - s) * 4}deg)`,
              filter: `blur(${blur}px)`,
              color: isAccent ? accent : color,
              textShadow: isAccent ? `0 0 40px ${accent}88` : "0 6px 30px #00000088",
            }}
          >
            {w}
          </span>
        );
      })}
    </div>
  );
};
