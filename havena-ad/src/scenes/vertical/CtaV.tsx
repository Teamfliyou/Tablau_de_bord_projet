import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { easeOut, progress, soft } from "../../animations/easing";
import { Backdrop } from "../../components/Backdrop";
import { Logo } from "../../components/Logo";
import { ProductVisual } from "../../components/ProductVisual";
import { RevealText } from "../../components/RevealText";
import { BRAND, PRODUCTS } from "../../data/brand";
import { COLORS, FONTS } from "../../styles/theme";

/** 23–30 s (9:16) : dernier visuel, logo, appel à l'action et adresse du site. */
export const CtaV: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const v = soft(frame, fps, 4);
  const glow = 0.5 + 0.5 * Math.sin(frame / 14);
  const sweep = progress(frame, 80, 40, easeOut);
  return (
    <AbsoluteFill>
      <Backdrop />
      <div style={{ position: "absolute", left: 110, top: 150, opacity: Math.min(1, v * 1.4), transform: `translateY(${(1 - v) * 200}px) scale(${1 + 0.03 * progress(frame, 0, 200, (t) => t)})`, borderRadius: 24, overflow: "hidden", border: `1px solid ${COLORS.brass}66`, boxShadow: `0 50px 120px #000000cc, 0 0 ${60 + 40 * glow}px ${COLORS.brass}33` }}>
        <ProductVisual product={PRODUCTS[2]} uid="ctav" width={860} ratio={0.75} />
      </div>
      <div style={{ position: "absolute", left: 260, top: 870 }}><Logo width={560} delay={2} /></div>
      <div style={{ position: "absolute", left: 60, right: 60, top: 1250, textAlign: "center" }}>
        <RevealText delay={34} style={{ fontFamily: FONTS.serif, fontWeight: 600, fontSize: 92, color: COLORS.cream }}>{BRAND.cta}</RevealText>
        <div style={{ height: 44 }} />
        <div style={{ position: "relative", display: "inline-block", overflow: "hidden", padding: "26px 70px", borderRadius: 999, background: `linear-gradient(90deg, ${COLORS.brass}, ${COLORS.brassLight})`, transform: `scale(${0.8 + 0.2 * soft(frame, fps, 56)})`, opacity: progress(frame, 56, 14), boxShadow: `0 0 ${30 + 30 * glow}px ${COLORS.brass}66` }}>
          <span style={{ fontFamily: FONTS.sans, fontWeight: 600, fontSize: 60, letterSpacing: "0.08em", color: COLORS.ink }}>{BRAND.url}</span>
          <div style={{ position: "absolute", top: 0, bottom: 0, width: 120, left: `${-30 + sweep * 150}%`, background: "linear-gradient(90deg, transparent, #ffffff88, transparent)", transform: "skewX(-20deg)" }} />
        </div>
        <div style={{ marginTop: 34, opacity: progress(frame, 70, 18), fontFamily: FONTS.sans, fontWeight: 300, fontSize: 34, color: COLORS.muted }}>Devis gratuit sous 72 h</div>
      </div>
    </AbsoluteFill>
  );
};
