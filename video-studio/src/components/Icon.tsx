import React from "react";
import { useCurrentFrame } from "remotion";
import { easeOut, progress } from "../lib/anim";
import type { IconName } from "../lib/types";

/** Icônes SVG 24×24 (trait). Chaque chemin est tracé progressivement. */
const PATHS: Record<IconName, string[]> = {
  rocket: ["M12 2c3 2 5 6 5 10l-2 4H9l-2-4c0-4 2-8 5-10z", "M9 16l-3 3 1-5", "M15 16l3 3-1-5", "M12 9.5a1.2 1.2 0 100 .01", "M10 20c.5 1 1.2 1.8 2 2 .8-.2 1.5-1 2-2"],
  bulb: ["M9 18h6", "M10 21h4", "M12 3a6 6 0 00-4 10.5c.7.7 1 1.5 1 2.5h6c0-1 .3-1.8 1-2.5A6 6 0 0012 3z"],
  chart: ["M3 3v18h18", "M7 15l4-5 3 3 5-7", "M16 6h3v3"],
  clock: ["M12 3a9 9 0 100 18 9 9 0 000-18z", "M12 7v5l3 2"],
  target: ["M12 3a9 9 0 100 18 9 9 0 000-18z", "M12 7a5 5 0 100 10 5 5 0 000-10z", "M12 11a1 1 0 100 2 1 1 0 000-2z"],
  shield: ["M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6l8-3z", "M8.5 12l2.5 2.5L16 9.5"],
  heart: ["M12 20s-8-5-8-11a4.5 4.5 0 018-2.8A4.5 4.5 0 0120 9c0 6-8 11-8 11z"],
  star: ["M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9L12 3z"],
  bolt: ["M13 2L4 14h7l-1 8 9-12h-7l1-8z"],
  check: ["M12 3a9 9 0 100 18 9 9 0 000-18z", "M8 12.5l3 3 5-6"],
  users: ["M9 11a3.5 3.5 0 100-7 3.5 3.5 0 000 7z", "M2.5 20c0-3.5 3-6 6.5-6s6.5 2.5 6.5 6", "M16 4.5a3.5 3.5 0 010 6.5", "M18 14c2 .6 3.5 2.4 3.5 5"],
  globe: ["M12 3a9 9 0 100 18 9 9 0 000-18z", "M3 12h18", "M12 3c3 3 3 15 0 18", "M12 3c-3 3-3 15 0 18"],
  code: ["M8 7l-5 5 5 5", "M16 7l5 5-5 5", "M14 4l-4 16"],
  play: ["M12 3a9 9 0 100 18 9 9 0 000-18z", "M10 8.5v7l6-3.5-6-3.5z"],
  arrow: ["M4 12h16", "M14 6l6 6-6 6"],
  money: ["M12 3a9 9 0 100 18 9 9 0 000-18z", "M14.5 9.2c-.5-.8-1.4-1.2-2.5-1.2-1.4 0-2.5.7-2.5 1.8s1 1.5 2.5 2 2.5.9 2.5 2-1.1 1.8-2.5 1.8c-1.1 0-2-.4-2.5-1.2", "M12 6.5V8", "M12 16v1.5"],
};

export const Icon: React.FC<{
  name: IconName;
  size?: number;
  color: string;
  glow?: string;
  /** frame local de début du tracé */
  delay?: number;
  strokeWidth?: number;
}> = ({ name, size = 220, color, glow, delay = 0, strokeWidth = 1.4 }) => {
  const frame = useCurrentFrame();
  const draw = progress(frame, delay, 28, easeOut);
  const fill = progress(frame, delay + 20, 18);
  return (
    <svg
      width={size} height={size} viewBox="0 0 24 24" fill="none"
      stroke={color} strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round"
      style={{ filter: glow ? `drop-shadow(0 0 ${size * 0.08}px ${glow})` : undefined, overflow: "visible" }}
    >
      {PATHS[name].map((d, i) => (
        <path key={i} d={d} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - draw}
          fill={color} fillOpacity={fill * 0.08} />
      ))}
    </svg>
  );
};

/** Pastille ronde avec anneau animé autour d'une icône. */
export const IconBadge: React.FC<{ name: IconName; color: string; glow: string; size?: number; delay?: number }> = ({
  name, color, glow, size = 360, delay = 0,
}) => {
  const frame = useCurrentFrame();
  const ring = progress(frame, delay, 40);
  const r = size / 2 - 6;
  const circ = 2 * Math.PI * r;
  return (
    <div style={{ position: "relative", width: size, height: size, display: "flex", alignItems: "center", justifyContent: "center" }}>
      <svg width={size} height={size} style={{ position: "absolute", transform: `rotate(${-90 + frame * 0.6}deg)` }}>
        <circle cx={size / 2} cy={size / 2} r={r} fill={`${color}10`} stroke={color} strokeWidth={5}
          strokeLinecap="round" strokeDasharray={circ} strokeDashoffset={circ * (1 - ring)}
          style={{ filter: `drop-shadow(0 0 18px ${glow})` }} />
      </svg>
      <Icon name={name} size={size * 0.52} color={color} glow={glow} delay={delay + 8} />
    </div>
  );
};
