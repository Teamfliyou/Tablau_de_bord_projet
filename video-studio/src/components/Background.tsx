import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { rand } from "../lib/anim";
import type { Theme } from "../lib/theme";

/** Fond animé : dégradé, halos qui dérivent, grille en perspective, particules. */
export const Background: React.FC<{ theme: Theme }> = ({ theme }) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const t = frame / 30;

  const orb = (i: number, color: string, size: number) => {
    const x = width * (0.5 + 0.38 * Math.sin(t * 0.35 + i * 2.1));
    const y = height * (0.5 + 0.34 * Math.cos(t * 0.27 + i * 1.7));
    return (
      <div
        key={i}
        style={{
          position: "absolute", left: x - size / 2, top: y - size / 2,
          width: size, height: size, borderRadius: "50%",
          background: `radial-gradient(circle, ${color}55 0%, ${color}00 70%)`,
          filter: "blur(30px)",
        }}
      />
    );
  };

  const particles = Array.from({ length: 36 }, (_, i) => {
    const speed = 20 + rand(i + 1) * 60;
    const y = (height + 40 - ((frame * speed) / 30 + rand(i + 50) * height)) % (height + 80);
    const x = rand(i + 9) * width + Math.sin(t + i) * 14;
    const s = 3 + rand(i + 3) * 6;
    return (
      <div
        key={i}
        style={{
          position: "absolute", left: x, top: y < 0 ? y + height + 80 : y, width: s, height: s,
          borderRadius: "50%", background: i % 2 ? theme.accent : theme.accent2,
          opacity: 0.15 + rand(i + 7) * 0.45,
        }}
      />
    );
  });

  const gridShift = (frame * 1.2) % 120;
  return (
    <AbsoluteFill style={{ background: `linear-gradient(160deg, ${theme.bg2}, ${theme.bg1} 70%)` }}>
      {orb(0, theme.accent, 900)}
      {orb(1, theme.accent2, 800)}
      <AbsoluteFill
        style={{
          backgroundImage:
            `linear-gradient(${theme.accent}14 2px, transparent 2px), linear-gradient(90deg, ${theme.accent}14 2px, transparent 2px)`,
          backgroundSize: "120px 120px",
          backgroundPosition: `0 ${gridShift}px`,
          maskImage: "linear-gradient(to bottom, transparent 0%, #000 55%, transparent 100%)",
          WebkitMaskImage: "linear-gradient(to bottom, transparent 0%, #000 55%, transparent 100%)",
        }}
      />
      {particles}
      {/* Vignette */}
      <AbsoluteFill style={{ background: "radial-gradient(ellipse at center, transparent 55%, #000000aa 100%)" }} />
    </AbsoluteFill>
  );
};
