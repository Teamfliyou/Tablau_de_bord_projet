import React from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import { easeOut, pop, progress } from "../lib/anim";
import { FONT_STACK, type Theme } from "../lib/theme";
import type { DataPoint } from "../lib/types";

const fmt = (n: number) => (Math.abs(n) >= 100 ? Math.round(n).toString() : (Math.round(n * 10) / 10).toString().replace(".", ","));

/** Nombre qui compte jusqu'à `value`. */
export const Counter: React.FC<{
  value: number; prefix?: string; suffix?: string; fontSize?: number; color: string; glow: string; delay?: number;
}> = ({ value, prefix = "", suffix = "", fontSize = 300, color, glow, delay = 0 }) => {
  const frame = useCurrentFrame();
  const p = progress(frame, delay, 45, easeOut);
  return (
    <div style={{ fontFamily: FONT_STACK, fontWeight: 900, fontSize, color, lineHeight: 1,
      fontVariantNumeric: "tabular-nums", textShadow: `0 0 60px ${glow}aa`, letterSpacing: -6 }}>
      {prefix}{fmt(value * p)}<span style={{ fontSize: fontSize * 0.55 }}>{suffix}</span>
    </div>
  );
};

/** Barres animées avec valeurs qui montent. */
export const BarChart: React.FC<{ data: DataPoint[]; theme: Theme; width?: number; height?: number; delay?: number }> = ({
  data, theme, width = 900, height = 760, delay = 0,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const max = Math.max(...data.map((d) => d.value));
  const gap = 28;
  const bw = (width - gap * (data.length - 1)) / data.length;
  const labelH = 70;
  const plotH = height - labelH - 60;
  return (
    <svg width={width} height={height} style={{ overflow: "visible", fontFamily: FONT_STACK }}>
      <defs>
        <linearGradient id="barGrad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={theme.accent} />
          <stop offset="100%" stopColor={theme.accent2} />
        </linearGradient>
      </defs>
      {[0.25, 0.5, 0.75, 1].map((g) => (
        <line key={g} x1={0} x2={width} y1={60 + plotH * (1 - g)} y2={60 + plotH * (1 - g)}
          stroke="#ffffff" strokeOpacity={0.1 * progress(frame, delay, 20)} strokeDasharray="8 10" />
      ))}
      {data.map((d, i) => {
        const s = Math.min(1.08, pop(frame, fps, delay + 8 + i * 6, 13));
        const h = (d.value / max) * plotH * s;
        const x = i * (bw + gap);
        const y = 60 + plotH - h;
        const isMax = d.value === max;
        return (
          <g key={i}>
            <rect x={x} y={y} width={bw} height={Math.max(0, h)} rx={20} fill="url(#barGrad)"
              style={{ filter: isMax ? `drop-shadow(0 0 30px ${theme.accent}99)` : undefined }} opacity={isMax ? 1 : 0.8} />
            <text x={x + bw / 2} y={y - 18} textAnchor="middle" fill="#fff" fontSize={46} fontWeight={800}
              opacity={Math.min(1, s)}>{fmt(d.value * Math.min(1, s))}</text>
            <text x={x + bw / 2} y={height - 18} textAnchor="middle" fill={theme.muted} fontSize={40} fontWeight={700}>
              {d.label}
            </text>
          </g>
        );
      })}
    </svg>
  );
};

/** Courbe qui se trace avec aire dégradée et point lumineux. */
export const LineChart: React.FC<{ data: DataPoint[]; theme: Theme; width?: number; height?: number; delay?: number }> = ({
  data, theme, width = 900, height = 760, delay = 0,
}) => {
  const frame = useCurrentFrame();
  const labelH = 70, pad = 50;
  const plotH = height - labelH - pad * 2;
  const max = Math.max(...data.map((d) => d.value));
  const min = Math.min(...data.map((d) => d.value));
  const span = max - min || 1;
  const pts = data.map((d, i) => ({
    x: (i / (data.length - 1)) * width,
    y: pad + plotH - ((d.value - min) / span) * plotH * 0.9 - plotH * 0.05,
  }));
  const path = pts.map((p, i) => `${i ? "L" : "M"}${p.x.toFixed(1)} ${p.y.toFixed(1)}`).join(" ");
  const area = `${path} L${width} ${pad + plotH} L0 ${pad + plotH} Z`;
  const p = progress(frame, delay, 60);
  // Point courant le long de la polyligne
  const seg = p * (pts.length - 1);
  const i0 = Math.min(pts.length - 2, Math.floor(seg));
  const f = seg - i0;
  const cur = { x: pts[i0].x + (pts[i0 + 1].x - pts[i0].x) * f, y: pts[i0].y + (pts[i0 + 1].y - pts[i0].y) * f };
  const val = data[i0].value + (data[i0 + 1].value - data[i0].value) * f;
  return (
    <svg width={width} height={height} style={{ overflow: "visible", fontFamily: FONT_STACK }}>
      <defs>
        <linearGradient id="areaGrad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={theme.accent} stopOpacity={0.45} />
          <stop offset="100%" stopColor={theme.accent} stopOpacity={0} />
        </linearGradient>
        <clipPath id="reveal"><rect x={-20} y={-40} width={(width + 40) * p} height={height + 80} /></clipPath>
      </defs>
      {[0, 1, 2, 3].map((g) => (
        <line key={g} x1={0} x2={width} y1={pad + (plotH * g) / 3} y2={pad + (plotH * g) / 3} stroke="#fff" strokeOpacity={0.1} strokeDasharray="8 10" />
      ))}
      <g clipPath="url(#reveal)">
        <path d={area} fill="url(#areaGrad)" />
        <path d={path} fill="none" stroke={theme.accent} strokeWidth={10} strokeLinecap="round" strokeLinejoin="round"
          style={{ filter: `drop-shadow(0 0 16px ${theme.accent})` }} />
      </g>
      {data.map((d, i) => (
        <text key={i} x={pts[i].x} y={height - 18} textAnchor={i === 0 ? "start" : i === data.length - 1 ? "end" : "middle"}
          fill={theme.muted} fontSize={40} fontWeight={700}>{d.label}</text>
      ))}
      <circle cx={cur.x} cy={cur.y} r={20} fill="#fff" style={{ filter: `drop-shadow(0 0 22px ${theme.accent})` }} />
      <text x={Math.min(width - 90, Math.max(90, cur.x))} y={cur.y - 44} textAnchor="middle" fill="#fff" fontSize={56} fontWeight={900}>
        {fmt(val)}
      </text>
    </svg>
  );
};
