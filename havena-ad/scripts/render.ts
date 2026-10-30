/**
 * npm run render                      → output/havena-publicite-16-9.mp4
 * npm run still -- 45 120 300         → output/stills/frame-XXXX.png pour chaque frame donnée
 * Chromium personnalisé : BROWSER_EXECUTABLE=/chemin/chrome
 */
import fs from "node:fs";
import path from "node:path";
import { bundle } from "@remotion/bundler";
import { renderMedia, renderStill, selectComposition } from "@remotion/renderer";

async function main() {
  const root = path.resolve(__dirname, "..");
  const args = process.argv.slice(2);
  const stillMode = args[0] === "--still";
  const frames = args.slice(1).map(Number).filter(Number.isFinite);
  const browserExecutable = process.env.BROWSER_EXECUTABLE ?? null;
  const chromiumOptions = { enableMultiProcessOnLinux: true };

  console.log("📦 Bundle…");
  const serveUrl = await bundle({ entryPoint: path.join(root, "src/index.ts"), publicDir: path.join(root, "public") });
  const composition = await selectComposition({ serveUrl, id: "HavenaAd", browserExecutable, chromiumOptions });

  if (stillMode) {
    fs.mkdirSync(path.join(root, "output/stills"), { recursive: true });
    for (const frame of frames) {
      const output = path.join(root, `output/stills/frame-${String(frame).padStart(4, "0")}.png`);
      await renderStill({ composition, serveUrl, output, frame, browserExecutable, chromiumOptions });
      console.log(`🖼  ${output}`);
    }
    return;
  }

  const outputLocation = path.join(root, "output/havena-publicite-16-9.mp4");
  fs.mkdirSync(path.dirname(outputLocation), { recursive: true });
  let last = -1;
  await renderMedia({
    composition, serveUrl, outputLocation, codec: "h264", browserExecutable, chromiumOptions, crf: 16,
    onProgress: ({ progress }) => {
      const pct = Math.floor(progress * 100);
      if (pct !== last && pct % 5 === 0) console.log(`🎞  ${(last = pct)} %`);
    },
  });
  console.log(`✅ ${outputLocation}`);
}

main().catch((e) => { console.error(e); process.exit(1); });
