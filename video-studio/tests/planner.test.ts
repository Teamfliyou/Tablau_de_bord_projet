import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import { buildSubtitleWords, planFromText } from "../src/lib/planner";
import { extractNumber, parseScript } from "../src/lib/scriptParser";

test("la démo dure exactement 30 s", () => {
  const plan = planFromText(fs.readFileSync("content/demo.txt", "utf8"));
  assert.equal(plan.totalFrames, 900);
  const sum = plan.scenes.reduce((a, s) => a + s.durationInFrames, 0);
  assert.equal(sum - (plan.scenes.length - 1) * plan.transitionFrames, 900);
});

test("la durée de la voix off prime sur @duration", () => {
  const plan = planFromText("@duration 30\n\nBonjour à tous.\n\nMerci beaucoup.", 12);
  assert.equal(plan.totalFrames, 360);
});

test("découpage automatique d'un texte brut", () => {
  const p = parseScript("Première phrase. Deuxième phrase. Troisième ? Quatrième phrase. Cinquième.");
  assert.equal(p.scenes.length, 3);
  assert.equal(p.scenes[0].type, "hook");
  assert.equal(p.scenes[2].type, "outro");
});

test("bars sans données : pas de données inventées", () => {
  const p = parseScript("[bars]\nTexte sans données 42%.");
  assert.notEqual(p.scenes[0].type, "bars");
});

test("extraction de nombres", () => {
  assert.deepEqual(extractNumber("Une hausse de 87 % ce mois"), { value: 87, prefix: "", suffix: "%" });
  assert.equal(extractNumber("1,5 M€")?.value, 1.5);
});

test("les sous-titres couvrent tous les mots, sans retour en arrière", () => {
  const plan = planFromText(fs.readFileSync("content/demo.txt", "utf8"));
  const words = buildSubtitleWords(plan);
  const expected = plan.scenes.reduce((n, s) => n + s.text.split(/\s+/).length, 0);
  assert.equal(words.length, expected);
  for (const w of words) assert.ok(w.to >= w.from && w.to <= plan.totalFrames);
});
