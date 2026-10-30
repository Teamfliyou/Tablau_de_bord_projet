import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { easeOut, progress, soft } from "../../animations/easing";
import { Backdrop } from "../../components/Backdrop";
import { ProductVisual } from "../../components/ProductVisual";
import { RevealText } from "../../components/RevealText";
import { ARGUMENTS, PRODUCTS } from "../../data/brand";
import { COLORS, FONTS } from "../../styles/theme";

const ICONS: Record<string, string[]> = {
  layers: ["M12 3l9 5-9 5-9-5 9-5z", "M3 12.5l9 5 9-5", "M3 17l9 5 9-5"],
  clock: ["M12 3a9 9 0 100 18 9 9 0 000-18z", "M12 7v5l3 2"],
  user: ["M12 11a4 4 0 100-8 4 4 0 000 8z", "M4 21c0-4.4 3.6-8 8-8s8 3.6 8 8"],
};

/** 16–23 s (9:16) : trois arguments empilés sur un décor flouté. */
export const ArgumentsV: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  return (
    <AbsoluteFill>
      <Backdrop />
      <div style={{ position: "absolute", left: -500, top: 300, opacity: 0.25, filter: "blur(14px)", transform: `translateX(${-frame * 0.6}px)` }}>
        <ProductVisual product={PRODUCTS[0]} uid="argv" width={2400} />
      </div>
      <AbsoluteFill style={{ background: `${COLORS.ink}b8` }} />
      <div style={{ position: "absolute", left: 0, right: 0, top: 190, textAlign: "center" }}>
        <RevealText delay={2} style={{ fontFamily: FONTS.serif, fontWeight: 500, fontSize: 108, color: COLORS.cream }}>
          Pourquoi <span style={{ fontStyle: "italic", color: COLORS.brassLight }}>Havena</span> ?
        </RevealText>
      </div>
      <div style={{ position: "absolute", left: 70, right: 70, top: 470, display: "flex", flexDirection: "column", gap: 44 }}>
        {ARGUMENTS.map((a, i) => {
          const d = 16 + i * 22;
          const s = soft(frame, fps, d);
          const border = progress(frame, d, 40);
          const draw = progress(frame, d + 8, 30, easeOut);
          return (
            <div key={a.title} style={{ position: "relative", height: 310, boxSizing: "border-box", padding: "0 56px", display: "flex", alignItems: "center", gap: 48, borderRadius: 28, background: "linear-gradient(160deg,#ffffff16,#ffffff06)", opacity: Math.min(1, s * 1.5), transform: `translateX(${(1 - s) * 220}px)` }}>
              <svg style={{ position: "absolute", inset: 0 }} width="100%" height="100%">
                <rect x={1} y={1} width="calc(100% - 2px)" height="calc(100% - 2px)" rx={28} fill="none" stroke={COLORS.brass} strokeWidth={2} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - border} opacity={0.8} />
              </svg>
              <svg width={120} height={120} viewBox="0 0 24 24" fill="none" stroke={COLORS.brass} strokeWidth={1.3} strokeLinecap="round" strokeLinejoin="round" style={{ flex: "none" }}>
                {ICONS[a.icon].map((dd, k) => <path key={k} d={dd} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - draw} />)}
              </svg>
              <div>
                <div style={{ fontFamily: FONTS.serif, fontWeight: 600, fontSize: 84, lineHeight: 1.05, color: COLORS.cream, whiteSpace: "nowrap" }}>{a.title}</div>
                <div style={{ marginTop: 14, fontFamily: FONTS.sans, fontWeight: 300, fontSize: 32, lineHeight: 1.3, color: COLORS.muted }}>{a.text}</div>
              </div>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};
