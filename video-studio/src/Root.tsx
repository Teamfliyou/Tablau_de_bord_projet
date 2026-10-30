import React from "react";
import { Composition, staticFile, type CalculateMetadataFunction } from "remotion";
import { getAudioDurationInSeconds } from "@remotion/media-utils";
import { loadFont } from "@remotion/google-fonts/Montserrat";
import { planFromText } from "./lib/planner";
import { parseScript } from "./lib/scriptParser";
import { VIDEO } from "./lib/theme";
import type { VideoPlan } from "./lib/types";
import { Video } from "./Video";

loadFont("normal", { weights: ["700", "800", "900"], subsets: ["latin", "latin-ext"] });

export type VideoProps = {
  /** Chemin d'un script dans /public (ex. "scripts/demo.txt"). */
  scriptFile: string;
  /** Alternative : script passé directement (prioritaire si non vide). */
  script?: string;
  /** Rempli automatiquement par calculateMetadata. */
  plan?: VideoPlan;
};

const calculateMetadata: CalculateMetadataFunction<VideoProps> = async ({ props }) => {
  const source = props.script?.trim()
    ? props.script
    : await fetch(staticFile(props.scriptFile)).then((r) => {
        if (!r.ok) throw new Error(`Script introuvable : ${props.scriptFile}`);
        return r.text();
      });

  // Voix off facultative : sa durée pilote la durée totale de la vidéo.
  const voice = parseScript(source).voice;
  let audioSeconds: number | undefined;
  if (voice) {
    try {
      audioSeconds = await getAudioDurationInSeconds(staticFile(voice));
    } catch {
      console.warn(`Voix off "${voice}" introuvable : durée estimée à partir du texte.`);
    }
  }
  const plan = planFromText(source, audioSeconds);
  if (!audioSeconds) plan.voice = undefined;
  return { durationInFrames: plan.totalFrames, props: { ...props, plan } };
};

export const RemotionRoot: React.FC = () => (
  <Composition
    id="FacelessVideo"
    component={({ plan }: VideoProps) => (plan ? <Video plan={plan} /> : null)}
    width={VIDEO.width}
    height={VIDEO.height}
    fps={VIDEO.fps}
    durationInFrames={900}
    defaultProps={{ scriptFile: "scripts/demo.txt" } as VideoProps}
    calculateMetadata={calculateMetadata}
  />
);
