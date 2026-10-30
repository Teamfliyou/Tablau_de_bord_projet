import React, { useMemo } from "react";
import { AbsoluteFill, Audio, staticFile } from "remotion";
import { TransitionSeries, linearTiming, springTiming, type TransitionPresentation } from "@remotion/transitions";
import { slide } from "@remotion/transitions/slide";
import { fade } from "@remotion/transitions/fade";
import { wipe } from "@remotion/transitions/wipe";
import { flip } from "@remotion/transitions/flip";
import { Background } from "./components/Background";
import { CameraRig, type CameraMove } from "./components/CameraRig";
import { Subtitles } from "./components/Subtitles";
import { buildSubtitleWords } from "./lib/planner";
import { THEMES } from "./lib/theme";
import type { VideoPlan } from "./lib/types";
import { SCENE_COMPONENTS } from "./scenes/scenes";

const CAMERA: CameraMove[] = ["push", "panLeft", "pull", "panRight", "tilt"];

/** Transitions utilisées en alternance (toutes en code, via @remotion/transitions). */
const transitionFor = (i: number): TransitionPresentation<any> => {
  switch (i % 4) {
    case 0: return slide({ direction: "from-bottom" });
    case 1: return wipe({ direction: "from-left" });
    case 2: return flip({ direction: "from-right" });
    default: return fade();
  }
};

export const Video: React.FC<{ plan: VideoPlan }> = ({ plan }) => {
  const theme = THEMES[plan.theme];
  const words = useMemo(() => buildSubtitleWords(plan), [plan]);
  const T = plan.transitionFrames;

  return (
    <AbsoluteFill style={{ background: theme.bg1 }}>
      <Background theme={theme} />
      <TransitionSeries>
        {plan.scenes.flatMap((scene, i) => {
          const Scene = SCENE_COMPONENTS[scene.type];
          const nodes = [
            <TransitionSeries.Sequence key={`s${scene.id}`} durationInFrames={scene.durationInFrames}>
              <CameraRig move={CAMERA[i % CAMERA.length]} duration={scene.durationInFrames}>
                <Scene scene={scene} theme={theme} />
              </CameraRig>
            </TransitionSeries.Sequence>,
          ];
          if (i < plan.scenes.length - 1) {
            nodes.push(
              <TransitionSeries.Transition
                key={`t${scene.id}`}
                presentation={transitionFor(i)}
                timing={i % 4 === 0 ? springTiming({ durationInFrames: T, config: { damping: 200 } }) : linearTiming({ durationInFrames: T })}
              />,
            );
          }
          return nodes;
        })}
      </TransitionSeries>
      <Subtitles words={words} theme={theme} />
      {plan.voice ? <Audio src={staticFile(plan.voice)} /> : null}
    </AbsoluteFill>
  );
};
