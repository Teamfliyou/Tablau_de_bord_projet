import React from "react";
import { AbsoluteFill, Sequence, useCurrentFrame } from "remotion";
import { easeInOut, easeOut, progress } from "../../animations/easing";
import { Backdrop } from "../../components/Backdrop";
import { ProductVisual } from "../../components/ProductVisual";
import { RevealText } from "../../components/RevealText";
import { PRODUCTS, type Product } from "../../data/brand";
import { COLORS, FONTS } from "../../styles/theme";

const SLOT = 80;
const OVERLAP = 22;
const TINTS: Record<string, string> = { "atelier-gris": "#8a8f94", "atelier-bois": COLORS.brass, scarlet: "#d9b97c" };

const SlideV: React.FC<{ product: Product; index: number }> = ({ product, index }) => {
  const frame = useCurrentFrame();
  const iris = index === 0 ? 1 : progress(frame, 0, OVERLAP, easeInOut);
  const push = progress(frame, 0, SLOT + OVERLAP, (t) => t);
  const inV = progress(frame, 4, 30, easeOut);
  return (
    <AbsoluteFill style={{ clipPath: index === 0 ? undefined : `circle(${iris * 2400}px at 540px 700px)` }}>
      <Backdrop tint={TINTS[product.id]} />
      <div style={{ position: "absolute", left: 60, top: 150, opacity: inV, transform: `translateY(${(1 - inV) * 160}px)` }}>
        <div style={{ width: 960, height: 900, borderRadius: 26, overflow: "hidden", boxShadow: "0 50px 120px #000000bb", border: `1px solid ${COLORS.brass}55` }}>
          <div style={{ transform: `scale(${1.14 - 0.1 * push})`, transformOrigin: "55% 55%" }}>
            <ProductVisual product={product} uid={`pv${index}`} width={1200} ratio={900 / 1200} />
          </div>
        </div>
      </div>
      <div style={{ position: "absolute", left: 60, top: 1120, fontFamily: FONTS.serif, fontSize: 80, lineHeight: 1, fontWeight: 600, color: "transparent", WebkitTextStroke: `2px ${COLORS.brass}66` }}>0{index + 1}</div>
      <div style={{ position: "absolute", left: 60, right: 60, top: 1150, textAlign: "center" }}>
        <RevealText delay={10} style={{ fontFamily: FONTS.sans, fontWeight: 600, fontSize: 28, letterSpacing: "0.4em", color: COLORS.brass }}>{product.collection.toUpperCase()}</RevealText>
        <div style={{ height: 18 }} />
        <RevealText delay={16} style={{ fontFamily: FONTS.serif, fontWeight: 600, fontSize: 104, color: COLORS.cream }}>{product.name}</RevealText>
        <div style={{ height: 14 }} />
        <RevealText delay={24} style={{ fontFamily: FONTS.sans, fontWeight: 300, fontSize: 32, color: COLORS.muted, letterSpacing: "0.03em" }}>{product.detail}</RevealText>
        <div style={{ height: 24 }} />
        {product.price ? (
          <div style={{ display: "flex", alignItems: "baseline", justifyContent: "center", gap: 18 }}>
            <RevealText delay={30} style={{ fontFamily: FONTS.serif, fontWeight: 600, fontSize: 132, color: COLORS.brassLight }}>{product.price}</RevealText>
            <RevealText delay={36} style={{ fontFamily: FONTS.sans, fontWeight: 300, fontSize: 36, color: COLORS.muted }}>TTC</RevealText>
          </div>
        ) : (
          <RevealText delay={30} style={{ fontFamily: FONTS.serif, fontStyle: "italic", fontWeight: 500, fontSize: 60, color: COLORS.brassLight }}>{product.tagline}</RevealText>
        )}
      </div>
    </AbsoluteFill>
  );
};

/** 8–16 s (9:16) : une cuisine par plan, ouverture circulaire entre chaque plan. */
export const ProductsV: React.FC = () => (
  <AbsoluteFill>
    {PRODUCTS.map((p, i) => (
      <Sequence key={p.id} from={i * SLOT} durationInFrames={i === PRODUCTS.length - 1 ? SLOT : SLOT + OVERLAP}>
        <SlideV product={p} index={i} />
      </Sequence>
    ))}
  </AbsoluteFill>
);
