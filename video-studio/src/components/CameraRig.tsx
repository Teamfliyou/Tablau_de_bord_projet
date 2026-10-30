import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { easeInOut, progress } from "../lib/anim";

export type CameraMove = "push" | "pull" | "panLeft" | "panRight" | "tilt";

/** Caméra simulée : zoom / travelling lents + léger tremblement "main levée". */
export const CameraRig: React.FC<{
  move?: CameraMove;
  duration: number;
  shake?: number;
  children: React.ReactNode;
}> = ({ move = "push", duration, shake = 1, children }) => {
  const frame = useCurrentFrame();
  const { width } = useVideoConfig();
  const p = progress(frame, 0, duration, easeInOut);
  const t = frame / 30;

  let scale = 1, x = 0, y = 0, rot = 0;
  if (move === "push") scale = 1 + 0.08 * p;
  if (move === "pull") scale = 1.1 - 0.08 * p;
  if (move === "panLeft") { scale = 1.06; x = width * 0.02 * (1 - 2 * p); }
  if (move === "panRight") { scale = 1.06; x = width * 0.02 * (2 * p - 1); }
  if (move === "tilt") { scale = 1.05; rot = -1 + 2 * p; }

  x += Math.sin(t * 1.3) * 4 * shake;
  y += Math.cos(t * 1.7) * 5 * shake;
  rot += Math.sin(t * 0.9) * 0.25 * shake;

  return (
    <AbsoluteFill style={{ transform: `translate(${x}px, ${y}px) scale(${scale}) rotate(${rot}deg)` }}>
      {children}
    </AbsoluteFill>
  );
};
