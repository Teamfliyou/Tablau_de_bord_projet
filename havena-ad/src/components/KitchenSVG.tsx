import React from "react";
import { interpolate } from "remotion";

export type Finish = "blanc" | "sauge" | "anthracite";

type Palette = { wall: string; wall2: string; cab: string; cabShade: string; handle: string; counter: string; floor: string; floor2: string; tile: string; sky: string };

const PALETTES: Record<Finish, Palette> = {
  blanc: { wall: "#ebe5da", wall2: "#ddd5c7", cab: "#f6f2ea", cabShade: "#e1dacd", handle: "#3a3631", counter: "#d6b48a", floor: "#b98c5e", floor2: "#a37a50", tile: "#d9d1c2", sky: "#cfe3ee" },
  sauge: { wall: "#efe9df", wall2: "#e0d8ca", cab: "#7f9c88", cabShade: "#668471", handle: "#d9b97c", counter: "#f1ece2", floor: "#a67d55", floor2: "#926c47", tile: "#d8cfbf", sky: "#d6e6ea" },
  anthracite: { wall: "#cfc7ba", wall2: "#bdb4a6", cab: "#33383c", cabShade: "#262a2d", handle: "#c9a46a", counter: "#e4ddd0", floor: "#6f5b48", floor2: "#5e4c3b", tile: "#b9b0a2", sky: "#c4d4de" },
};

/**
 * Cuisine dessinée entièrement en SVG (viewBox 1200×800, ratio 3:2).
 * `reveal` (0→1) fait apparaître les éléments un à un ; omis = tout affiché.
 * `uid` évite les collisions d'identifiants de dégradés quand plusieurs cuisines sont à l'écran.
 */
export const KitchenSVG: React.FC<{ finish: Finish; uid: string; reveal?: number; style?: React.CSSProperties }> = ({
  finish, uid, reveal, style,
}) => {
  const p = PALETTES[finish];
  // Chaque groupe apparaît dans une fenêtre [a, b] de `reveal`.
  const grp = (a: number, b: number, dy = 24): React.CSSProperties => {
    if (reveal === undefined) return {};
    const t = interpolate(reveal, [a, b], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
    return { opacity: t, transform: `translateY(${(1 - t) * dy}px)` };
  };
  const id = (n: string) => `${n}-${uid}`;

  const wallDoors = [60, 200, 340].map((x, i) => (
    <g key={i}>
      <rect x={x + 3} y={93} width={134} height={232} rx={4} fill={p.cab} stroke={p.cabShade} strokeWidth={2} />
      <rect x={x + 120} y={250} width={5} height={52} rx={2.5} fill={p.handle} />
    </g>
  ));
  const baseDoors = [60, 233, 406, 579].map((x, i) => (
    <g key={i}>
      <rect x={x + 3} y={572} width={164} height={178} rx={4} fill={p.cab} stroke={p.cabShade} strokeWidth={2} />
      <rect x={x + 40} y={590} width={90} height={5} rx={2.5} fill={p.handle} />
    </g>
  ));
  const tiles = Array.from({ length: 9 }, (_, r) => Array.from({ length: 22 }, (_, c) => (
    <rect key={`${r}-${c}`} x={60 + c * 31 + (r % 2 ? 15 : 0)} y={334 + r * 23} width={29} height={21} rx={2} fill={p.tile} opacity={0.55} />
  )));
  const lamps = [880, 975, 1070].map((x, i) => (
    <g key={i}>
      <line x1={x} y1={0} x2={x} y2={360 + (i % 2) * 10} stroke={p.handle} strokeWidth={3} />
      <path d={`M${x - 34} ${402 + (i % 2) * 10} Q${x} ${340 + (i % 2) * 10} ${x + 34} ${402 + (i % 2) * 10} Z`} fill={finish === "anthracite" ? "#c9a46a" : "#2f2b27"} />
      <ellipse cx={x} cy={404 + (i % 2) * 10} rx={34} ry={5} fill="#ffe7b0" />
      <ellipse cx={x} cy={430 + (i % 2) * 10} rx={80} ry={60} fill={`url(#${id("glow")})`} />
    </g>
  ));

  return (
    <svg viewBox="0 0 1200 800" width="100%" height="100%" preserveAspectRatio="xMidYMid slice" style={{ display: "block", ...style }}>
      <defs>
        <linearGradient id={id("wall")} x1="0" y1="0" x2="0" y2="1"><stop offset="0" stopColor={p.wall} /><stop offset="1" stopColor={p.wall2} /></linearGradient>
        <linearGradient id={id("floor")} x1="0" y1="0" x2="0" y2="1"><stop offset="0" stopColor={p.floor2} /><stop offset="1" stopColor={p.floor} /></linearGradient>
        <linearGradient id={id("sky")} x1="0" y1="0" x2="0" y2="1"><stop offset="0" stopColor={p.sky} /><stop offset="1" stopColor="#fff8e8" /></linearGradient>
        <linearGradient id={id("light")} x1="0" y1="0" x2="1" y2="1"><stop offset="0" stopColor="#fff3cf" stopOpacity="0.55" /><stop offset="1" stopColor="#fff3cf" stopOpacity="0" /></linearGradient>
        <radialGradient id={id("glow")}><stop offset="0" stopColor="#ffe2a0" stopOpacity="0.75" /><stop offset="1" stopColor="#ffe2a0" stopOpacity="0" /></radialGradient>
        <linearGradient id={id("shadow")} x1="0" y1="0" x2="0" y2="1"><stop offset="0" stopColor="#000" stopOpacity="0.22" /><stop offset="1" stopColor="#000" stopOpacity="0" /></linearGradient>
      </defs>

      <g style={grp(0, 0.15, 0)}>
        <rect width={1200} height={650} fill={`url(#${id("wall")})`} />
        <rect y={640} width={1200} height={160} fill={`url(#${id("floor")})`} />
        {Array.from({ length: 6 }, (_, i) => <line key={i} x1={0} x2={1200} y1={668 + i * 26} y2={668 + i * 26} stroke="#00000018" strokeWidth={2} />)}
      </g>

      <g style={grp(0.1, 0.3)}>
        <rect x={800} y={60} width={340} height={270} rx={6} fill={p.handle} />
        <rect x={814} y={74} width={312} height={242} fill={`url(#${id("sky")})`} />
        <line x1={970} y1={74} x2={970} y2={316} stroke={p.handle} strokeWidth={8} />
        <line x1={814} y1={195} x2={1126} y2={195} stroke={p.handle} strokeWidth={8} />
        <path d="M814 316 L1126 74 L1126 120 L860 316 Z" fill={`url(#${id("light")})`} />
        <rect x={790} y={328} width={360} height={14} rx={3} fill={p.cab} />
      </g>

      <g style={grp(0.22, 0.42)}>{tiles}<rect x={60} y={334} width={690} height={206} fill={`url(#${id("shadow")})`} /></g>

      <g style={grp(0.3, 0.5, -30)}>
        <rect x={60} y={90} width={420} height={240} rx={5} fill={p.cabShade} />
        {wallDoors}
        <path d="M520 330 L560 150 L690 150 L730 330 Z" fill={finish === "anthracite" ? "#b0a89a" : "#b8b1a4"} />
        <rect x={595} y={0} width={60} height={152} fill="#a79f91" />
        <rect x={520} y={322} width={210} height={10} rx={3} fill="#8f887b" />
      </g>

      <g style={grp(0.42, 0.62, 40)}>
        <rect x={60} y={568} width={690} height={196} rx={5} fill={p.cabShade} />
        {baseDoors}
        <rect x={60} y={764} width={690} height={16} fill="#1f1c19" />
      </g>

      <g style={grp(0.55, 0.72, 16)}>
        <rect x={50} y={540} width={710} height={28} rx={4} fill={p.counter} />
        <rect x={50} y={540} width={710} height={6} rx={3} fill="#ffffff" opacity={0.45} />
        <rect x={560} y={532} width={150} height={9} rx={3} fill="#1b1917" />
        {[595, 635, 675].map((x) => <circle key={x} cx={x} cy={536} r={9} fill="#3c3835" />)}
        <path d="M360 540 L360 488 Q360 476 372 476 L452 476 Q464 476 464 488 L464 540 Z" fill="#8c867c" />
        <rect x={352} y={470} width={120} height={8} rx={4} fill="#a29b90" />
        <path d="M110 540 Q110 500 140 500 L170 500 Q200 500 200 540 Z" fill="#e8dfcf" />
        <circle cx={133} cy={496} r={13} fill="#c24a2b" /><circle cx={160} cy={494} r={13} fill="#d9a82c" /><circle cx={184} cy={497} r={12} fill="#7a9a3a" />
        <rect x={250} y={496} width={10} height={44} rx={4} fill="#d6cdbd" />
        <path d="M255 496 Q230 470 236 440 Q256 464 255 496 Z M255 496 Q282 476 280 446 Q258 466 255 496 Z" fill="#5f8a4c" />
      </g>

      <g style={grp(0.6, 0.82, 40)}>
        <rect x={810} y={548} width={340} height={20} rx={4} fill={p.counter} />
        <rect x={810} y={548} width={340} height={5} rx={2.5} fill="#ffffff" opacity={0.45} />
        <rect x={820} y={568} width={320} height={196} rx={4} fill={p.cabShade} />
        {[826, 987].map((x, i) => (
          <g key={i}>
            <rect x={x} y={574} width={152} height={184} rx={3} fill={p.cab} stroke={p.cabShade} strokeWidth={2} />
            <rect x={x + 36} y={590} width={80} height={5} rx={2.5} fill={p.handle} />
          </g>
        ))}
        <rect x={820} y={764} width={320} height={16} fill="#1f1c19" />
        {[865, 975, 1085].map((x) => (
          <g key={x}>
            <rect x={x - 28} y={470} width={56} height={14} rx={7} fill={p.handle} />
            <line x1={x - 18} y1={484} x2={x - 24} y2={560} stroke={p.handle} strokeWidth={5} />
            <line x1={x + 18} y1={484} x2={x + 24} y2={560} stroke={p.handle} strokeWidth={5} />
          </g>
        ))}
      </g>

      <g style={grp(0.7, 0.95, -50)}>{lamps}</g>
    </svg>
  );
};
