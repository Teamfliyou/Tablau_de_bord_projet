import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { COLORS } from "../styles/theme";

const NOISE = `url("data:image/svg+xml;utf8,${encodeURIComponent(
  "<svg xmlns='http://www.w3.org/2000/svg' width='256' height='256'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/></filter><rect width='256' height='256' filter='url(#n)'/></svg>",
)}")`;

/** Fond sombre chaud avec halo laiton qui dérive lentement. */
export const Backdrop: React.FC<{ tint?: string }> = ({ tint = COLORS.brass }) => {
  const frame = useCurrentFrame();
  const x = 50 + 18 * Math.sin(frame / 90);
  const y = 40 + 12 * Math.cos(frame / 120);
  return (
    <AbsoluteFill style={{ background: `radial-gradient(900px 700px at ${x}% ${y}%, ${tint}33, transparent 70%), linear-gradient(160deg, ${COLORS.ink2}, ${COLORS.ink})` }} />
  );
};

/** Finition globale : grain de pellicule, vignette douce, balayage de lumière. */
export const FilmFinish: React.FC = () => {
  const frame = useCurrentFrame();
  const sweep = ((frame % 300) / 300) * 260 - 80;
  return (
    <AbsoluteFill style={{ pointerEvents: "none" }}>
      <AbsoluteFill style={{ background: "radial-gradient(ellipse at center, transparent 58%, #00000070 100%)" }} />
      <AbsoluteFill style={{ background: `linear-gradient(105deg, transparent ${sweep - 12}%, #ffffff0d ${sweep}%, transparent ${sweep + 12}%)` }} />
      <AbsoluteFill style={{ backgroundImage: NOISE, backgroundSize: "256px 256px", backgroundPosition: `${(frame * 37) % 256}px ${(frame * 91) % 256}px`, opacity: 0.06, mixBlendMode: "overlay" }} />
    </AbsoluteFill>
  );
};
