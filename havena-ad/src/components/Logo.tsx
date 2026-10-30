import React from "react";
import { Img, staticFile, useCurrentFrame } from "remotion";
import { BRAND } from "../data/brand";
import { easeOut, progress } from "../animations/easing";
import { COLORS, FONTS } from "../styles/theme";

/** Zone utile du logo officiel (carré 900×900, texte noir sur fond clair) : x 12–88 %, y 25–70 %. */
const CROP = { x: 0.12, y: 0.25, w: 0.76, h: 0.45 };

/**
 * Logo animé. Avec BRAND.logo : le logo officiel, recadré sur sa partie utile, sur une carte claire qui se révèle par un masque.
 * Sinon : pictogramme tracé + nom.
 */
export const Logo: React.FC<{ width?: number; delay?: number }> = ({ width = 560, delay = 0 }) => {
  const frame = useCurrentFrame();
  if (BRAND.logo) {
    const h = (width * CROP.h) / CROP.w;
    const full = width / CROP.w;
    const reveal = progress(frame, delay, 28, easeOut);
    return (
      <div style={{ width, height: h, borderRadius: 22, overflow: "hidden", background: "#fdfcfa", position: "relative", clipPath: `inset(0 ${(1 - reveal) * 100}% 0 0 round 22px)`, boxShadow: "0 24px 70px #00000066" }}>
        <Img src={staticFile(BRAND.logo)} style={{ position: "absolute", width: full, height: full, left: -CROP.x * full, top: -CROP.y * full, transform: `scale(${1.08 - 0.08 * reveal})` }} />
      </div>
    );
  }
  const draw = progress(frame, delay, 34);
  return (
    <div style={{ fontFamily: FONTS.sans, fontWeight: 600, fontSize: width / 6, color: COLORS.cream, letterSpacing: "0.3em", opacity: draw }}>{BRAND.name}</div>
  );
};
