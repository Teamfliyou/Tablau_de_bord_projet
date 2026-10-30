import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { easeOut, progress, soft } from "../animations/easing";
import { Backdrop } from "../components/Backdrop";
import { ProductVisual } from "../components/ProductVisual";
import { RevealText } from "../components/RevealText";
import { ARGUMENTS, PRODUCTS } from "../data/brand";
import { COLORS, FONTS } from "../styles/theme";

const ICONS: Record<string, string[]> = {
  euro: ["M12 3a9 9 0 100 18 9 9 0 000-18z", "M15.5 9a4 4 0 100 6", "M7.5 11h5.5", "M7.5 13.5h5"],
  layers: ["M12 3l9 5-9 5-9-5 9-5z", "M3 12.5l9 5 9-5", "M3 17l9 5 9-5"],
  clock: ["M12 3a9 9 0 100 18 9 9 0 000-18z", "M12 7v5l3 2"],
  user: ["M12 11a4 4 0 100-8 4 4 0 000 8z", "M4 21c0-4.4 3.6-8 8-8s8 3.6 8 8"],
  palette: ["M12 3a9 9 0 100 18c1.5 0 2-1 1.5-2s0-2 1.5-2h2a3 3 0 003-3c0-6-4-11-8-11z", "M8 11a1 1 0 100 .01", "M12 8a1 1 0 100 .01", "M16 10a1 1 0 100 .01"],
  design: ["M4 20L20 4", "M4 20l3-8 5 5-8 3z", "M14 4l6 6", "M16 2l6 6"],
};

/** 16–23 s : trois arguments (prix, finitions, design) sur un décor de cuisine assombri et flouté. */
export const ArgumentsScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  return (
    <AbsoluteFill>
      <Backdrop />
      <div style={{ position: "absolute", inset: -40, opacity: 0.28, filter: "blur(14px)", transform: `scale(${1.15 + frame * 0.0006}) translateX(${-frame * 0.5}px)` }}>
        <ProductVisual product={PRODUCTS[0]} uid="arg" width={2000} />
      </div>
      <AbsoluteFill style={{ background: `${COLORS.ink}b8` }} />
      <div style={{ position: "absolute", left: 0, right: 0, top: 130, textAlign: "center" }}>
        <RevealText delay={2} style={{ fontFamily: FONTS.serif, fontWeight: 500, fontSize: 92, color: COLORS.cream }}>
          Pourquoi <span style={{ fontStyle: "italic", color: COLORS.brassLight }}>Havena</span> ?
        </RevealText>
      </div>
      <div style={{ position: "absolute", left: 120, right: 120, top: 380, display: "flex", gap: 60 }}>
        {ARGUMENTS.map((a, i) => {
          const d = 18 + i * 22;
          const s = soft(frame, fps, d);
          const border = progress(frame, d, 40);
          const draw = progress(frame, d + 8, 30, easeOut);
          return (
            <div key={a.title} style={{ flex: 1, height: 480, position: "relative", padding: 48, borderRadius: 28, background: "linear-gradient(160deg,#ffffff16,#ffffff06)", opacity: Math.min(1, s * 1.5), transform: `translateY(${(1 - s) * 90}px)`, boxSizing: "border-box" }}>
              <svg style={{ position: "absolute", inset: 0 }} width="100%" height="100%">
                <rect x={1} y={1} width="calc(100% - 2px)" height="calc(100% - 2px)" rx={28} fill="none" stroke={COLORS.brass} strokeWidth={2} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - border} opacity={0.8} />
              </svg>
              <svg width={110} height={110} viewBox="0 0 24 24" fill="none" stroke={COLORS.brass} strokeWidth={1.3} strokeLinecap="round" strokeLinejoin="round">
                {ICONS[a.icon].map((dd, k) => <path key={k} d={dd} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - draw} />)}
              </svg>
              <div style={{ marginTop: 48, fontFamily: FONTS.serif, fontWeight: 600, fontSize: 66, lineHeight: 1.05, whiteSpace: "nowrap", color: COLORS.cream }}>{a.title}</div>
              <div style={{ marginTop: 20, fontFamily: FONTS.sans, fontWeight: 300, fontSize: 30, lineHeight: 1.3, color: COLORS.muted }}>{a.text}</div>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};
