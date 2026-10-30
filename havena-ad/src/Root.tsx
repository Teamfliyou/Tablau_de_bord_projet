import React from "react";
import { Composition } from "remotion";
import { loadFont as loadSerif } from "@remotion/google-fonts/CormorantGaramond";
import { loadFont as loadSans } from "@remotion/google-fonts/Montserrat";
import { HavenaAd } from "./compositions/HavenaAd";
import { HavenaAdVertical } from "./compositions/HavenaAdVertical";
import { VERTICAL, VIDEO } from "./styles/theme";

loadSerif("normal", { weights: ["500", "600"], subsets: ["latin", "latin-ext"] });
loadSerif("italic", { weights: ["500", "600"], subsets: ["latin", "latin-ext"] });
loadSans("normal", { weights: ["300", "600"], subsets: ["latin", "latin-ext"] });

export const RemotionRoot: React.FC = () => (
  <>
    <Composition id="HavenaAd" component={HavenaAd} width={VIDEO.width} height={VIDEO.height} fps={VIDEO.fps} durationInFrames={VIDEO.durationInFrames} />
    <Composition id="HavenaAdVertical" component={HavenaAdVertical} width={VERTICAL.width} height={VERTICAL.height} fps={VERTICAL.fps} durationInFrames={VERTICAL.durationInFrames} />
  </>
);
