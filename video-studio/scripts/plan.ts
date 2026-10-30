/** Usage : npm run plan -- <script.txt> — affiche le découpage en scènes sans rendre. */
import fs from "node:fs";
import { planFromText } from "../src/lib/planner";

const file = process.argv[2];
if (!file) { console.error("Usage: npm run plan -- <script.txt>"); process.exit(1); }
const plan = planFromText(fs.readFileSync(file, "utf8"));
console.log(`${plan.title} — ${(plan.totalFrames / plan.fps).toFixed(1)} s, thème ${plan.theme}`);
for (const s of plan.scenes) {
  console.log(`#${s.id} ${s.type.padEnd(8)} ${(s.durationInFrames / plan.fps).toFixed(1).padStart(4)}s  [${s.icon}] ${s.title}`);
}
