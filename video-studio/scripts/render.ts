/**
 * Usage : npm run render -- <script.txt> [sortie.mp4] [--frames=0-90] [--still=120]
 * Script texte → découpage en scènes → rendu MP4 1080x1920.
 */
import path from "node:path";
import fs from "node:fs";
import { bundle } from "@remotion/bundler";
import { renderMedia, renderStill, selectComposition } from "@remotion/renderer";

const args = process.argv.slice(2);
const flags = Object.fromEntries(args.filter((a) => a.startsWith("--")).map((a) => a.replace(/^--/, "").split("=")));
const [scriptPath, outArg] = args.filter((a) => !a.startsWith("--"));
if (!scriptPath) {
  console.error("Usage: npm run render -- <script.txt> [sortie.mp4] [--frames=0-90] [--still=120]");
  process.exit(1);
}
async function main() {
const root = path.resolve(__dirname, "..");
  const script = fs.readFileSync(path.resolve(scriptPath), "utf8");
  const outputLocation = path.resolve(outArg ?? `out/${path.basename(scriptPath, path.extname(scriptPath))}.mp4`);
  fs.mkdirSync(path.dirname(outputLocation), { recursive: true });
  
  const browserExecutable = process.env.BROWSER_EXECUTABLE ?? null;
  const chromiumOptions = { enableMultiProcessOnLinux: true };
  
  console.log("📦 Bundle…");
  const serveUrl = await bundle({ entryPoint: path.join(root, "src/index.ts"), publicDir: path.join(root, "public") });
  const inputProps = { scriptFile: "", script };
  const composition = await selectComposition({ serveUrl, id: "FacelessVideo", inputProps, browserExecutable, chromiumOptions });
  const plan = composition.props.plan as { scenes: { type: string; durationInFrames: number }[] };
  console.log(`🎬 ${plan.scenes.length} scènes : ${plan.scenes.map((s) => s.type).join(" → ")}`);
  console.log(`⏱  ${composition.durationInFrames} frames (${(composition.durationInFrames / composition.fps).toFixed(1)} s)`);
  
  if (flags.still) {
    const png = outputLocation.replace(/\.mp4$/, `-${flags.still}.png`);
    await renderStill({ composition, serveUrl, output: png, frame: Number(flags.still), inputProps, browserExecutable, chromiumOptions });
    console.log(`🖼  ${png}`);
  } else {
    let lastPct = -1;
    const [a, b] = (flags.frames ?? "").split("-").map(Number);
    await renderMedia({
      composition, serveUrl, outputLocation, codec: "h264", inputProps, browserExecutable, chromiumOptions,
      frameRange: flags.frames ? [a, b] : null,
      onProgress: ({ progress }) => {
        const pct = Math.floor(progress * 100);
        if (pct !== lastPct) process.stdout.write(`\r🎞  ${(lastPct = pct)} %`);
      },
    });
    console.log(`\n✅ ${outputLocation}`);
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
