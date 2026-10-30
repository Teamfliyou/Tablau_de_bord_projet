import React from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import { BarChart, Counter, LineChart } from "../components/Charts";
import { IconBadge, Icon } from "../components/Icon";
import { KineticText } from "../components/KineticText";
import { pop, progress } from "../lib/anim";
import { FONT_STACK } from "../lib/theme";
import type { SceneType } from "../lib/types";
import type { SceneComponentProps } from "./SceneProps";

/** Zone utile : au-dessus des sous-titres (qui occupent le bas de l'écran). */
const Layout: React.FC<{ children: React.ReactNode; gap?: number }> = ({ children, gap = 70 }) => (
  <div style={{ position: "absolute", left: 70, right: 70, top: 130, bottom: 560, display: "flex",
    flexDirection: "column", alignItems: "center", justifyContent: "center", gap }}>{children}</div>
);

const Title: React.FC<{ text: string; theme: SceneComponentProps["theme"]; size?: number; delay?: number; hl?: number }> = ({
  text, theme, size = 100, delay = 0, hl = 1,
}) => <KineticText text={text} color={theme.text} accent={theme.accent} fontSize={size} delay={delay} highlightLast={hl} />;

export const HookScene: React.FC<SceneComponentProps> = ({ scene, theme }) => (
  <Layout>
    <IconBadge name={scene.icon} color={theme.accent} glow={theme.accent2} size={340} />
    <Title text={scene.title} theme={theme} size={128} delay={12} hl={2} />
  </Layout>
);

export const StatScene: React.FC<SceneComponentProps> = ({ scene, theme }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const label = pop(frame, fps, 40);
  return (
    <Layout gap={40}>
      <Icon name={scene.icon} size={200} color={theme.accent2} glow={theme.accent2} />
      <Counter value={scene.value ?? 0} prefix={scene.prefix} suffix={scene.suffix} color={theme.accent} glow={theme.accent} delay={8} fontSize={330} />
      <div style={{ opacity: label, transform: `translateY(${(1 - label) * 40}px)`, width: "100%" }}>
        <Title text={scene.title} theme={theme} size={78} delay={36} hl={0} />
      </div>
    </Layout>
  );
};

export const BarsScene: React.FC<SceneComponentProps> = ({ scene, theme }) => (
  <Layout gap={50}>
    <Title text={scene.title} theme={theme} size={84} />
    <BarChart data={scene.data ?? []} theme={theme} delay={14} />
  </Layout>
);

export const LineScene: React.FC<SceneComponentProps> = ({ scene, theme }) => (
  <Layout gap={50}>
    <Title text={scene.title} theme={theme} size={84} />
    <LineChart data={scene.data ?? []} theme={theme} delay={14} />
  </Layout>
);

export const ListScene: React.FC<SceneComponentProps> = ({ scene, theme }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  return (
    <Layout gap={36}>
      <Title text={scene.title} theme={theme} size={84} />
      <div style={{ display: "flex", flexDirection: "column", gap: 22, width: "100%" }}>
        {(scene.items ?? []).map((item, i) => {
          const s = pop(frame, fps, 18 + i * 14, 14);
          return (
            <div key={i} style={{
              display: "flex", alignItems: "center", gap: 36, padding: "22px 36px", borderRadius: 36,
              background: "linear-gradient(90deg,#ffffff1c,#ffffff08)", border: `2px solid ${theme.accent}55`,
              transform: `translateX(${(1 - s) * 700}px)`, opacity: Math.min(1, s * 2),
              backdropFilter: "blur(6px)", fontFamily: FONT_STACK,
            }}>
              <div style={{ width: 96, height: 96, flex: "none", borderRadius: "50%", background: `${theme.accent}22`,
                display: "flex", alignItems: "center", justifyContent: "center" }}>
                <Icon name="check" size={72} color={theme.accent} delay={22 + i * 14} />
              </div>
              <div style={{ fontSize: 50, fontWeight: 800, color: theme.text, lineHeight: 1.1 }}>{item}</div>
            </div>
          );
        })}
      </div>
    </Layout>
  );
};

export const QuestionScene: React.FC<SceneComponentProps> = ({ scene, theme }) => {
  const frame = useCurrentFrame();
  const pulse = 1 + 0.04 * Math.sin(frame / 6);
  return (
    <Layout>
      <div style={{ transform: `scale(${pulse})` }}>
        <IconBadge name={scene.icon} color={theme.accent2} glow={theme.accent2} size={300} />
      </div>
      <Title text={scene.title} theme={theme} size={104} delay={10} hl={2} />
    </Layout>
  );
};

export const OutroScene: React.FC<SceneComponentProps> = ({ scene, theme }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const btn = pop(frame, fps, 34, 10);
  const glow = 0.5 + 0.5 * Math.sin(frame / 7);
  return (
    <Layout>
      <Title text={scene.title} theme={theme} size={118} hl={2} />
      <div style={{
        transform: `scale(${0.6 + 0.4 * btn})`, opacity: Math.min(1, btn * 2), display: "flex", alignItems: "center", gap: 28,
        padding: "34px 70px", borderRadius: 999, fontFamily: FONT_STACK, fontWeight: 900, fontSize: 64,
        color: "#06020f", background: `linear-gradient(90deg, ${theme.accent}, ${theme.accent2})`,
        boxShadow: `0 0 ${40 + 50 * glow}px ${theme.accent}aa`, textTransform: "uppercase",
      }}>
        <Icon name="heart" size={80} color="#06020f" delay={36} strokeWidth={2} />
        {scene.cta ?? "Abonne-toi"}
      </div>
      <div style={{ opacity: progress(frame, 50, 20), transform: `translateY(${Math.sin(frame / 5) * 12}px)` }}>
        <Icon name="arrow" size={120} color={theme.text} strokeWidth={1.6} />
      </div>
    </Layout>
  );
};

export const SCENE_COMPONENTS: Record<SceneType, React.FC<SceneComponentProps>> = {
  hook: HookScene, stat: StatScene, bars: BarsScene, line: LineScene,
  list: ListScene, question: QuestionScene, outro: OutroScene,
};
