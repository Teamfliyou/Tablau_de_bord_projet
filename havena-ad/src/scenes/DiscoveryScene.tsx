import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { easeOut, progress } from "../animations/easing";
import { Backdrop } from "../components/Backdrop";
import { Logo } from "../components/Logo";
import { ProductVisual } from "../components/ProductVisual";
import { RevealText } from "../components/RevealText";
import { BRAND, PRODUCTS } from "../data/brand";
import { COLORS, FONTS } from "../styles/theme";

/** 3–8 s : le logo se dessine, la promesse de marque apparaît, trois cuisines défilent en parallaxe. */
export const DiscoveryScene: React.FC = () => {
  const frame = useCurrentFrame();
  const cards = [
    { p: PRODUCTS[0], x: 1010, y: 90, w: 560, speed: 0.55, delay: 20 },
    { p: PRODUCTS[1], x: 1420, y: 400, w: 440, speed: 0.9, delay: 30 },
    { p: PRODUCTS[2], x: 880, y: 600, w: 480, speed: 0.35, delay: 40 },
  ];
  return (
    <AbsoluteFill>
      <Backdrop />
      {cards.map(({ p, x, y, w, speed, delay }, i) => {
        const t = progress(frame, delay, 40, easeOut);
        const drift = -frame * speed;
        return (
          <div key={p.id} style={{ position: "absolute", left: x, top: y, opacity: t, transform: `translate(${(1 - t) * 160 + drift * 0.6}px, ${(1 - t) * 90 + drift * 0.25}px) scale(${0.9 + 0.1 * t})`, borderRadius: 18, overflow: "hidden", boxShadow: "0 40px 90px #000000aa", border: `1px solid ${COLORS.brass}44` }}>
            <ProductVisual product={p} uid={`d${i}`} reveal={progress(frame, delay, 30)} width={w} />
          </div>
        );
      })}
      <AbsoluteFill style={{ background: `linear-gradient(90deg, ${COLORS.ink} 0%, ${COLORS.ink}f2 36%, ${COLORS.ink}00 62%)` }} />
      <div style={{ position: "absolute", left: 120, top: 330 }}>
        <Logo size={120} delay={4} />
        <div style={{ height: 70 }} />
        <RevealText delay={44} style={{ fontFamily: FONTS.serif, fontWeight: 500, fontSize: 82, color: COLORS.cream }}>Créez la cuisine</RevealText>
        <RevealText delay={52} style={{ fontFamily: FONTS.serif, fontWeight: 500, fontStyle: "italic", fontSize: 82, color: COLORS.brassLight }}>qui vous ressemble.</RevealText>
        <div style={{ height: 3, width: 160 * progress(frame, 70, 30), background: COLORS.brass, marginTop: 36 }} />
      </div>
    </AbsoluteFill>
  );
};
