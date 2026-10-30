import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { easeInOut, progress } from "../../animations/easing";
import { ProductVisual } from "../../components/ProductVisual";
import { RevealText } from "../../components/RevealText";
import { BRAND, PRODUCTS } from "../../data/brand";
import { COLORS, FONTS } from "../../styles/theme";

/** 0–3 s (9:16) : photo plein cadre en zoom arrière, titre en dessous. */
export const HookV: React.FC = () => {
  const frame = useCurrentFrame();
  const z = progress(frame, 0, 84, easeInOut);
  const fade = 0.35 + 0.65 * progress(frame, 0, 30);
  return (
    <AbsoluteFill style={{ background: COLORS.ink }}>
      <div style={{ position: "absolute", left: 0, top: 0, width: 1080, height: 1080, overflow: "hidden" }}>
        <div style={{ opacity: fade, marginLeft: -255, transformOrigin: "62% 60%", transform: `scale(${1.45 - 0.4 * z})` }}>
          <ProductVisual product={PRODUCTS[1]} uid="hookv" width={1590} ratio={1080 / 1590} />
        </div>
      </div>
      <div style={{ position: "absolute", left: 0, top: 0, width: 1080, height: 1080, background: `linear-gradient(to bottom, transparent 55%, ${COLORS.ink} 100%)` }} />
      <div style={{ position: "absolute", left: 60, right: 60, top: 1010, textAlign: "center" }}>
        <RevealText delay={8} style={{ fontFamily: FONTS.sans, fontWeight: 600, fontSize: 30, letterSpacing: "0.5em", color: COLORS.brass }}>{BRAND.name}</RevealText>
        <div style={{ height: 30 }} />
        <RevealText delay={14} style={{ fontFamily: FONTS.serif, fontWeight: 600, fontSize: 170, color: COLORS.cream }}>La cuisine</RevealText>
        <RevealText delay={22} style={{ fontFamily: FONTS.serif, fontWeight: 600, fontStyle: "italic", fontSize: 170, color: COLORS.brassLight }}>à votre façon.</RevealText>
      </div>
    </AbsoluteFill>
  );
};
