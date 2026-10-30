import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { easeOut, progress } from "../../animations/easing";
import { Backdrop } from "../../components/Backdrop";
import { Logo } from "../../components/Logo";
import { ProductVisual } from "../../components/ProductVisual";
import { RevealText } from "../../components/RevealText";
import { PRODUCTS } from "../../data/brand";
import { COLORS, FONTS } from "../../styles/theme";

/** 3–8 s (9:16) : logo, promesse de marque, frise de cuisines qui défile. */
export const DiscoveryV: React.FC = () => {
  const frame = useCurrentFrame();
  const t = progress(frame, 24, 40, easeOut);
  return (
    <AbsoluteFill>
      <Backdrop />
      <div style={{ position: "absolute", left: 190, top: 260 }}><Logo width={700} delay={4} /></div>
      <div style={{ position: "absolute", left: 60, right: 60, top: 760, textAlign: "center" }}>
        <RevealText delay={40} style={{ fontFamily: FONTS.serif, fontWeight: 500, fontSize: 96, color: COLORS.cream }}>Cuisines sur mesure,</RevealText>
        <RevealText delay={48} style={{ fontFamily: FONTS.serif, fontWeight: 500, fontStyle: "italic", fontSize: 96, color: COLORS.brassLight }}>conçues pour durer.</RevealText>
        <div style={{ height: 3, width: 180 * progress(frame, 70, 30), background: COLORS.brass, margin: "44px auto 0" }} />
      </div>
      <div style={{ position: "absolute", top: 1180, left: 0, display: "flex", gap: 40, opacity: t, transform: `translateX(${60 - frame * 4.2}px) translateY(${(1 - t) * 80}px)` }}>
        {[...PRODUCTS, ...PRODUCTS].map((p, i) => (
          <div key={i} style={{ flex: "none", borderRadius: 20, overflow: "hidden", boxShadow: "0 40px 90px #000000aa", border: `1px solid ${COLORS.brass}44` }}>
            <ProductVisual product={p} uid={`dv${i}`} width={620} />
          </div>
        ))}
      </div>
    </AbsoluteFill>
  );
};
