import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { easeInOut, progress } from "../animations/easing";
import { KitchenSVG } from "../components/KitchenSVG";
import { RevealText } from "../components/RevealText";
import { BRAND } from "../data/brand";
import { COLORS, FONTS } from "../styles/theme";

/** 0–3 s : zoom arrière cinématique depuis les suspensions jusqu'à la cuisine complète + accroche prix. */
export const HookScene: React.FC = () => {
  const frame = useCurrentFrame();
  const z = progress(frame, 0, 84, easeInOut);
  const scale = 2.6 - 1.5 * z;
  const reveal = 0.25 + 0.75 * progress(frame, 0, 45, easeInOut);
  return (
    <AbsoluteFill style={{ background: COLORS.ink, overflow: "hidden" }}>
      <div style={{ position: "absolute", left: 560, top: 40, width: 1500, height: 1000, transformOrigin: "75% 42%", transform: `scale(${scale}) rotate(${(1 - z) * 1.2}deg)` }}>
        <KitchenSVG finish="blanc" uid="hook" reveal={reveal} />
      </div>
      <AbsoluteFill style={{ background: `linear-gradient(90deg, ${COLORS.ink} 0%, ${COLORS.ink}f0 38%, ${COLORS.ink}00 78%)` }} />
      <div style={{ position: "absolute", left: 120, top: 330, width: 900 }}>
        <RevealText delay={8} style={{ fontFamily: FONTS.sans, fontWeight: 600, fontSize: 26, letterSpacing: "0.5em", color: COLORS.brass }}>{BRAND.name}</RevealText>
        <div style={{ height: 28 }} />
        <RevealText delay={14} style={{ fontFamily: FONTS.serif, fontWeight: 600, fontSize: 150, color: COLORS.cream }}>Votre cuisine,</RevealText>
        <RevealText delay={22} style={{ fontFamily: FONTS.serif, fontWeight: 600, fontStyle: "italic", fontSize: 150, color: COLORS.brassLight }}>
          à {BRAND.price}
        </RevealText>
      </div>
    </AbsoluteFill>
  );
};
