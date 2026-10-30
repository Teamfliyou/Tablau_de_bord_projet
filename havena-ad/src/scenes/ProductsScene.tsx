import React from "react";
import { AbsoluteFill, Sequence, useCurrentFrame } from "remotion";
import { easeInOut, easeOut, progress } from "../animations/easing";
import { Backdrop } from "../components/Backdrop";
import { ProductVisual } from "../components/ProductVisual";
import { RevealText } from "../components/RevealText";
import { PRODUCTS, type Product } from "../data/brand";
import { COLORS, FONTS } from "../styles/theme";

const SLOT = 80;
const OVERLAP = 22;

const TINTS: Record<string, string> = { "atelier-gris": "#8a8f94", "atelier-bois": COLORS.brass, scarlet: "#d9b97c" };

/** Une cuisine mise en scène : texte à gauche, visuel en parallaxe à droite, ouverture en iris. */
const ProductSlide: React.FC<{ product: Product; index: number }> = ({ product, index }) => {
  const frame = useCurrentFrame();
  const iris = index === 0 ? 1 : progress(frame, 0, OVERLAP, easeInOut);
  const push = progress(frame, 0, SLOT + OVERLAP, (t) => t);
  const inV = progress(frame, 4, 30, easeOut);
  return (
    <AbsoluteFill style={{ clipPath: index === 0 ? undefined : `circle(${iris * 2300}px at 1360px 540px)` }}>
      <Backdrop tint={TINTS[product.id]} />
      <div style={{ position: "absolute", left: 130, top: 150, fontFamily: FONTS.serif, fontSize: 190, lineHeight: 1, fontWeight: 600, color: "transparent", WebkitTextStroke: `2px ${COLORS.brass}33`, transform: `translateX(${-frame * 0.3}px)` }}>
        0{index + 1}
      </div>
      <div style={{ position: "absolute", left: 130, top: 380, width: 740 }}>
        <RevealText delay={10} style={{ fontFamily: FONTS.sans, fontWeight: 600, fontSize: 24, letterSpacing: "0.4em", color: COLORS.brass }}>{product.collection.toUpperCase()}</RevealText>
        <div style={{ height: 22 }} />
        <RevealText delay={16} style={{ fontFamily: FONTS.serif, fontWeight: 600, fontSize: 104, color: COLORS.cream }}>{product.name}</RevealText>
        <div style={{ height: 22 }} />
        <RevealText delay={24} style={{ fontFamily: FONTS.sans, fontWeight: 300, fontSize: 30, color: COLORS.muted, letterSpacing: "0.03em" }}>{product.detail}</RevealText>
        <div style={{ height: 36 }} />
        {product.price ? (
          <div style={{ display: "flex", alignItems: "baseline", gap: 20 }}>
            <RevealText delay={30} style={{ fontFamily: FONTS.serif, fontWeight: 600, fontSize: 124, color: COLORS.brassLight }}>{product.price}</RevealText>
            <RevealText delay={36} style={{ fontFamily: FONTS.sans, fontWeight: 300, fontSize: 32, color: COLORS.muted }}>TTC</RevealText>
          </div>
        ) : (
          <RevealText delay={30} style={{ fontFamily: FONTS.serif, fontStyle: "italic", fontWeight: 500, fontSize: 54, color: COLORS.brassLight }}>{product.tagline}</RevealText>
        )}
      </div>
      <div style={{ position: "absolute", left: 880, top: 205, opacity: inV, transform: `translateX(${(1 - inV) * 140}px)` }}>
        <div style={{ width: 960, height: 640, borderRadius: 24, overflow: "hidden", boxShadow: "0 50px 120px #000000bb", border: `1px solid ${COLORS.brass}55` }}>
          <div style={{ transform: `scale(${1.14 - 0.1 * push}) translateX(${(0.5 - push) * 40}px)`, transformOrigin: "60% 50%" }}>
            <ProductVisual product={product} uid={`p${index}`} width={960} />
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};

/** 8–16 s : trois cuisines, une par plan, avec ouverture circulaire entre chaque plan. */
export const ProductsScene: React.FC = () => (
  <AbsoluteFill>
    {PRODUCTS.map((p, i) => (
      <Sequence key={p.id} from={i * SLOT} durationInFrames={i === PRODUCTS.length - 1 ? SLOT : SLOT + OVERLAP}>
        <ProductSlide product={p} index={i} />
      </Sequence>
    ))}
  </AbsoluteFill>
);
