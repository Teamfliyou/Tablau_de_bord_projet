import React from "react";
import { Img, staticFile, useCurrentFrame } from "remotion";
import { BRAND } from "../data/brand";
import { progress } from "../animations/easing";
import { COLORS, FONTS } from "../styles/theme";

/** Logo animé : pictogramme tracé (arche) + nom espacé. Si BRAND.logo est fourni, l'image officielle est utilisée. */
export const Logo: React.FC<{ size?: number; delay?: number; color?: string }> = ({ size = 120, delay = 0, color = COLORS.cream }) => {
  const frame = useCurrentFrame();
  if (BRAND.logo) {
    const o = progress(frame, delay, 20);
    return <Img src={staticFile(BRAND.logo)} style={{ height: size * 1.4, opacity: o, transform: `scale(${0.92 + 0.08 * o})` }} />;
  }
  const draw = progress(frame, delay, 34);
  const letters = BRAND.name.split("");
  const spacing = 0.7 - 0.4 * progress(frame, delay + 6, 50);
  return (
    <div style={{ display: "flex", alignItems: "center", gap: size * 0.28 }}>
      <svg width={size * 0.8} height={size * 0.8} viewBox="0 0 60 60" fill="none" stroke={COLORS.brass} strokeWidth={2.6} strokeLinecap="round" strokeLinejoin="round">
        <path d="M10 52 V26 Q10 8 30 8 Q50 8 50 26 V52" pathLength={1} strokeDasharray={1} strokeDashoffset={1 - draw} />
        <path d="M20 52 V34 H40 V52" pathLength={1} strokeDasharray={1} strokeDashoffset={1 - progress(frame, delay + 14, 28)} />
        <path d="M6 52 H54" pathLength={1} strokeDasharray={1} strokeDashoffset={1 - progress(frame, delay + 8, 24)} />
      </svg>
      <div style={{ fontFamily: FONTS.sans, fontWeight: 600, fontSize: size * 0.62, color, letterSpacing: `${spacing}em`, whiteSpace: "nowrap" }}>
        {letters.map((l, i) => (
          <span key={i} style={{ opacity: progress(frame, delay + 8 + i * 3, 14) }}>{l}</span>
        ))}
      </div>
    </div>
  );
};
