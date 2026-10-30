import React from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import { soft, progress } from "../animations/easing";

/** Ligne de texte qui monte depuis un masque (overflow caché), avec léger flou à l'entrée. */
export const RevealText: React.FC<{
  children: React.ReactNode; delay?: number; style?: React.CSSProperties; lineHeight?: number;
}> = ({ children, delay = 0, style, lineHeight = 1.1 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = soft(frame, fps, delay);
  const blur = (1 - progress(frame, delay, 14)) * 10;
  return (
    <div style={{ overflow: "hidden", lineHeight, paddingBottom: "0.12em", marginBottom: "-0.12em" }}>
      <div style={{ transform: `translateY(${(1 - s) * 110}%)`, filter: `blur(${blur}px)`, ...style }}>{children}</div>
    </div>
  );
};
