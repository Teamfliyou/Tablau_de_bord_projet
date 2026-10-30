import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { easeInOut } from "../animations/easing";
import { WIPE_HALF } from "../animations/timing";
import { COLORS } from "../styles/theme";
import { interpolate } from "remotion";

/**
 * Transition « volet » : deux panneaux inclinés balaient l'écran et masquent la coupe, centrée sur la frame `at`.
 * `dir` donne le sens du balayage.
 */
export const PanelWipe: React.FC<{ at: number; dir?: "ltr" | "rtl" }> = ({ at, dir = "ltr" }) => {
  const frame = useCurrentFrame();
  const local = frame - (at - WIPE_HALF);
  if (local < 0 || local > WIPE_HALF * 2 + 4) return null;
  const sign = dir === "ltr" ? 1 : -1;
  const panel = (delay: number, color: string, z: number) => {
    const p = interpolate(local - delay, [0, WIPE_HALF * 2], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: easeInOut });
    return (
      <AbsoluteFill key={z} style={{ background: color, width: "135%", left: "-17%", transform: `translateX(${sign * (-100 + 200 * p)}%) skewX(${-14 * sign}deg)`, zIndex: z }} />
    );
  };
  return <AbsoluteFill style={{ overflow: "hidden", zIndex: 50 }}>{panel(0, COLORS.ink, 2)}{panel(3, COLORS.brass, 1)}{panel(-3, COLORS.brassLight, 0)}</AbsoluteFill>;
};
